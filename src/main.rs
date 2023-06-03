mod mandelbrot;

use sdl2::{
    pixels::{Color, PixelFormatEnum},
    event::{Event, WindowEvent},
    keyboard::Keycode
};
use std::iter;
use std::time::Duration;
use tokio::{time, runtime::Runtime, task::JoinHandle};
use pumptasks::SdlDispatcher;
use mandelbrot::*;

type Real = f64;

#[derive(Clone)]
enum CustomMessages {
    ResizeTexture
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

    'running: loop {
        let mut resized = false;
        let mut resize_texture = false;
        let mut redraw = false;
        let mut redraw_task: Option<JoinHandle<()>> = None;

        for event in iter::once(
            event_pump.wait_event()
        ).chain(
            event_pump.poll_iter()
        ) {
            if ui_executor.handle_sdl_event(&event) {
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
            if let Some(task) = redraw_task.take() {
                task.abort();
            }

            let disp = ui_dispatcher();
            let t = &mut resize_texture;
            redraw_task = Some(tokio_runtime.spawn(async move {
                time::sleep(Duration::from_millis(1000)).await;

                disp.spawn(async {
                    *t = true;
                }).await;
            }));
        }

        if resize_texture {
            (w, h) = canvas.output_size().unwrap();
            println!("Window resized to {}x{}.", w, h);
            texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, w, h).unwrap();
            resize_texture = false;
            redraw = true;
        }

        if redraw {
            canvas.clear();
            texture.with_lock(None, |buf, pitch| -> () {
                let scale: Real = 4.0 / Real::from(h);

                for (y, y_chart) in (0..h).zip((-(h as i32 / 2)..(h as i32 / 2)).rev()) {
                    for (x, x_chart) in (0..w).zip(-(w as i32 / 2)..w as i32 / 2) {
                        let pixel_index = usize::try_from(pitch as u32 * y + x * 3).unwrap();
                        (
                            buf[pixel_index],
                            buf[pixel_index + 1],
                            buf[pixel_index + 2]
                        ) = match bounded((Real::from(x_chart) * scale, Real::from(y_chart) * scale), 1000) {
                            (true, ..) => (0, 0, 0),
                            _ => (255, 255, 255)
                        }
                    }
                }
            });
            canvas.copy(&texture, None, None);
            canvas.present();
            redraw = false;
        }
    }
}

