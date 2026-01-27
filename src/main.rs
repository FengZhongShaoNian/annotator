mod application;
mod window;
mod egui_input;
mod egui_skia_painter;
mod gpu;

mod wp_fractional_scaling;

mod wp_viewporter;
mod surface_view;
mod sub_surface_view;
mod view;
mod dpi;
mod context;

use std::env;
use log::error;
use crate::application::Application;

fn main() {
    env_logger::init();
    
    let mut app = Application::new("site.nullable.annotator");

    for path in env::args_os().skip(1) {
        let image = match image::open(&path) {
            Ok(i) => i,
            Err(e) => {
                error!("Failed to open image {}.", path.to_string_lossy());
                error!("Error was: {e:?}");
                return;
            }
        };

        // We'll need the image in RGBA for drawing it
        let image = image.to_rgba8();
        app.open_image(image);
    }
    
    app.run();
}

