mod mandelbrot;

use sdl2::{
    pixels::{Color, PixelFormatEnum},
    event::{Event, WindowEvent},
    keyboard::Keycode
};
use std::{
    iter,
    time::Duration,
    task::Poll,
    vec::Vec
};
use tokio::{time, runtime::Runtime, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use pumptasks::SdlDispatcher;
use futures::poll;
use mandelbrot::*;

type Real = f64;

#[derive(Clone)]
enum CustomMessages {
    ResizeTexture,
    MandelbrotReady(Vec<u8>),
}

pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let event_subsystem = sdl_context.event().unwrap();
     
    let window = video_subsystem.window("Mandelbrot Explorer", 800, 600)
        .resizable()
        .opengl()
        .build()
        .unwrap();
 
    let mut canvas = window.into_canvas().build().unwrap();
    let texture_creator = canvas.texture_creator();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let (mut w, mut h) = canvas.output_size().unwrap();
    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, w, h).unwrap();
    let (ui_executor, ui_dispatcher) = pumptasks::new_executor_and_dispatcher::<CustomMessages, ()>(&event_subsystem);
    let tokio_runtime = Runtime::new().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let ui_dispatcher = move || -> SdlDispatcher {
        ui_dispatcher.clone()
    };

    let mut resize_timer_task: Option<JoinHandle<()>> = None;
    let mut mandelbrot_task: Option<(JoinHandle<()>, CancellationToken)> = None;

    'running: loop {
        let mut resized = false;
        let mut resize_texture = false;
        let mut redraw = false;
        let mut mandelbrot_ready = false;

        for event in iter::once(
            event_pump.wait_event()
        ).chain(
            event_pump.poll_iter()
        ) {
            if let Some(task) = ui_executor.handle_sdl_event::<CustomMessages, ()>(&event) {
                match task.input() {
                    CustomMessages::ResizeTexture => resize_texture = true,
                    CustomMessages::MandelbrotReady(data) => {
                        // Lock texture and copy data
                        texture.with_lock(None, |buf, pitch| {
                            for y in 0..h {
                                for x in 0..w {
                                    let pixel_index = usize::try_from(pitch as u32 * y + x * 3).unwrap();
                                    let mandelbrot_index = usize::try_from((w * y + x) * 3).unwrap();
                                    buf[pixel_index] = data[mandelbrot_index];
                                    buf[pixel_index + 1] = data[mandelbrot_index + 1];
                                    buf[pixel_index + 2] = data[mandelbrot_index + 2];
                                }
                            }
                        });
                        mandelbrot_ready = true;
                    }
                }

                task.complete(());
                continue;
            }

            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                Event::Window {
                    win_event: WindowEvent::Resized(..) | WindowEvent::SizeChanged(..),
                    ..
                } => {
                    resized = true;
                }
                _ => {}
            }
        }

        if resized {
            if let Some(task) = resize_timer_task.take() {
                task.abort();
            }

            if let Some((task, token)) = mandelbrot_task.take() {
                token.cancel();
                task.abort();
            }

            let disp = ui_dispatcher();
            resize_timer_task = Some(tokio_runtime.spawn(async move {
                time::sleep(Duration::from_millis(1000)).await;
                disp.spawn::<CustomMessages, ()>(CustomMessages::ResizeTexture).await;
            }));
        }

        if resize_texture {
            (w, h) = canvas.output_size().unwrap();
            texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, w, h).unwrap();
            redraw = true;
        }

        if redraw {
            if let Some((task, token)) = mandelbrot_task.take() {
                token.cancel();
                task.abort();
            }

            let disp = ui_dispatcher();
            let cancellation_token = CancellationToken::new();
            let cancellation_token_clone = cancellation_token.clone();
            mandelbrot_task = Some((tokio_runtime.spawn(async move{
                let scale: Real = 4.0 / Real::from(h);
                if let Some(buf) = mandelbrot::mandelbrot_set(
                    Real::from(-(w as i32 / 2)) * scale,
                    Real::from(-(h as i32 / 2)) * scale,
                    scale,
                    w, h,
                    cancellation_token_clone
                ) {
                    disp.spawn::<CustomMessages, ()>(CustomMessages::MandelbrotReady(buf)).await;
                }
            }), cancellation_token));
        }

        if mandelbrot_ready {
            canvas.clear();
            canvas.copy(&texture, None, None);
            canvas.present();
        }
    }
}

