use crate::annotator::svg_button::SvgButton;
use crate::annotator::{AnnotatorState, SharedAnnotatorState, ToolName};
use crate::application::Application;
use crate::dpi::{LogicalPosition, LogicalSize};
use crate::global::ReadGlobalMut;
use crate::icon::Icons;
use crate::view::ViewId;
use crate::window::AppWindow;
use egui::{vec2, Color32, Frame};

pub fn create_primary_toolbar(
    view_id: ViewId,
    app: &mut Application,
    window: &mut AppWindow,
    toolbar_size: LogicalSize<u32>,
    toolbar_positon: LogicalPosition<i32>,
) {
    let global_state = &app.global_state;

    // 创建工具条
    window.create_sub_surface_view(
        view_id,
        global_state,
        toolbar_size,
        toolbar_positon,
        Box::new(|input, egui_ctx, app, window, current_view| {
            // 构建 UI 的具体内容
            egui_ctx.run(input, move |ctx| {
                egui::CentralPanel::default()
                    .frame(Frame::new().fill(Color32::from_hex("#393b40").unwrap()))
                    .show(ctx, |ui| {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                        ui.spacing_mut().item_spacing = vec2(1.0, 0.0);
                        ui.horizontal(|ui| {
                            let annotator_state = window
                                .window_context
                                .globals_by_type
                                .require_ref_mut::<SharedAnnotatorState>()
                                .clone();

                            let mut annotator_state_mut_ref = annotator_state.borrow_mut();
                            let active_tool = annotator_state_mut_ref
                                .current_annotation_tool
                                .as_ref()
                                .map(|tool| tool.tool_name());

                            if ui
                                .add(SvgButton::new(
                                    "rectangle-tool".into(),
                                    Icons::DrawRectangle.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    true,
                                    matches!(active_tool, Some(ToolName::Rectangle)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(ToolName::Rectangle)) {
                                    annotator_state_mut_ref.deactivate_annotation_tool();
                                } else {
                                    annotator_state_mut_ref
                                        .activate_annotation_tool(ToolName::Rectangle);
                                    window.set_view_visible(
                                        &AnnotatorState::secondly_toolbar_id(),
                                        true,
                                    );
                                }
                            }
                            if ui
                                .add(SvgButton::new(
                                    "ellipse-tool".into(),
                                    Icons::DrawEllipse.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    true,
                                    matches!(active_tool, Some(ToolName::Ellipse)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(ToolName::Ellipse)) {
                                    annotator_state_mut_ref.deactivate_annotation_tool();
                                } else {
                                    annotator_state_mut_ref
                                        .activate_annotation_tool(ToolName::Ellipse);
                                    window.set_view_visible(
                                        &AnnotatorState::secondly_toolbar_id(),
                                        true,
                                    );
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "straight-line-tool".into(),
                                    Icons::DrawLine.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    true,
                                    matches!(active_tool, Some(ToolName::StraightLine)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(ToolName::StraightLine)) {
                                    annotator_state_mut_ref.deactivate_annotation_tool();
                                } else {
                                    annotator_state_mut_ref
                                        .activate_annotation_tool(ToolName::StraightLine);
                                    window.set_view_visible(
                                        &AnnotatorState::secondly_toolbar_id(),
                                        true,
                                    );
                                }
                            }
                            if ui
                                .add(SvgButton::new(
                                    "arrow-tool".into(),
                                    Icons::DrawArrow.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    true,
                                    matches!(active_tool, Some(ToolName::Arrow)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(ToolName::Arrow)) {
                                    annotator_state_mut_ref.deactivate_annotation_tool();
                                } else {
                                    annotator_state_mut_ref
                                        .activate_annotation_tool(ToolName::Arrow);
                                    window.set_view_visible(
                                        &AnnotatorState::secondly_toolbar_id(),
                                        true,
                                    );
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "pencil-tool".into(),
                                    Icons::DrawFreehand.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    true,
                                    matches!(active_tool, Some(ToolName::Pencil)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(ToolName::Pencil)) {
                                    annotator_state_mut_ref.deactivate_annotation_tool();
                                } else {
                                    annotator_state_mut_ref
                                        .activate_annotation_tool(ToolName::Pencil);
                                    window.set_view_visible(
                                        &AnnotatorState::secondly_toolbar_id(),
                                        true,
                                    );
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "marker-pen-tool".into(),
                                    Icons::DrawHighlight.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    true,
                                    matches!(active_tool, Some(ToolName::MarkerPen)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(ToolName::MarkerPen)) {
                                    annotator_state_mut_ref.deactivate_annotation_tool();
                                } else {
                                    annotator_state_mut_ref
                                        .activate_annotation_tool(ToolName::MarkerPen);
                                    window.set_view_visible(
                                        &AnnotatorState::secondly_toolbar_id(),
                                        true,
                                    );
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "mosaic-tool".into(),
                                    Icons::PixelArtTrace.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    true,
                                    matches!(active_tool, Some(ToolName::Mosaic)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(ToolName::Mosaic)) {
                                    annotator_state_mut_ref.deactivate_annotation_tool();
                                } else {
                                    annotator_state_mut_ref
                                        .activate_annotation_tool(ToolName::Mosaic);
                                    window.set_view_visible(
                                        &AnnotatorState::secondly_toolbar_id(),
                                        false,
                                    );
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "blur-tool".into(),
                                    Icons::BlurFx.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    true,
                                    matches!(active_tool, Some(ToolName::Blur)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(ToolName::Blur)) {
                                    annotator_state_mut_ref.deactivate_annotation_tool();
                                } else {
                                    annotator_state_mut_ref
                                        .activate_annotation_tool(ToolName::Blur);
                                    window.set_view_visible(
                                        &AnnotatorState::secondly_toolbar_id(),
                                        false,
                                    );
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "text-tool".into(),
                                    Icons::DrawText.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    true,
                                    matches!(active_tool, Some(ToolName::Text)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(ToolName::Text)) {
                                    annotator_state_mut_ref.deactivate_annotation_tool();
                                } else {
                                    annotator_state_mut_ref
                                        .activate_annotation_tool(ToolName::Text);
                                    window.set_view_visible(
                                        &AnnotatorState::secondly_toolbar_id(),
                                        true,
                                    );
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "serial-number-tool".into(),
                                    Icons::DrawNumber.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    true,
                                    matches!(active_tool, Some(ToolName::SerialNumber)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(ToolName::SerialNumber)) {
                                    annotator_state_mut_ref.deactivate_annotation_tool();
                                } else {
                                    annotator_state_mut_ref
                                        .activate_annotation_tool(ToolName::SerialNumber);
                                    window.set_view_visible(
                                        &AnnotatorState::secondly_toolbar_id(),
                                        true,
                                    );
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "eraser-tool".into(),
                                    Icons::DrawEraser.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    true,
                                    matches!(active_tool, Some(ToolName::Eraser)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(ToolName::Eraser)) {
                                    annotator_state_mut_ref.deactivate_annotation_tool();
                                } else {
                                    annotator_state_mut_ref
                                        .activate_annotation_tool(ToolName::Eraser);
                                    window.set_view_visible(
                                        &AnnotatorState::secondly_toolbar_id(),
                                        false,
                                    );
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "undo-tool".into(),
                                    Icons::EditUndo.get_image(),
                                    LogicalSize::new(32., 32.),
                                    !annotator_state_mut_ref.can_undo(),
                                    false,
                                    false,
                                ))
                                .clicked()
                            {
                                annotator_state_mut_ref.undo();
                            }

                            if ui
                                .add(SvgButton::new(
                                    "redo-tool".into(),
                                    Icons::EditRedo.get_image(),
                                    LogicalSize::new(32., 32.),
                                    !annotator_state_mut_ref.can_redo(),
                                    false,
                                    false,
                                ))
                                .clicked()
                            {
                                annotator_state_mut_ref.redo();
                            }

                            if ui
                                .add(SvgButton::new(
                                    "save-tool".into(),
                                    Icons::DocumentSave.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    false,
                                    false,
                                ))
                                .clicked()
                            {}

                            if ui
                                .add(SvgButton::new(
                                    "copy-tool".into(),
                                    Icons::EditCopy.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    false,
                                    false,
                                ))
                                .clicked()
                            {}
                            if ui
                                .add(SvgButton::new(
                                    "ok-tool".into(),
                                    Icons::DialogOk.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    false,
                                    false,
                                ))
                                .clicked()
                            {
                                annotator_state_mut_ref.deactivate_annotation_tool();
                                current_view.set_visible(false);
                            }

                            let tool_name = annotator_state_mut_ref
                                .current_annotation_tool
                                .as_ref()
                                .map(|t| t.tool_name());
                            if active_tool != tool_name {
                                println!("标注工具由{:?}切换为{:?}", active_tool, tool_name);
                                // 将栈顶的标注更新为非激活状态
                                annotator_state_mut_ref.annotations_stack.last_mut()
                                    .map(|annotation|annotation.activation_mut().deactivate());
                            }
                        });
                    });
            })
        }),
        None,
    );
}
