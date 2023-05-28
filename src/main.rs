mod mandelbrot;

use sdl2::{
    pixels::{Color, PixelFormatEnum},
    event::{Event, WindowEvent},
    keyboard::Keycode
};
use std::iter;
use std::time::Duration;
use mandelbrot::*;

type Real = f32;
 
pub fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
 
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
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        let mut resized = false;
        let mut redraw = false;

        for event in iter::once(
            event_pump.wait_event()
        ).chain(
            event_pump.poll_iter()
        ) {
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
            (w, h) = canvas.output_size().unwrap();
            println!("Window resized to {}x{}.", w, h);
            texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, w, h).unwrap();
            redraw = true;
        }

        if redraw {
            canvas.clear();
            texture.with_lock(None, |buf, pitch| -> () {
                for y in 0..usize::try_from(h).unwrap() {
                    for x in 0..usize::try_from(w).unwrap() {
                        let pixel_index: usize = pitch * y + x * 3;
                        (
                            buf[pixel_index],
                            buf[pixel_index + 1],
                            buf[pixel_index + 2]
                        ) = (127, 100, 127);
                    }
                }
            });
            canvas.copy(&texture, None, None);
            canvas.present();
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }
}

