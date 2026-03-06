use crate::annotator::{AnnotationTool, AnnotatorState, SharedAnnotatorState};
use crate::application::Application;
use crate::dpi::{LogicalPosition, LogicalSize};
use crate::global::ReadGlobal;
use crate::view::{View, ViewId};
use crate::window::AppWindow;
use egui::{vec2, Color32, Frame, Id, Ui};
use std::any::TypeId;
use std::sync::Arc;
use crate::annotator::drop_down_box::DropdownBox;

fn run_ui<F>(app: &mut Application, window: &mut AppWindow, current_view: &mut dyn View, ui: &mut Ui, func: F)
where
    F: Fn(&mut Application, &mut AppWindow, &mut dyn View, &mut Ui, &mut AnnotatorState),
{
    let annotator_state = window
        .window_context
        .globals_by_type
        .take::<SharedAnnotatorState>()
        .unwrap();
    let mut annotator_state_mut_ref = annotator_state.borrow_mut();
    func(app, window, current_view, ui, &mut annotator_state_mut_ref);
    drop(annotator_state_mut_ref);
    window
        .window_context
        .globals_by_type
        .insert(TypeId::of::<SharedAnnotatorState>(), annotator_state);
}

pub fn create_secondly_toolbar(
    view_id: ViewId,
    app: &mut Application,
    window: &mut AppWindow,
    toolbar_size: LogicalSize<u32>,
    toolbar_positon: LogicalPosition<i32>,
) {
    let global_state = &app.global_state;

    // 创建子工具条
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

                        run_ui(app, window, current_view, ui, move |app, window, current_view, ui, annotator_state| {
                            let active_tool = &mut annotator_state.current_annotation_tool;

                            if active_tool.is_none() {
                                current_view.set_visible(false);
                                return;
                            }

                            if matches!(active_tool, Some(AnnotationTool::Rectangle(..))) {
                                let dropdown = DropdownBox {
                                    id: Id::from("stroke-type-selector"),
                                    app,
                                    window,
                                    current_view,
                                    annotator_state,
                                    build_drop_down_box_button_fn: Arc::new(Box::new(|_id, ui| {
                                        ui.button("线条样式")
                                    })),
                                    build_drop_down_area_fn: Arc::new(Box::new(|app, window, current_view, annotator_state, ui| {
                                        ui.label("实线");
                                        ui.label("虚线");
                                        ui.label("点线");
                                    })),
                                };
                                ui.add(dropdown);
                            }
                        });


                    });
            })
        }),
        None,
    );
}
