#![feature(oneshot_channel)]

mod application;
mod egui_input;
mod gpu;
mod window;

mod wp_fractional_scaling;

mod annotator;
mod context;
mod dpi;
mod font;
mod global;
mod icon;

mod annotator_panel;
mod egui_off_screen_render;
mod layout;
mod primary_toolbar;
mod secondly_toolbar;
mod serial;
mod view;
mod wp_viewporter;

use crate::application::Application;
use crate::dpi::LogicalSize;
use crate::layout::build_annotator;
use crate::window::WindowConfiguration;
use log::error;
use std::env;
use std::rc::Rc;
use std::sync::Arc;

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

        let image = Arc::new(image.to_rgba8());
        let window_config = WindowConfiguration::new(
            app.app_id.to_string(),
            "".to_string(),
            LogicalSize::new(800, 600),
            None,
        );
        app.open_window(
            window_config,
            Box::new(move |input, egui_ctx, app, window, current_view| {
                build_annotator(input, egui_ctx, app, window, image.clone(), current_view)
            }),
        );
    }

    app.run();
}
