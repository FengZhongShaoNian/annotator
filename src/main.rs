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

mod view;
mod wp_viewporter;
mod primary_toolbar;
mod secondly_toolbar;
mod serial;
mod layout;
mod annotator_panel;

use crate::annotator::ellipse::{EllipseTool, EllipseState};
use crate::annotator::rectangle::{
    RectangleTool, RectangleState,
};
use crate::annotator::{Annotation, AnnotatorState, StrokeType, AnnotationTool};
use crate::application::Application;
use crate::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use crate::global::{ReadGlobalMut, ReadOrInsertGlobal};
use crate::window::WindowConfiguration;
use egui::load::SizedTexture;
use egui::{ColorImage, Frame, Image, ImageSource, Rect, pos2, vec2, Color32};
use log::error;
use std::env;
use std::sync::Arc;
use crate::layout::build_annotator;
use crate::primary_toolbar::create_primary_toolbar;
use crate::secondly_toolbar::create_secondly_toolbar;
use crate::view::ViewId;

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
