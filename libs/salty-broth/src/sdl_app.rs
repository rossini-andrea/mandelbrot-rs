use sdl2::{
    Sdl,
    VideoSubsystem,
    EventSubsystem,
    EventPump,
    pixels::{Color, PixelFormatEnum},
    event::{Event, WindowEvent},
    keyboard::Keycode,
    render::Canvas,
    video::Window,
};
use sdl_dispatch::SdlDispatcher;
use std::{
    iter,
    sync::Arc,
};
use tokio::runtime::Runtime;

macro_rules! dispatch_handlers {
    (
        $app,
        $(
            fn $func_name:ident($self, $essage: $type) {
                $impl:block
            }
        )
        
        *
    ) => {
        impl $app {
            fn $func_name($self, $message: type>) {
                $impl:block
            }

            fn handle_dispatch(&self, event: &Event ) -> bool {
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

macro_rules! dispatch_message {
    ($app, $name, $(par),*) => {
        // Get the dispatcher somehow
        sdl_app::ui_dispatcher().spawn::<concat_idents!($app, _, DispatchMessages), ()>(concat_idents!($app, _, DispatchMessages)::$Name)
    }
}

pub trait App {
    fn window_info(&self) -> (String, u32, u32);
    fn resized(&self);
    fn canvas_ready(&self, canvas: &Canvas<Window>);
    fn register_dispatch(&self, sdl_events: &sdl2::EventSubsystem);
    fn handle_dispatch(&self, message: &Event) -> bool;
}

static mut ui_dispatcher: Option<&'static SdlDispatcher> = None;

pub fn dispatcher() -> &'static SdlDispatcher {
    unsafe {
        ui_dispatcher.expect("Trying to use the dispatcher without a running app.")
    }
}

struct AppRunner {
    sdl_context: Sdl,
    video_subsystem: VideoSubsystem,
    event_subsystem: EventSubsystem,
    canvas: Canvas<Window>,
    event_pump: EventPump,
    dispatcher: SdlDispatcher,
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
    pub fn run<T>(&self, app: T) 
    where T: App {
        let mut event_pump = self.sdl_context.event_pump().unwrap();

        if self.with_tokio {
            // If you want tokio, initialize it in your main!
        };

        if self.with_dispatch {
            app.register_dispatch(&self.event_subsystem);
        }

        app.canvas_ready(&self.canvas);

        let handle_event = |event: Event, state: &mut PostPumpState| {
            if event.is_user_event() && app.handle_dispatch(&event) {
                return;
            }
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    state.quit = true;
                },
                Event::Window {
                    win_event: WindowEvent::Resized(..) | WindowEvent::SizeChanged(..),
                    ..
                } => {
                    state.resized = true;
                }
                _ => {}
            }
        };

        if self.passive {
            'running: loop {
                let mut postpumpstate = PostPumpState{..Default::default()};

                for event in iter::once(
                    event_pump.wait_event()
                ).chain(
                    event_pump.poll_iter()
                ) {
                    handle_event(event, &mut postpumpstate);
                }

                if postpumpstate.quit {
                    break 'running;
                }

                if postpumpstate.resized {
                    app.resized();
                }
            }
        } else {
            'running: loop {
                let mut postpumpstate = PostPumpState{..Default::default()};

                for event in event_pump.poll_iter() {
                    handle_event(event, &mut postpumpstate);
                }

                if postpumpstate.quit {
                    break 'running;
                }

    
                if postpumpstate.resized {
                    app.resized();
                }
            }
        }
    }

}

struct AppBuilder {
    title: String,
    window_size: (u32, u32),
    with_tokio: bool,
    with_dispatch: bool,
    with_egui: bool,
    passive: bool,
}

impl AppBuilder {
    fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            window_size: (800, 600),
            with_tokio: false,
            with_dispatch: false,
            with_egui: false,
            passive: false,
        }
    }

    fn window_size(&mut self, w: u32, h: u32) -> &mut Self {
        self.window_size = (w, h);
        self
    }
    
    fn with_tokio(&mut self) -> &mut Self {
        self.with_tokio = true;
        self
    }

    fn with_dispatch(&mut self) -> &mut Self {
        self.with_dispatch = true;
        self
    }

    fn passive_event_loop(&mut self) -> &mut Self {
        self.passive = true;
        self
    }

    fn with_egui(&mut self) -> &mut Self {
        self.with_egui = true;
        self
    }

    fn build(&self) -> AppRunner {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let event_subsystem = sdl_context.event().unwrap();

        let window = video_subsystem.window(&self.title, self.window_size.0, self.window_size.1)
            .resizable()
            .opengl()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();

        let dispatcher = SdlDispatcher::from_eventsubsystem(&event_subsystem);

        unsafe {
            ui_dispatcher = Some(&dispatcher);
        }

        AppRunner {
            sdl_context,
            video_subsystem,
            event_subsystem,
            canvas,
            event_pump,
            dispatcher,
            with_tokio: self.with_tokio,
            with_dispatch: self.with_dispatch,
            passive: self.passive
        }
    }
}
