use salty_broth::{
    dispatch_handlers,
    sdl_app,
    sdl_app::AppSystem,
};
use sdl_dispatch::SdlPumpTask;
use sdl2::{
    render::{ Canvas, Texture, TextureCreator },
    video::Window,
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

type Real = f32;

#[derive(Default)]
pub struct App<'a> {
    app_system: &'a AppSystem,
    resize_timer_task: Option<JoinHandle<()>>, 
    mandelbrot_task: Option<(JoinHandle<()>, CancellationToken)>,
    texture_creator: TextureCreator<Canvas<Window>>,
    texture: Texture<'a>,
    w: u32, h: u32,
}

impl From<&AppSystem> for App<'_> {
    fn from(app_system: &AppSystem) -> Self {
        let texture_creator = app_system.canvas().texture_creator();
        let (w, h) = app_system.canvas().output_size().unwrap();
        let texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, w, h).unwrap();
        Self {
            app_system,
            resize_timer_task: None, 
            mandelbrot_task: None, 
            texture_creator,
            texture,
            w, h
        }
    }
}

impl sdl_app::App for App<'_> {
    fn start(&self) {
        self.app_system.canvas().clear();
        self.app_system.canvas().present();
    }

    fn resized(&self) {
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
            disp.spawn(ResizeTexture{}).await;
        }));
    }
}

struct ResizeTexture {}
struct Redraw {}
struct MandelbrotReady {
    buf: Vec<(u8, u8, u8)>,
}

dispatch_handlers! {
    App,
    fn resize_texture(&self, task: SdlPumpTask<ResizeTexture, ()>) {
        (self.w, self.h) = self.canvas.output_size().unwrap();
        self.texture = self.texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, self.w, self.h).unwrap();
        task.complete();
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
        task.complete();

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
                    ) = task.input().buf[mandelbrot_index];
                }
            }
        });
        self.canvas.clear();
        self.canvas.copy(&self.texture, None, None);
        self.canvas.present();
    }
}
