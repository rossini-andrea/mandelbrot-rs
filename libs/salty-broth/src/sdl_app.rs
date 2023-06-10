use sdl2::{
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
    fn canvas_ready(&self, canvas: Canvas<Window>);
    fn handle_dispatch(&self, message: &Event);
}

static mut ui_dispatcher: Option<Arc<SdlDispatcher>> = None;

pub fn dispatcher() -> Arc<SdlDispatcher> {
    ui_dispatcher.expect("Trying to use the dispatcher without a running app.").clone()
}

struct AppRunner {
    sdl_context: SdlContext,
    video_subsystem: VideoSubsystem,
    event_subsystem: EventSubsystem,
    window: Window,
    canvas: Canvas,
    event_pump: EventPump,
    self.with_tokio: bool,
    self.with_dispatch: bool,
    self.passive: bool,
}

impl AppRunner {
    /// Defines flags to notifiy interesting things discovered while
    /// pumping events
    struct PostPumpState {
        pub resized: bool,
    };

    /// Runs an app inside an event loop.
    pub fn run<T>(&self, app: T) 
    where T: App {
        let mut event_pump = sdl_context.event_pump().unwrap();

        let _tokio = if self.with_tokio {
            let runtime = Runtime::new().unwrap();
            Some(runtime, runtime.enter())
        } else {
            None
        };

        if self.with_dispatch {
            app.register_dispatch(&self.sdl_events);
        }

        app.canvas_ready(self.canvas);

        if self.passive {
            'running: loop {
                let mut postpumpstate = PostPumpState{};

                for event in iter::once(
                    event_pump.wait_event()
                ).chain(
                    event_pump.poll_iter()
                ) {
                    self.handle_event(event, &mut postpumpstate);
                }

                if postpumpstate.resized {
                    app.resized();
                }
            }
        } else {
            'running: loop {
                let mut postpumpstate = PostPumpState{};

                for event in event_pump.poll_iter() {
                    self.handle_event(event, &mut postpumpstate);
                }
     
                if postpumpstate.resized {
                    app.resized();
                }
            }
        }
    }

    fn handle_event(&self, event: Event, &mut state: PostPumpState) {
        if event.is_user_event() && app.handle_dispatch(&event) {
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
                state.resized = true;
            }
            _ => {}
        }
    }
}

struct AppBuilder {
    title: str,
    window_size: (u32, u32),
    with_tokio: bool,
    with_dispatch: bool,
    passive: bool,
}

impl AppBuilder {
    fn new(title: str) -> &mut Self {
        Self {
            title: title,
            window_size: (800, 600),
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

        let window = video_subsystem.window(&self.title, self.size.0, self.size.1)
            .resizable()
            .opengl()
            .build()
            .unwrap();
        let mut canvas = window.into_canvas().build().unwrap();
        let mut event_pump = sdl_context.event_pump().unwrap();

        if self.with_dispatch {
            ui_dispatcher = Some(SdlDispatcher::from_event_subsystem()(&event_subsystem));
        }

        AppRunner {
            sdl_context,
            video_subsystem,
            event_subsystem,
            window,
            canvas,
            event_pump,
            self.with_tokio,
            self.with_dispatch,
            self.passive
        }
    }
}
