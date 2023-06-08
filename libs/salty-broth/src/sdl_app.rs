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
            message $func_name:ident($($par_name: $par_type),*) {
                $impl:block
            }
        )
        ,
        *
    ) => {
        #[derive(Clone)]
        enum concat_idents!($app, _, DispatchMessages) {
            $(
                $func_name(struct { $($par_name: $par_type),* }),
            )*
        }

        impl $app {
            fn handle_dispatch(&self, message: concat_idents!($app, _, DispatchMessages)) {
                match message {
                    $(
                        $func_name({$($par_name),*}) => { $impl },
                    )*
                }
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
    type DispatchMessages;

    fn window_info(&self) -> (String, u32, u32);
    fn resized(&self);
    fn canvas_ready(&self, canvas: Canvas<Window>);
    fn handle_dispatch(&self, message: &Self::DispatchMessages);
}

static mut ui_dispatcher: Option<Arc<SdlDispatcher>> = None;

pub fn dispatcher() -> Arc<SdlDispatcher> {
    ui_dispatcher.expect("Trying to use the dispatcher without a running app.").clone()
}

pub fn run<T>(app: T) 
where T: App, <T as App>::DispatchMessages: Send + Clone {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let event_subsystem = sdl_context.event().unwrap();

    let (title, w, h) = app.window_info();
    let window = video_subsystem.window(&title, w, h)
        .resizable()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let (ui_executor, disp) = sdl_dispatch::new_executor_and_dispatcher::<T::DispatchMessages, ()>(&event_subsystem);
    let mut event_pump = sdl_context.event_pump().unwrap();

    ui_dispatcher = Some(disp);

    let tokio_runtime = Runtime::new().unwrap();
    let _guard = tokio_runtime.enter();

    app.canvas_ready(canvas);

    'running: loop {
        let mut resized = false;

        for event in iter::once(
            event_pump.wait_event()
        ).chain(
            event_pump.poll_iter()
        ) {
            if let Some(task) = ui_executor.handle_sdl_event::<T::DispatchMessages, ()>(&event) {
                app.handle_dispatch(task.input());
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
            app.resized();
        }
    }
}
