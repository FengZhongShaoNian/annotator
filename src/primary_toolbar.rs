use crate::annotator::ellipse::EllipseState;
use crate::annotator::rectangle::RectangleState;
use crate::annotator::svg_button::SvgButton;
use crate::annotator::{AnnotatorState, AnnotationTool, SharedAnnotatorState};
use crate::application::Application;
use crate::dpi::{LogicalPosition, LogicalSize};
use crate::global::ReadGlobal;
use crate::icon::Icons;
use crate::view::ViewId;
use crate::window::AppWindow;
use egui::{vec2, Color32, Frame};
use std::any::TypeId;
use std::cell::RefCell;
use std::rc::Rc;

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
                                .take::<SharedAnnotatorState>()
                                .unwrap();

                            let mut annotator_state_mut_ref = annotator_state.borrow_mut();
                            let active_tool = &mut annotator_state_mut_ref.current_annotation_tool;
                            
                            if ui.add(SvgButton::new(
                                "rectangle-tool".into(),
                                Icons::DrawRectangle.get_image(),
                                LogicalSize::new(32., 32.),
                                true,
                                matches!(active_tool, Some(AnnotationTool::Rectangle(..))),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(AnnotationTool::Rectangle(..))) {
                                    // annotator_state_mut_ref.current_annotation_tool = None;
                                } else {
                                    // annotator_state.current_annotation_tool =
                                    //     Some(AnnotationTool::Rectangle);
                                    window.set_view_visible(&AnnotatorState::secondly_toolbar_id(), true);
                                }
                            }
                            if ui
                                .add(SvgButton::new(
                                    "ellipse-tool".into(),
                                    Icons::DrawEllipse.get_image(),
                                    LogicalSize::new(32., 32.),
                                    true,
                                    matches!(active_tool, Some(AnnotationTool::Ellipse(..))),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(AnnotationTool::Ellipse(..))) {
                                    // annotator_state_mut_ref.current_annotation_tool = None;
                                } else {
                                    // annotator_state.current_annotation_tool =
                                    //     Some(AnnotationTool::Ellipse);
                                    window.set_view_visible(&AnnotatorState::secondly_toolbar_id(), true);
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "straight-line-tool".into(),
                                    Icons::DrawLine.get_image(),
                                    LogicalSize::new(32., 32.),
                                    true,
                                    matches!(active_tool, Some(AnnotationTool::StraightLine)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(AnnotationTool::StraightLine)) {
                                    // annotator_state_mut_ref.current_annotation_tool = None;
                                } else {
                                    // annotator_state.current_annotation_tool =
                                    //     Some(AnnotationTool::StraightLine);
                                    window.set_view_visible(&AnnotatorState::secondly_toolbar_id(), true);
                                }
                            }
                            if ui
                                .add(SvgButton::new(
                                    "arrow-tool".into(),
                                    Icons::DrawArrow.get_image(),
                                    LogicalSize::new(32., 32.),
                                    true,
                                    matches!(active_tool, Some(AnnotationTool::Arrow)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(AnnotationTool::Arrow)) {
                                    // annotator_state_mut_ref.current_annotation_tool = None;
                                } else {
                                    // annotator_state.current_annotation_tool = Some(AnnotationTool::Arrow);
                                    window.set_view_visible(&AnnotatorState::secondly_toolbar_id(), true);
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "pencil-tool".into(),
                                    Icons::DrawFreehand.get_image(),
                                    LogicalSize::new(32., 32.),
                                    true,
                                    matches!(active_tool, Some(AnnotationTool::Pencil)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(AnnotationTool::Pencil)) {
                                    // annotator_state_mut_ref.current_annotation_tool = None;
                                } else {
                                    // annotator_state.current_annotation_tool =
                                    //     Some(AnnotationTool::Pencil);
                                    window.set_view_visible(&AnnotatorState::secondly_toolbar_id(), true);
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "marker-pen-tool".into(),
                                    Icons::DrawHighlight.get_image(),
                                    LogicalSize::new(32., 32.),
                                    true,
                                    matches!(active_tool, Some(AnnotationTool::MarkerPen)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(AnnotationTool::MarkerPen)) {
                                    // annotator_state_mut_ref.current_annotation_tool = None;
                                } else {
                                    // annotator_state.current_annotation_tool =
                                    //     Some(AnnotationTool::MarkerPen);
                                    window.set_view_visible(&AnnotatorState::secondly_toolbar_id(), true);
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "mosaic-tool".into(),
                                    Icons::PixelArtTrace.get_image(),
                                    LogicalSize::new(32., 32.),
                                    true,
                                    matches!(active_tool, Some(AnnotationTool::Mosaic)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(AnnotationTool::Mosaic)) {
                                    // annotator_state_mut_ref.current_annotation_tool = None;
                                } else {
                                    // annotator_state.current_annotation_tool =
                                    //     Some(AnnotationTool::Mosaic);
                                    window.set_view_visible(&AnnotatorState::secondly_toolbar_id(), true);
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "blur-tool".into(),
                                    Icons::BlurFx.get_image(),
                                    LogicalSize::new(32., 32.),
                                    true,
                                    matches!(active_tool, Some(AnnotationTool::Blur)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(AnnotationTool::Blur)) {
                                    // annotator_state_mut_ref.current_annotation_tool = None;
                                } else {
                                    // annotator_state.current_annotation_tool = Some(AnnotationTool::Blur);
                                    window.set_view_visible(&AnnotatorState::secondly_toolbar_id(), true);
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "text-tool".into(),
                                    Icons::DrawText.get_image(),
                                    LogicalSize::new(32., 32.),
                                    true,
                                    matches!(active_tool, Some(AnnotationTool::Text)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(AnnotationTool::Text)) {
                                    // annotator_state_mut_ref.current_annotation_tool = None;
                                } else {
                                    // annotator_state.current_annotation_tool = Some(AnnotationTool::Text);
                                    window.set_view_visible(&AnnotatorState::secondly_toolbar_id(), true);
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "serial-number-tool".into(),
                                    Icons::DrawNumber.get_image(),
                                    LogicalSize::new(32., 32.),
                                    true,
                                    matches!(active_tool, Some(AnnotationTool::SerialNumber)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(AnnotationTool::SerialNumber)) {
                                    // annotator_state_mut_ref.current_annotation_tool = None;
                                } else {
                                    // annotator_state.current_annotation_tool =
                                    //     Some(AnnotationTool::SerialNumber);
                                    window.set_view_visible(&AnnotatorState::secondly_toolbar_id(), true);
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "eraser-tool".into(),
                                    Icons::DrawEraser.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    matches!(active_tool, Some(AnnotationTool::Eraser)),
                                ))
                                .clicked()
                            {
                                if matches!(active_tool, Some(AnnotationTool::Eraser)) {
                                    // annotator_state_mut_ref.current_annotation_tool = None;
                                } else {
                                    // annotator_state.current_annotation_tool =
                                    //     Some(AnnotationTool::Eraser);
                                    window.set_view_visible(&AnnotatorState::secondly_toolbar_id(), true);
                                }
                            }

                            if ui
                                .add(SvgButton::new(
                                    "undo-tool".into(),
                                    Icons::EditUndo.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    false,
                                ))
                                .clicked()
                            {}

                            if ui
                                .add(SvgButton::new(
                                    "redo-tool".into(),
                                    Icons::EditRedo.get_image(),
                                    LogicalSize::new(32., 32.),
                                    false,
                                    false,
                                ))
                                .clicked()
                            {}

                            if ui
                                .add(SvgButton::new(
                                    "save-tool".into(),
                                    Icons::DocumentSave.get_image(),
                                    LogicalSize::new(32., 32.),
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
                                ))
                                .clicked()
                            {
                                annotator_state_mut_ref.current_annotation_tool = None;
                                current_view.set_visible(false);
                            }

                            // if matches!(active_tool, ) != annotator_state.current_annotation_tool {
                            //     println!(
                            //         "标注工具由{:?}切换为{:?}",
                            //         active_tool, annotator_state.current_annotation_tool
                            //     );
                            //
                            //     if let Some(tool) = active_tool {
                            //         match tool {
                            //             AnnotationTool::Rectangle => {
                            //                 let rectangle_state = annotator_state
                            //                     .annotations_stack
                            //                     .last_mut()
                            //                     .map(|annotation| {
                            //                         annotation.downcast_mut::<RectangleState>()
                            //                     })
                            //                     .flatten();
                            //                 if let Some(rectangle_state) = rectangle_state {
                            //                     rectangle_state.deactivate();
                            //                 }
                            //             }
                            //             AnnotationTool::Ellipse => {
                            //                 let ellipse_state = annotator_state
                            //                     .annotations_stack
                            //                     .last_mut()
                            //                     .map(|annotation| {
                            //                         annotation.downcast_mut::<EllipseState>()
                            //                     })
                            //                     .flatten();
                            //                 if let Some(ellipse_state) = ellipse_state {
                            //                     ellipse_state.deactivate();
                            //                 }
                            //             }
                            //             AnnotationTool::StraightLine => {}
                            //             AnnotationTool::Arrow => {}
                            //             AnnotationTool::Pencil => {}
                            //             AnnotationTool::MarkerPen => {}
                            //             AnnotationTool::Mosaic => {}
                            //             AnnotationTool::Blur => {}
                            //             AnnotationTool::Text => {}
                            //             AnnotationTool::SerialNumber => {}
                            //             AnnotationTool::Watermark => {}
                            //             AnnotationTool::Eraser => {}
                            //         }
                            //     }
                            // }
                            drop(annotator_state_mut_ref);
                            window.window_context.globals_by_type.insert(TypeId::of::<SharedAnnotatorState>(), annotator_state);
                        });
                    });
            })
        }),
        None,
    );
}
