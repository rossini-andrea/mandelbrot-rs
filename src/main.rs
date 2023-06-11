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
use salty_broth::sdl_app::*;
use tokio::{time, runtime::Runtime, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use sdl_dispatch::SdlDispatcher;
use futures::poll;
use mandelbrot::*;

type Real = f64;

#[derive(Clone)]
enum CustomMessages {
    ResizeTexture,
    MandelbrotReady(Vec<(u8, u8, u8)>),
}

pub fn main() {
    let apprunner = AppBuilder::new("Mandelbrot Explorer")
        .window_size(800, 600)
        .with_dispatch()
        .with_tokio()
        .passive_event_loop()
        .build();

    let tokio = Runtime::new();
    let _guard = tokio.enter();
}
/* 
    let texture_creator = canvas.texture_creator();

    canvas.set_draw_color(Color::RGB(0, 255, 255));
    canvas.clear();
    canvas.present();
    let (mut w, mut h) = canvas.output_size().unwrap();
    let mut texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, w, h).unwrap();
    
    sdl_dispatch::register_task_type::<CustomMessages, ()>(&event_subsystem);

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
            if let Some(task) = sdl_dispatch::handle_sdl_event::<CustomMessages, ()>(&event) {
                match task.input() {
                    CustomMessages::ResizeTexture => resize_texture = true,
                    CustomMessages::MandelbrotReady(data) => {
                        // Lock texture and copy data
                        texture.with_lock(None, |buf, pitch| {
                            for y in 0..h {
                                for x in 0..w {
                                    let pixel_index = usize::try_from(pitch as u32 * y + x * 3).unwrap();
                                    let mandelbrot_index = usize::try_from(w * y + x).unwrap();
                                    (
                                        buf[pixel_index],
                                        buf[pixel_index + 1],
                                        buf[pixel_index + 2]
                                    ) = data[mandelbrot_index];
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
                    - Real::from(w / 2) * scale,
                    - Real::from(h / 2) * scale,
                    scale,
                    w as usize, h as usize, 20000,
                    &vec![(255, 255, 255), (0, 0, 0)],
                    cancellation_token_clone
                ).await {
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
*/
