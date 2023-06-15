use sdl2::{
    Sdl,
    VideoSubsystem,
    EventSubsystem,
    EventPump,
    event::{Event, WindowEvent},
    keyboard::Keycode,
    render::{Canvas, TextureCreator},
    video::{Window, WindowContext},
};
use sdl_dispatch::SdlDispatcher;
use std::{
    iter,
};

#[macro_export]
macro_rules! dispatch_handlers {
    (
        $app:ident < $( $gen:tt ),+ >,
        $(
            fn $func_name:ident (&$self:ident, $message:ident : $type:ty )
            $impl:block
        )
        *
    ) => {
        impl $app<$($gen,)*> {
            $(
            fn $func_name(&$self, $message: $type)
                $impl
            )*
        }

        impl sdl_app::DispatchHandler for $app<$($gen,)*> {
            fn handle_dispatch(&mut self, event: &Event ) -> bool {
                if !event.is_user() {
                    return false;
                }
                $(
                    if let Some(task) = event.as_user_event_type::<$type>() {
                        self.$func_name(task);
                        return true;
                    }
                )*
                return false;
            }

            fn register_dispatch(&self, sdl_events: &sdl2::EventSubsystem) {
                $(
                sdl_events.register_custom_event::<$type>().expect("Types already registered");
                )*
            }
        }
    }
}

pub struct AppSystem {
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
    dispatcher: SdlDispatcher,
}

impl AppSystem {
    pub fn canvas(&mut self) -> &mut Canvas<Window> {
        &mut self.canvas
    }

    pub fn dispatcher(&mut self) -> &SdlDispatcher {
        &self.dispatcher
    }

    pub fn texture_creator(&mut self) -> &TextureCreator<WindowContext> {
        &self.texture_creator
    }
}

pub trait App {
    fn start(&mut self);
    fn resized(&mut self);
    fn stop(&mut self);
}

pub trait DispatchHandler {
    fn register_dispatch(&self, sdl_events: &sdl2::EventSubsystem);
    fn handle_dispatch(&mut self, message: &Event) -> bool;
}

pub struct AppRunner {
    sdl_context: Sdl,
    video_subsystem: VideoSubsystem,
    event_subsystem: EventSubsystem,
    event_pump: EventPump,
    app_system: AppSystem,
    with_tokio: bool,
    with_dispatch: bool,
    passive: bool,
}

/// Defines flags to notifiy interesting things discovered while
/// pumping events
#[derive(Default)]
struct PostPumpState {
    pub resized: bool,
    pub quit: bool,
}

impl AppRunner {
    /// Runs an app inside an event loop.
    pub fn run<'a, T>(&'a mut self) 
    where T: App + DispatchHandler + From<&'a mut AppSystem> { 
        let mut event_pump = self.sdl_context.event_pump().unwrap();

        if self.with_tokio {
            // If you want tokio, initialize it in your main!
        };

        let mut app = T::from(&mut self.app_system);

        if self.with_dispatch {
            app.register_dispatch(&self.event_subsystem);
        }

        app.start();

        'running: loop {
            let mut postpumpstate = PostPumpState{..Default::default()};

            let mut iterator = if self.passive {
                Some(iter::once(event_pump.wait_event()))
            } else {
                None
            }.into_iter()
                .flatten()
                .chain(event_pump.poll_iter());

            for event in iterator {                
                if event.is_user_event() && app.handle_dispatch(&event) {
                    return;
                }

                match event {
                    Event::Quit {..} |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        postpumpstate.quit = true;
                    },
                    Event::Window {
                        win_event: WindowEvent::Resized(..) | WindowEvent::SizeChanged(..),
                        ..
                    } => {
                        postpumpstate.resized = true;
                    }
                    _ => {}
                }
            }

            if postpumpstate.quit {
                break 'running;
            }

            if postpumpstate.resized {
                app.resized();
            }
        }

        app.stop();
    }
}

pub struct AppBuilder {
    title: String,
    window_size: (u32, u32),
    with_tokio: bool,
    with_dispatch: bool,
    with_egui: bool,
    passive: bool,
}

impl AppBuilder {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            window_size: (800, 600),
            with_tokio: false,
            with_dispatch: false,
            with_egui: false,
            passive: false,
        }
    }

    pub fn window_size(&mut self, w: u32, h: u32) -> &mut Self {
        self.window_size = (w, h);
        self
    }
    
    pub fn with_tokio(&mut self) -> &mut Self {
        self.with_tokio = true;
        self
    }

    pub fn with_dispatch(&mut self) -> &mut Self {
        self.with_dispatch = true;
        self
    }

    pub fn passive_event_loop(&mut self) -> &mut Self {
        self.passive = true;
        self
    }

    pub fn with_egui(&mut self) -> &mut Self {
        self.with_egui = true;
        self
    }

    pub fn build(&self) -> AppRunner {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let event_subsystem = sdl_context.event().unwrap();

        let window = video_subsystem.window(&self.title, self.window_size.0, self.window_size.1)
            .resizable()
            .opengl()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        let texture_creator = canvas.texture_creator();
        let event_pump = sdl_context.event_pump().unwrap();

        let dispatcher = SdlDispatcher::from_eventsubsystem(&event_subsystem);

        AppRunner {
            sdl_context,
            video_subsystem,
            event_subsystem,
            event_pump,
            app_system: AppSystem{
                canvas,
                texture_creator,
                dispatcher
            },
            with_tokio: self.with_tokio,
            with_dispatch: self.with_dispatch,
            passive: self.passive
        }
    }
}
