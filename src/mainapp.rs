use salty_broth::{
    dispatch_handlers,
    sdl_app,
};
use sdl_dispatch::SdlPumpTask;
use sdl2::{
    render::{ Canvas, Texture, TextureCreator, TextureValueError },
    video::{ Window, WindowContext },
    event::Event,
    pixels::PixelFormatEnum,
};
use std::{rc::Rc, cell::RefCell};
use tokio::{
    time,
    time::Duration,
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use crate::mandelbrot;

type Real = f64;

/// Represents the handler for SDL events, keeps track of redraw
/// processes.
pub struct MainApp {
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    resize_timer_task: Option<JoinHandle<()>>, 
    mandelbrot_task: Option<(JoinHandle<()>, CancellationToken)>,
    texture: Texture,
    w: u32, h: u32,
}

impl From<Canvas<Window>> for MainApp {
    fn from(canvas: Canvas<Window>) -> Self {
        let (w, h) = canvas.output_size().unwrap();
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, w, h).unwrap();
        Self {
            canvas,
            texture_creator,
            resize_timer_task: None, 
            mandelbrot_task: None, 
            texture,
            w, h
        }
    }
}

impl Drop for MainApp {
    fn drop(&mut self) {
    }
}

impl sdl_app::App for MainApp {
    /// Application start does simply clear the main window.
    fn start(&mut self) {
        self.canvas.clear();
        self.canvas.present();
    }

    /// When the application is resized we launch an asynchronous
    /// wait of 1 second, aborting any other task.
    fn resized(&mut self) {
        if let Some(task) = self.resize_timer_task.take() {
            task.abort();
        }

        if let Some((task, token)) = self.mandelbrot_task.take() {
            token.cancel();
            task.abort();
        }

        self.resize_timer_task = Some(tokio::spawn(async move {
            time::sleep(Duration::from_millis(1000)).await;
            sdl_dispatch::spawn::<ResizeTexture, ()>(ResizeTexture{}).await;
        }));
    }

    /// Stop should handle the return value to the main loop.
    fn stop(&mut self) {

    }
}

struct ResizeTexture {}
struct Redraw {}
struct MandelbrotReady {
    buf: Vec<(u8, u8, u8)>,
}

dispatch_handlers! {
    MainApp ,

    fn resize_texture(&mut self, task: SdlPumpTask<ResizeTexture, ()>) {
        (self.w, self.h) = self.canvas.output_size().unwrap();
        self.texture = self.texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, self.w, self.h)
            .unwrap();
        task.complete(());
        // TODO: Dispatch a redraw message.
    }

    fn redraw(&mut self, _msg: Redraw) {
        if let Some((task, token)) = self.mandelbrot_task.take() {
            token.cancel();
            task.abort();
        }

        let cancellation_token = CancellationToken::new();
        let cancellation_token_clone = cancellation_token.clone();
        let (w, h) = (self.w, self. h);
        self.mandelbrot_task = Some((tokio::spawn(async move{
            let scale: Real = 4.0 / Real::from(h);
            if let Some(buf) = mandelbrot::mandelbrot_set(
                - Real::from(w / 2) * scale,
                - Real::from(h / 2) * scale,
                scale,
                w as usize, h as usize, 20000,
                &vec![(255, 255, 255), (0, 0, 0)],
                cancellation_token_clone
            ).await {
                sdl_dispatch::spawn::<MandelbrotReady, ()>(MandelbrotReady{buf}).await;
            }
        }), cancellation_token));
    }

    fn mandelbrot_ready(&mut self, task: SdlPumpTask<MandelbrotReady, ()>) {
        let image = &task.input().buf;

        // Lock texture and copy data
        self.texture.with_lock(None, |buf, pitch| {
            for y in 0..self.h {
                for x in 0..self.w {
                    let pixel_index = usize::try_from(pitch as u32 * y + x * 3).unwrap();
                    let mandelbrot_index = usize::try_from(self.w * y + x).unwrap();
                    (
                        buf[pixel_index],
                        buf[pixel_index + 1],
                        buf[pixel_index + 2]
                    ) = image[mandelbrot_index];
                }
            }
        });

        task.complete(());

        self.canvas.clear();
        self.canvas.copy(&self.texture, None, None);
        self.canvas.present();
    }
}
