use salty_broth::{
    dispatch_handlers,
    sdl_app,
    sdl_app::AppSystem,
};
use sdl_dispatch::SdlPumpTask;
use sdl2::{
    render::{ Canvas, Texture, TextureCreator },
    video::{ Window, WindowContext },
    event::Event,
    pixels::PixelFormatEnum,
};
use tokio::{
    time,
    time::Duration,
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use crate::mandelbrot;

type Real = f64;

pub struct MainApp<'a> {
    app_system: &'a mut AppSystem,
    resize_timer_task: Option<JoinHandle<()>>, 
    mandelbrot_task: Option<(JoinHandle<()>, CancellationToken)>,
    texture: Texture<'a>,
    w: u32, h: u32,
}

impl<'a> From<&'a mut AppSystem> for MainApp<'a> {
    fn from(app_system: &'a mut AppSystem) -> Self {
        let (w, h) = app_system.canvas().output_size().unwrap();
        let texture = {
            app_system.texture_creator().create_texture_streaming(PixelFormatEnum::RGB24, w, h).unwrap()
        };
        Self {
            app_system,
            resize_timer_task: None, 
            mandelbrot_task: None, 
            texture,
            w, h
        }
    }
}

impl sdl_app::App for MainApp<'_> {
    fn start(&mut self) {
        self.app_system.canvas().clear();
        self.app_system.canvas().present();
    }

    fn resized(&mut self) {
        if let Some(task) = self.resize_timer_task.take() {
            task.abort();
        }

        if let Some((task, token)) = self.mandelbrot_task.take() {
            token.cancel();
            task.abort();
        }

        let disp = self.app_system.dispatcher();
        self.resize_timer_task = Some(tokio::spawn(async move {
            time::sleep(Duration::from_millis(1000)).await;
            disp.spawn::<ResizeTexture, ()>(ResizeTexture{}).await;
        }));
    }

    fn stop(&mut self) {

    }
}

struct ResizeTexture {}
struct Redraw {}
struct MandelbrotReady {
    buf: Vec<(u8, u8, u8)>,
}

dispatch_handlers! {
    MainApp<'_>,
    fn resize_texture(&self, task: SdlPumpTask<ResizeTexture, ()>) {
        (self.w, self.h) = self.app_system.canvas().output_size().unwrap();
        self.texture = self.app_system.texture_creator().create_texture_streaming(PixelFormatEnum::RGB24, self.w, self.h).unwrap();
        task.complete(());
        // TODO: Dispatch a redraw message.
    }

    fn redraw(&self, _msg: Redraw) {
        if let Some((task, token)) = self.mandelbrot_task.take() {
            token.cancel();
            task.abort();
        }

        let disp = self.app_system.dispatcher();
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
                disp.spawn::<MandelbrotReady, ()>(MandelbrotReady{buf}).await;
            }
        }), cancellation_token));
    }

    fn mandelbrot_ready(&self, task: SdlPumpTask<MandelbrotReady, ()>) {
        let image = task.input().buf;
        task.complete(());

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

        let canvas = self.app_system.canvas();
        canvas.clear();
        canvas.copy(&self.texture, None, None);
        canvas.present();
    }
}
