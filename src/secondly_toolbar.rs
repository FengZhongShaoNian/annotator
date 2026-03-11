use crate::annotator::drop_down_box::create_stroke_type_selector;
use crate::annotator::{AnnotatorState, FillColorSupport, SharedAnnotatorState, StrokeColorSupport, StrokeTypeSupport, ToolName};
use crate::application::Application;
use crate::dpi::{LogicalPosition, LogicalSize};
use crate::global::ReadGlobal;
use crate::view::{View, ViewId};
use crate::window::AppWindow;
use egui::{
    vec2, Color32, Frame, Id, Margin, Response, Sense, Stroke, StrokeKind, Ui,
    Widget,
};
use std::any::TypeId;

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
                                    let active_tool = annotator_state
                                        .current_annotation_tool
                                        .as_ref();

                                    let Some(active_tool) = active_tool else {
                                        current_view.set_visible(false);
                                        return;
                                    };

                                    let supports_set_stroke_type = active_tool.supports_set_stroke_type();
                                    let supports_set_stroke_color = active_tool.supports_set_stroke_color();

                                    if supports_set_stroke_type {
                                        create_stroke_type_selector(
                                            Id::from("stroke-type-selector"),
                                            app,
                                            window,
                                            current_view,
                                            annotator_state,
                                            ui,
                                        );
                                    }

                                    if supports_set_stroke_color {
                                        create_color_selector(
                                            app,
                                            window,
                                            current_view,
                                            annotator_state,
                                            ui,
                                        );
                                    }

                                    // let new_size = LogicalSize::new(width, current_view.size().height);
                                    // window.window_context.commands.push_back(Command::ResizeView(current_view.id(), new_size));
                                    return;
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
    annotator_state: &mut AnnotatorState,
    ui: &mut Ui,
) -> u32 {
    let button_width = 18f32;
    let width = button_width * 5. + ui.spacing().item_spacing.x * 5.;
    let tool = annotator_state.current_annotation_tool.as_mut().unwrap();
    let current_color = tool.stroke_color();
    if tool.tool_name() != ToolName::MarkerPen {
        if ui
            .add(ColorButton::new(
                Color32::RED,
                button_width,
                button_width,
                current_color == Color32::RED,
            ))
            .clicked()
        {
            tool.set_stroke_color(Color32::RED);
            if tool.fill_color().is_some() {
                tool.set_fill_color(Color32::RED);
            }
        }
        if ui
            .add(ColorButton::new(
                Color32::BLACK,
                button_width,
                button_width,
                current_color == Color32::BLACK,
            ))
            .clicked()
        {
            tool.set_stroke_color(Color32::BLACK);
            if tool.fill_color().is_some() {
                tool.set_fill_color(Color32::BLACK);
            }
        }
        if ui
            .add(ColorButton::new(
                Color32::BLUE,
                button_width,
                button_width,
                current_color == Color32::BLUE,
            ))
            .clicked()
        {
            tool.set_stroke_color(Color32::BLUE);
            if tool.fill_color().is_some() {
                tool.set_fill_color(Color32::BLUE);
            }
        }
        if ui
            .add(ColorButton::new(
                Color32::GREEN,
                button_width,
                button_width,
                current_color == Color32::GREEN,
            ))
            .clicked()
        {
            tool.set_stroke_color(Color32::GREEN);
            if tool.fill_color().is_some() {
                tool.set_fill_color(Color32::GREEN);
            }
        }
        if ui
            .add(ColorButton::new(
                Color32::GOLD,
                button_width,
                button_width,
                current_color == Color32::GOLD,
            ))
            .clicked()
        {
            tool.set_stroke_color(Color32::GOLD);
            if tool.fill_color().is_some() {
                tool.set_fill_color(Color32::GOLD);
            }
        }
    }else {
        // 马克笔是半透明的
        let red = Color32::from_rgba_unmultiplied(255, 0, 0, 76);
        if ui
            .add(ColorButton::new(
                red,
                button_width,
                button_width,
                current_color == red,
            ))
            .clicked()
        {
            tool.set_stroke_color(red);
            if tool.fill_color().is_some() {
                tool.set_fill_color(red);
            }
        }
        let black = Color32::from_rgba_unmultiplied(0, 0, 0, 76);
        if ui
            .add(ColorButton::new(
                black,
                button_width,
                button_width,
                current_color == black,
            ))
            .clicked()
        {
            tool.set_stroke_color(black);
            if tool.fill_color().is_some() {
                tool.set_fill_color(black);
            }
        }

        let blue = Color32::from_rgba_unmultiplied(0, 0, 255, 76);
        if ui
            .add(ColorButton::new(
                blue,
                button_width,
                button_width,
                current_color == blue,
            ))
            .clicked()
        {
            tool.set_stroke_color(blue);
            if tool.fill_color().is_some() {
                tool.set_fill_color(blue);
            }
        }
        let green = Color32::from_rgba_unmultiplied(0, 255, 0, 76);
        if ui
            .add(ColorButton::new(
                green,
                button_width,
                button_width,
                current_color == green,
            ))
            .clicked()
        {
            tool.set_stroke_color(green);
            if tool.fill_color().is_some() {
                tool.set_fill_color(green);
            }
        }
        let gold = Color32::from_rgba_unmultiplied(255, 215, 0, 76);
        if ui
            .add(ColorButton::new(
                gold,
                button_width,
                button_width,
                current_color == gold,
            ))
            .clicked()
        {
            tool.set_stroke_color(gold);
            if tool.fill_color().is_some() {
                tool.set_fill_color(gold);
            }
        }
    }

    if current_color != tool.stroke_color() {
        annotator_state
            .annotations_stack
            .last_mut()
            .map(|annotation| {
                if annotation.was_created_by(tool) && annotation.activation().is_active() {
                    annotation.set_stroke_color(tool.stroke_color());
                    if annotation.fill_color().is_some() && tool.fill_color().is_some() {
                        annotation.set_fill_color(tool.fill_color().unwrap());
                    }
                }
            });
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
        let (rect, response) =
            ui.allocate_exact_size(vec2(self.width, self.height), Sense::click());
        if self.checked {
            ui.painter().rect(
                rect,
                0.,
                self.color,
                Stroke::new(3., Color32::from_hex("#0860f2").unwrap()),
                StrokeKind::Inside,
            );
        } else {
            ui.painter().rect_filled(rect, 0., self.color);
        }
        response
    }
}
