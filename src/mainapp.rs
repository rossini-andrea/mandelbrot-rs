use color_eyre::Result;
use salty_broth::{
    dispatch_handlers,
    sdl_app,
};
use sdl_dispatch::SdlPumpTask;
use sdl2::{
    render::{ Canvas, RenderTarget, Texture, TextureCreator },
    video::{ Window, WindowContext },
    event::Event,
    mouse::MouseButton,
    pixels::{ Color, PixelFormatEnum },
    rect::{ Rect, Point },
};
use std::mem;
use tokio::{
    time,
    time::Duration,
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use crate::mandelbrot;
use crate::mathutils;

type Real = f64;

/// Represents the handler for SDL events, keeps track of redraw
/// processes.
pub struct MainApp {
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    resize_timer_task: Option<JoinHandle<Result<(), String>>>, 
    mandelbrot_task: Option<(JoinHandle<Result<(), String>>, CancellationToken)>,
    texture: Texture,
    w: u32, h: u32,
    selection_center: Option<Point>,
    selection: Option<Rect>,
}

impl TryFrom<Canvas<Window>> for MainApp {
    type Error = String;

    fn try_from(canvas: Canvas<Window>) -> Result<Self, Self::Error> {
        let (w, h) = canvas.output_size()?;
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, w, h)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            canvas,
            texture_creator,
            resize_timer_task: None, 
            mandelbrot_task: None, 
            texture,
            w, h,
            selection_center: None,
            selection: None,
        })
    }
}

impl Drop for MainApp {
    fn drop(&mut self) {
        unsafe {
        }
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
            sdl_dispatch::spawn::<ResizeTexture, Result<(), String>>(ResizeTexture{}).await
                .map_err(|e| e.to_string())??;
            Ok(())
        }));
    }

    /// Stop should handle the return value to the main loop.
    fn stop(&mut self) {

    }

    fn sdl_event(&mut self, event: Event) {
        match event {
            Event::MouseButtonDown{ mouse_btn: MouseButton::Left, x, y, ..} => {
                self.selection_center = Some(Point::new(x, y))
            },
            Event::MouseMotion{x, y, ..} => {
                if let Some(center) = self.selection_center {
                    self.selection = Some(mathutils::selection_from_center_with_ratio(
                        center,
                        Point::new(x, y),
                        self.w as f32 / self.h as f32
                    ));
                    if let Some(err) = self.render().err() {
                        println!("{}", err);
                    }
                }
            },
            Event::MouseButtonUp{mouse_btn: MouseButton::Left, x, y, ..} => {
                self.selection_center = None;
                self.selection = None;
                if let Some(err) = self.render().err() {
                    println!("{}", err);
                }
            },
            _ => {}
        }
    }
}

struct ResizeTexture {}
struct Redraw {}
struct MandelbrotReady {
    buf: Vec<(u8, u8, u8)>,
}

dispatch_handlers! {
    MainApp ,

    fn resize_texture(&mut self, task: SdlPumpTask<ResizeTexture, Result<(), String>>) {
        let result = (|| -> Result<(), String> {
            (self.w, self.h) = self.canvas.output_size()?;
        
            unsafe {
                mem::replace(
                    &mut self.texture,
                    self.texture_creator.create_texture_streaming(PixelFormatEnum::RGB24, self.w, self.h)
                        .map_err(|e| e.to_string())?
                ).destroy();
            }

            sdl_dispatch::send::<Redraw>(Redraw{});
            Ok(())
        })();
    
        task.complete(result);
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
            if let Some(buf) = mandelbrot::compute_set(
                - Real::from(w / 2) * scale,
                - Real::from(h / 2) * scale,
                scale,
                w as usize, h as usize, 20000,
                &vec![(255, 255, 255), (0, 0, 0)],
                cancellation_token_clone
            ).await {
                sdl_dispatch::spawn::<MandelbrotReady, Result<(), String>>(MandelbrotReady{buf})
                    .await
                    .map_err(|_| "Task canceled")??;
            }

            Ok(())
        }), cancellation_token));
    }

    fn mandelbrot_ready(&mut self, task: SdlPumpTask<MandelbrotReady, Result<(), String>>) {
        let result: Result<(), String> = (|| {
            let image = &task.input().buf;

            // Lock texture and copy data
            _ = self.texture.with_lock(None, |buf, pitch| -> Result<(), String> {
                for y in 0..self.h as usize {
                    for x in 0..self.w as usize {
                        let pixel_index = pitch * y + x * 3;
                        let mandelbrot_index = self.w as usize * y + x;
                        (
                            buf[pixel_index],
                            buf[pixel_index + 1],
                            buf[pixel_index + 2]
                        ) = image[mandelbrot_index];
                    }
                }

                Ok(())
            })?;

            self.render()?;
            Ok(())
        })();

        task.complete(result);
    }
}

impl MainApp {
    fn render(&mut self) -> Result<(), String> {
        self.canvas.clear();
        self.canvas.copy(&self.texture, None, None)?;

        if let Some(rect) = self.selection {
            self.canvas.set_draw_color(Color::RGB(255, 0, 0));
            self.canvas.draw_rect(rect)?;
        }
        
        self.canvas.present();
        Ok(())
    }
}

