use crate::annotator::ellipse::EllipseState;
use crate::annotator::rectangle::RectangleState;
use crate::annotator::svg_button::SvgButton;
use crate::annotator::{AnnotatorState, StrokeType, ToolType};
use crate::application::Application;
use crate::context::Command;
use crate::dpi::{LogicalBounds, LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use crate::global::ReadGlobalMut;
use crate::icon::Icons;
use crate::view::ViewId;
use crate::window::AppWindow;
use egui::{Color32, Frame, vec2};
use std::sync::Arc;

pub fn create_secondly_toolbar(
    view_id: ViewId,
    app: &mut Application,
    window: &mut AppWindow,
    root_view_size: LogicalSize<u32>,
) {
    let position_calculator = Arc::new(
        |parent_surface_size: &PhysicalSize<u32>, subview_size: &PhysicalSize<u32>| {
            let subview_width = &subview_size.width;
            PhysicalPosition::new(
                parent_surface_size.width - subview_width,
                parent_surface_size.height + 54,
            )
        },
    );

    let global_state = &app.global_state;

    let toolbar_size = LogicalSize::new(600, 32);

    let scale_factor = window.scale_factor().unwrap();
    let physical_position = position_calculator(
        &root_view_size.to_physical(scale_factor),
        &toolbar_size.to_physical(scale_factor),
    );
    let logical_position = physical_position.to_logical(scale_factor);

    // 创建子工具条
    window.create_sub_surface_view(
        view_id,
        global_state,
        toolbar_size,
        logical_position,
        Box::new(|input, egui_ctx, app, window, current_view| {
            // 构建 UI 的具体内容
            egui_ctx.run(input, move |ctx| {
                egui::CentralPanel::default()
                    .frame(Frame::new().fill(Color32::from_hex("#393b40").unwrap()))
                    .show(ctx, |ui| {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                        ui.spacing_mut().item_spacing = vec2(1.0, 0.0);

                        let current_view_id =
                            window.window_context.current_view_id.clone().unwrap();
                        let annotator_state = window
                            .window_context
                            .globals_by_type
                            .require_ref_mut::<AnnotatorState>();
                        let active_tool = annotator_state.current_annotation_tool;

                        if active_tool.is_none() {
                            current_view.set_visible(false);
                            return;
                        }

                        if matches!(active_tool, Some(ToolType::Rectangle)) {
                            let tool_state = &mut annotator_state.rectangle_annotation_tool_state;

                            let label = match tool_state.style.stroke_type {
                                StrokeType::SolidLine => "实线",
                                StrokeType::DashedLine => "虚线",
                                StrokeType::DottedLine => "点线",
                            };
                            let stroke_type = egui::ComboBox::from_label("");
                            stroke_type.selected_text(label).show_ui(ui, |ui| {
                                if ui.label("实线").clicked() {
                                    tool_state.style.stroke_type = StrokeType::SolidLine;
                                }
                                if ui.label("虚线").clicked() {
                                    tool_state.style.stroke_type = StrokeType::DashedLine;
                                }
                                if ui.label("点线").clicked() {
                                    tool_state.style.stroke_type = StrokeType::DottedLine;
                                }
                            });
                        }
                    });
            })
        }),
        Some(position_calculator),
    );
}
