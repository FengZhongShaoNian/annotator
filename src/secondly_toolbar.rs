use crate::annotator::drop_down_box::{DropdownBox, create_stroke_type_selector};
use crate::annotator::{
    AnnotationTool, AnnotatorState, PainterExt, SharedAnnotatorState, StrokeType,
};
use crate::application::Application;
use crate::dpi::{LogicalPosition, LogicalSize};
use crate::global::ReadGlobal;
use crate::view::{View, ViewId};
use crate::window::AppWindow;
use egui::{
    Color32, Frame, Id, Margin, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, Widget, pos2,
    vec2,
};
use std::any::TypeId;
use std::sync::Arc;
use crate::context::Command;

fn run_ui<F>(
    app: &mut Application,
    window: &mut AppWindow,
    current_view: &mut dyn View,
    ui: &mut Ui,
    func: F,
) where
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
                    .frame(
                        Frame::new()
                            .fill(Color32::from_hex("#393b40").unwrap())
                            .inner_margin(Margin::same(5)),
                    )
                    .show(ctx, |ui| {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                        ui.horizontal_centered(|ui| {
                            run_ui(
                                app,
                                window,
                                current_view,
                                ui,
                                move |app, window, current_view, ui, annotator_state| {
                                    let active_tool = &mut annotator_state.current_annotation_tool;

                                    if active_tool.is_none() {
                                        current_view.set_visible(false);
                                        return;
                                    }

                                    if matches!(active_tool, Some(AnnotationTool::Rectangle(..))) {
                                        let mut width = create_stroke_type_selector(
                                            Id::from("stroke-type-selector"),
                                            app,
                                            window,
                                            current_view,
                                            annotator_state,
                                            ui,
                                        );

                                        width += create_color_selector(
                                            app,
                                            window,
                                            current_view,
                                            annotator_state,
                                            ui,
                                        );

                                        width += 20;

                                        let new_size = LogicalSize::new(width, current_view.size().height);
                                        window.window_context.commands.push_back(Command::ResizeView(current_view.id(), new_size));
                                        return;
                                    }
                                },
                            );
                        })
                    });
            })
        }),
        None,
    );
}

fn create_color_selector(
    app: &mut Application,
    window: &mut AppWindow,
    current_view: &mut dyn View,
    annotation_state: &mut AnnotatorState,
    ui: &mut Ui,
) -> u32 {
    let button_width = 18f32;
    let width = button_width * 5. + ui.spacing().item_spacing.x * 5.;
    let tool = annotation_state.current_annotation_tool.as_mut().unwrap();
    let current_color = tool.color().unwrap();
    if ui.add(ColorButton::new(Color32::RED, button_width, button_width, current_color == Color32::RED)).clicked() {
        tool.set_color(Color32::RED);
    }
    if ui.add(ColorButton::new(Color32::BLACK, button_width, button_width, current_color == Color32::BLACK)).clicked() {
        tool.set_color(Color32::BLACK);
    }
    if ui.add(ColorButton::new(Color32::BLUE, button_width, button_width, current_color == Color32::BLUE)).clicked() {
        tool.set_color(Color32::BLUE);
    }
    if ui.add(ColorButton::new(Color32::GREEN, button_width, button_width, current_color == Color32::GREEN)).clicked() {
        tool.set_color(Color32::GREEN);
    }
    if ui.add(ColorButton::new(Color32::GOLD, button_width, button_width, current_color == Color32::GOLD)).clicked() {
        tool.set_color(Color32::GOLD);
    }
    width.round() as u32
}

struct ColorButton {
    color: Color32,
    width: f32,
    height: f32,
    checked: bool,
}

impl ColorButton {
    fn new(color: Color32, width: f32, height: f32, checked: bool) -> ColorButton {
        ColorButton {
            color,
            width,
            height,
            checked,
        }
    }
}

impl Widget for ColorButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(self.width, self.height), Sense::click());
        if self.checked {
            ui.painter().rect(rect, 0., self.color, Stroke::new(3., Color32::from_hex("#0860f2").unwrap()), StrokeKind::Inside);
        }else {
            ui.painter().rect_filled(rect, 0., self.color);
        }
        response
    }
}
