use std::any::TypeId;
use crate::annotator::{AnnotatorState, StrokeType, ToolType};
use crate::application::Application;
use crate::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use crate::view::ViewId;
use crate::window::AppWindow;
use egui::{vec2, Color32, Frame};
use std::sync::Arc;
use crate::global::{ReadGlobal, ReadGlobalMut};

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

                        let mut annotator_state = window
                            .window_context
                            .globals_by_type
                            .take::<AnnotatorState>()
                            .unwrap();
                        let active_tool = annotator_state.current_annotation_tool;

                        if active_tool.is_none() {
                            current_view.set_visible(false);
                            window.window_context.globals_by_type.insert(TypeId::of::<AnnotatorState>(), annotator_state);
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
                            let pointer_pos = ui.ctx().input(|input| {
                                if input.pointer.button_clicked(egui::PointerButton::Primary) {
                                    return input.pointer.interact_pos();
                                }
                                None
                            });
                            stroke_type.selected_text(label).show_ui(ui, move |ui| {
                            });
                            let select_stroke_type_popup_id :ViewId = "select-stoke-type-popup".into();

                            if let Some(pointer_pos) = pointer_pos && !window.views.contains_key(&select_stroke_type_popup_id){
                                // let mut popup_position = LogicalPosition::new(pointer_pos.x as u32, pointer_pos.y as u32);
                                let mut popup_position = LogicalPosition::new(0 as u32, 0 as u32);
                                popup_position.x = 1364-600;
                                popup_position.y = 718-100;
                                window.create_xdg_popup_view(
                                    select_stroke_type_popup_id,
                                    &app.global_state,
                                    LogicalSize::new(200, 68),
                                    popup_position,
                                    false,
                                    Box::new(|input, egui_ctx, app, window, current_view| {
                                        // 构建 UI 的具体内容
                                        egui_ctx.run(input, move |ctx| {
                                            egui::CentralPanel::default()
                                                .frame(Frame::new().fill(Color32::from_hex("#393b40").unwrap()))
                                                .show(ctx, |ui| {
                                                    ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                                                    ui.spacing_mut().item_spacing = vec2(1.0, 0.0);

                                                    let annotator_state = window
                                                        .window_context
                                                        .globals_by_type
                                                        .require_ref_mut::<AnnotatorState>();
                                                    let active_tool = annotator_state.current_annotation_tool;
                                                    let tool_state = &mut annotator_state.rectangle_annotation_tool_state;
                                                    if ui.button("——————").clicked() {
                                                        tool_state.style.stroke_type = StrokeType::SolidLine;
                                                        current_view.close();
                                                    }
                                                    if ui.button("______").clicked() {
                                                        tool_state.style.stroke_type = StrokeType::DashedLine;
                                                        current_view.close();
                                                    }
                                                    if ui.button("......").clicked() {
                                                        tool_state.style.stroke_type = StrokeType::DottedLine;
                                                        current_view.close();
                                                    }
                                                });
                                        })
                                    }),
                                );
                            }

                        }

                        window.window_context.globals_by_type.insert(TypeId::of::<AnnotatorState>(), annotator_state);
                    });
            })
        }),
        Some(position_calculator),
    );
}
