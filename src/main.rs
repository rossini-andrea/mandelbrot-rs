mod mandelbrot;
mod mainapp;

use color_eyre::eyre::Result;
use salty_broth::sdl_app::*;
use tokio::{runtime::Runtime};

pub fn main() -> Result<()> {
    color_eyre::install()?;

    let apprunner = AppBuilder::new("Mandelbrot Explorer")
        .window_size(800, 600)
        .with_dispatch()
        .with_tokio()
        .passive_event_loop()
        .build();

    let tokio = Runtime::new()?;
    let _guard = tokio.enter();
    apprunner.run::<mainapp::MainApp>();
 
    Ok(())
}

