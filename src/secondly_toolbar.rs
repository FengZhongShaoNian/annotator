use std::any::TypeId;
use crate::annotator::{AnnotatorState, StrokeType, ToolType};
use crate::application::Application;
use crate::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use crate::view::ViewId;
use crate::window::AppWindow;
use egui::{vec2, Color32, Frame};
use std::sync::Arc;
use wayland_protocols::xdg::shell::client::xdg_positioner;
use crate::global::{ReadGlobal, ReadGlobalMut};
use crate::view::xdg_popup_view::TriggerType;

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
                                let positioner = window.create_positioner(&app.global_state);

                                // 弹出框的尺寸
                                positioner.set_size(100, 80);

                                // 父表面内的锚点矩形
                                positioner.set_anchor_rect(0, 0, 80, 40);
                                // 指定锚定矩形的哪一条边或角与弹出窗口对齐
                                positioner.set_anchor(xdg_positioner::Anchor::Bottom);
                                // 弹窗相对于锚点的伸展方向
                                positioner.set_gravity(xdg_positioner::Gravity::Bottom);
                                // 空间不足时的自动调整策略
                                positioner.set_constraint_adjustment(xdg_positioner::ConstraintAdjustment::all());

                                window.create_xdg_popup_view(
                                    select_stroke_type_popup_id,
                                    &app.global_state,
                                    TriggerType::MousePress,
                                    positioner,
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
