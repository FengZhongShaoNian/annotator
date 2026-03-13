use crate::annotator::AnnotatorState;
use crate::annotator_panel::create_annotator_panel;
use crate::application::Application;
use crate::context::Command;
use crate::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use crate::primary_toolbar::create_primary_toolbar;
use crate::secondly_toolbar::create_secondly_toolbar;
use crate::view::View;
use crate::window::AppWindow;
use egui::{Context, FullOutput, RawInput};
use image::RgbaImage;
use sctk::shell::WaylandSurface;
use std::sync::Arc;
use taffy::prelude::*;
use taffy::{geometry::Rect, style::LengthPercentageAuto};

// 主窗口和工具条以及工具条之间的间隙
const GAP: u32 = 12;

// 布局说明：
// 1.annotator_panel与底层的xdg_toplevel右对齐
// 2.工具条与annotator_panel右对齐
// 3.底层的xdg_toplevel的大小需要能够容纳annotator_panel和工具条
pub fn build_annotator(
    input: RawInput,
    egui_ctx: &mut Context,
    app: &mut Application,
    window: &mut AppWindow,
    image: Arc<RgbaImage>,
    current_view: &mut dyn View,
) -> FullOutput {
    // 标注面板的ID
    let annotator_panel_id = AnnotatorState::annotator_panel_id();
    // 主工具条的ID
    let primary_toolbar_id = AnnotatorState::primary_toolbar_id();
    // 次工具的ID
    let secondly_toolbar_id = AnnotatorState::secondly_toolbar_id();

    let annotator_panel_created = window.views.contains_key(&annotator_panel_id);
    let annotator_panel_size = if annotator_panel_created {
        let annotator_panel = window
            .views
            .get(&annotator_panel_id)
            .unwrap()
            .as_ref()
            .unwrap();
        annotator_panel.get_view_ref().size()
    } else {
        let scale_factor = window.scale_factor().unwrap();
        PhysicalSize::new(image.width(), image.height()).to_logical(scale_factor)
    };
    let mut tree: TaffyTree<()> = TaffyTree::new();

    let annotator_panel_node = tree
        .new_leaf(Style {
            size: Size {
                width: length(annotator_panel_size.width as f32),
                height: length(annotator_panel_size.height as f32),
            },
            ..Default::default()
        })
        .unwrap();

    let primary_toolbar_size = LogicalSize::new(528, 32);

    let primary_toolbar_node = tree
        .new_leaf(Style {
            size: Size {
                width: length(primary_toolbar_size.width as f32),
                height: length(primary_toolbar_size.height as f32),
            },
            margin: Rect {
                left: LengthPercentageAuto::length(0.),
                right: LengthPercentageAuto::length(0.),
                top: LengthPercentageAuto::length(GAP as f32),
                bottom: LengthPercentageAuto::length(0.),
            },
            ..Default::default()
        })
        .unwrap();

    let secondary_toolbar_created = window.views.contains_key(&secondly_toolbar_id);
    let secondly_toolbar_size = if secondary_toolbar_created {
        let secondary_toolbar = window
            .views
            .get(&secondly_toolbar_id)
            .unwrap()
            .as_ref()
            .unwrap();
        secondary_toolbar.get_view_ref().size()
    } else {
        LogicalSize::new(503, 32)
    };

    let secondly_toolbar_node = tree
        .new_leaf(Style {
            size: Size {
                width: length(secondly_toolbar_size.width as f32),
                height: length(secondly_toolbar_size.height as f32),
            },
            margin: Rect {
                left: LengthPercentageAuto::length(0.),
                right: LengthPercentageAuto::length(0.),
                top: LengthPercentageAuto::length(GAP as f32),
                bottom: LengthPercentageAuto::length(0.),
            },
            ..Default::default()
        })
        .unwrap();

    let required_min_width = annotator_panel_size
        .width
        .max(primary_toolbar_size.width)
        .max(secondly_toolbar_size.width);

    let required_min_height = annotator_panel_size.height
        + primary_toolbar_size.height
        + secondly_toolbar_size.height
        + GAP * 2;

    let root_node = tree
        .new_with_children(
            Style {
                flex_direction: FlexDirection::Column,
                flex_wrap: FlexWrap::NoWrap,
                justify_content: Some(JustifyContent::FlexEnd),
                align_items: Some(AlignItems::End),
                size: Size {
                    width: length(required_min_width as f32),
                    height: length(required_min_height as f32),
                },
                ..Default::default()
            },
            &[
                annotator_panel_node,
                primary_toolbar_node,
                secondly_toolbar_node,
            ],
        )
        .unwrap();

    tree.compute_layout(root_node, Size::MAX_CONTENT).unwrap();
    let root_node_layout_result = tree.layout(root_node).unwrap();
    let annotator_panel_node_result = tree.layout(annotator_panel_node).unwrap();
    let primary_toolbar_node_layout_result = tree.layout(primary_toolbar_node).unwrap();
    let secondly_toolbar_node_layout_result = tree.layout(secondly_toolbar_node).unwrap();

    if !window.views.contains_key(&annotator_panel_id) {
        let annotator_panel_position = LogicalPosition::new(
            annotator_panel_node_result.location.x.round() as i32,
            annotator_panel_node_result.location.y.round() as i32,
        );
        create_annotator_panel(
            annotator_panel_id.clone(),
            app,
            window,
            image,
            annotator_panel_position,
        );

        let primary_toolbar_position = LogicalPosition::new(
            primary_toolbar_node_layout_result.location.x.round() as i32,
            primary_toolbar_node_layout_result.location.y.round() as i32,
        );
        create_primary_toolbar(
            primary_toolbar_id.clone(),
            app,
            window,
            primary_toolbar_size,
            primary_toolbar_position,
        );

        let secondly_toolbar_position = LogicalPosition::new(
            secondly_toolbar_node_layout_result.location.x.round() as i32,
            secondly_toolbar_node_layout_result.location.y.round() as i32,
        );
        create_secondly_toolbar(
            secondly_toolbar_id.clone(),
            app,
            window,
            secondly_toolbar_size,
            secondly_toolbar_position,
        );
    } else {
        // 主工具条的位置基本不会变，所以只调整次工具条的位置
        let secondly_toolbar_position = LogicalPosition::new(
            secondly_toolbar_node_layout_result.location.x.round() as i32,
            secondly_toolbar_node_layout_result.location.y.round() as i32,
        );
        let secondly_toolbar = window
            .views
            .get(&secondly_toolbar_id)
            .unwrap()
            .as_ref()
            .unwrap();
        let secondly_toolbar_current_position = secondly_toolbar.get_view_ref().position().unwrap();
        if secondly_toolbar_current_position != secondly_toolbar_position {
            window
                .window_context
                .commands
                .push_back(Command::RepositionSubView(
                    secondly_toolbar_id,
                    secondly_toolbar_position,
                ));
        }
    }

    let main_window_size = LogicalSize::new(
        root_node_layout_result.size.width.round() as u32,
        root_node_layout_result.size.height.round() as u32,
    );
    let current_main_window_size = current_view.size();
    if current_main_window_size != main_window_size {
        window
            .window_context
            .commands
            .push_back(Command::ResizeView(current_view.id(), main_window_size));

        // 鼠标穿透
        let qh = &app.global_state.queue_handle;
        let empty_region = app
            .global_state
            .compositor_state
            .wl_compositor()
            .create_region(qh, ());
        window.xdg_window().set_input_region(Some(&empty_region));
    }

    egui_ctx.run(input, move |_ctx| {})
}
