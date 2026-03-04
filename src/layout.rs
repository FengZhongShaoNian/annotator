use crate::annotator::AnnotatorState;
use crate::annotator_panel::create_annotator_panel;
use crate::application::Application;
use crate::context::Command;
use crate::dpi::{LogicalPosition, LogicalSize};
use crate::primary_toolbar::create_primary_toolbar;
use crate::secondly_toolbar::create_secondly_toolbar;
use crate::view::View;
use crate::window::AppWindow;
use egui::{Context, FullOutput, RawInput};
use image::RgbaImage;
use sctk::shell::WaylandSurface;
use std::cmp::max;
use std::sync::Arc;

pub fn build_annotator(
    input: RawInput,
    egui_ctx: &mut Context,
    app: &mut Application,
    window: &mut AppWindow,
    image: Arc<RgbaImage>,
    current_view: &mut dyn View,
) -> FullOutput {
    let annotator_panel_id = AnnotatorState::annotator_panel_id();
    let primary_toolbar_id = AnnotatorState::primary_toolbar_id();
    let secondly_toolbar_id = AnnotatorState::secondly_toolbar_id();
    if !window.views.contains_key(&annotator_panel_id) {
        create_annotator_panel(annotator_panel_id.clone(), app, window, image);

        let annotator_panel = window
            .views
            .get(&annotator_panel_id)
            .unwrap()
            .as_ref()
            .unwrap();
        let annotator_panel_position = annotator_panel.get_view_ref().position();
        let annotator_panel_size = annotator_panel.get_view_ref().size();
        let annotator_panel_required_size =
            calculate_required_size(annotator_panel_position, annotator_panel_size);

        let primary_toolbar_size = LogicalSize::new(600, 32);
        let primary_toolbar_margin_top = 12;

        let primary_toolbar_position: LogicalPosition<i32> = LogicalPosition::new(
            (annotator_panel_required_size.width - primary_toolbar_size.width) as i32,
            (annotator_panel_required_size.height + primary_toolbar_margin_top) as i32,
        );

        let secondly_toolbar_margin_top = 6;
        let secondly_toolbar_size = LogicalSize::new(600, 32);

        let secondly_toolbar_position: LogicalPosition<i32> = LogicalPosition::new(
            (annotator_panel_required_size.width - secondly_toolbar_size.width) as i32,
            (primary_toolbar_position.y + primary_toolbar_size.height as i32 + secondly_toolbar_margin_top),
        );

        create_primary_toolbar(
            primary_toolbar_id.clone(),
            app,
            window,
            primary_toolbar_size,
            primary_toolbar_position,
        );
        create_secondly_toolbar(
            secondly_toolbar_id.clone(),
            app,
            window,
            secondly_toolbar_size,
            secondly_toolbar_position,
        );
        let secondly_toolbar_size = LogicalSize::new(600, 32);
    }

    let annotator_panel = window
        .views
        .get(&annotator_panel_id)
        .unwrap()
        .as_ref()
        .unwrap();
    let primary_toolbar = window
        .views
        .get(&primary_toolbar_id)
        .unwrap()
        .as_ref()
        .unwrap();
    let secondly_toolbar = window
        .views
        .get(&secondly_toolbar_id)
        .unwrap()
        .as_ref()
        .unwrap();

    let annotator_panel_position = annotator_panel.get_view_ref().position();
    let annotator_panel_size = annotator_panel.get_view_ref().size();

    let primary_toolbar_position = primary_toolbar.get_view_ref().position();
    let primary_toolbar_size = primary_toolbar.get_view_ref().size();

    let secondly_toolbar_position = secondly_toolbar.get_view_ref().position();
    let secondly_toolbar_size = secondly_toolbar.get_view_ref().size();

    let required_window_size = calculate_required_window_size(
        annotator_panel_position,
        annotator_panel_size,
        primary_toolbar_position,
        primary_toolbar_size,
        secondly_toolbar_position,
        secondly_toolbar_size,
    );

    let main_window_size = current_view.size();
    if main_window_size != required_window_size {
        window
            .window_context
            .commands
            .push_back(Command::ResizeView(current_view.id(), required_window_size));
        
        // 鼠标穿透
        let qh = &app.global_state.queue_handle;
        let empty_region = app.global_state.compositor_state.wl_compositor().create_region(qh, ());
        window.xdg_window().set_input_region(Some(&empty_region));
    }
    egui_ctx.run(input, move |_ctx| {})
}

/// 计算容纳annotator_panel、primary_toolbar、secondly_toolbar需要多大的窗口空间
fn calculate_required_window_size(
    annotator_panel_position: Option<LogicalPosition<i32>>,
    annotator_panel_size: LogicalSize<u32>,
    primary_toolbar_position: Option<LogicalPosition<i32>>,
    primary_toolbar_size: LogicalSize<u32>,
    secondly_toolbar_position: Option<LogicalPosition<i32>>,
    secondly_toolbar_size: LogicalSize<u32>,
) -> LogicalSize<u32> {
    let annotator_panel_required_size =
        calculate_required_size(annotator_panel_position, annotator_panel_size);
    let primary_toolbar_required_size =
        calculate_required_size(primary_toolbar_position, primary_toolbar_size);
    let secondly_toolbar_required_size =
        calculate_required_size(secondly_toolbar_position, secondly_toolbar_size);

    let width = max(
        annotator_panel_required_size.width,
        max(
            primary_toolbar_required_size.width,
            secondly_toolbar_required_size.width,
        ),
    );
    let height = max(
        annotator_panel_required_size.height,
        max(
            secondly_toolbar_required_size.height,
            secondly_toolbar_required_size.height,
        ),
    );

    LogicalSize::new(width, height)
}

/// 计算容纳子表面需要多大的父表面
fn calculate_required_size(
    sub_view_position: Option<LogicalPosition<i32>>,
    sub_view_size: LogicalSize<u32>,
) -> LogicalSize<u32> {
    if let (Some(pos), size) = (sub_view_position, sub_view_size) {
        let width = pos.x + (size.width as i32);
        let height = pos.y + (size.height as i32);
        LogicalSize::new(width as u32, height as u32)
    } else {
        sub_view_size
    }
}
