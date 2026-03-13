use crate::annotator::drop_down_box::{STROKE_TYPE_SELECTOR_WIDTH, create_stroke_type_selector};
use crate::annotator::{
    AnnotatorState, FillColorSupport, FontColorSupport, SharedAnnotatorState, StrokeColorSupport,
    StrokeTypeSupport, ToolName,
};
use crate::application::Application;
use crate::context::Command;
use crate::dpi::{LogicalPosition, LogicalSize};
use crate::global::ReadGlobal;
use crate::view::{View, ViewId};
use crate::window::AppWindow;
use egui::{
    Color32, Frame, Id, Margin, Response, Sense, Stroke, StrokeKind, Ui, Widget, response, vec2,
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

                        {
                            let has_active_tool = window
                                .window_context
                                .globals_by_type
                                .require_ref::<SharedAnnotatorState>()
                                .borrow()
                                .current_annotation_tool
                                .is_some();

                            if !has_active_tool {
                                current_view.set_visible(false);
                                return;
                            };
                        }

                        ui.horizontal_centered(|ui| {
                            run_ui(
                                app,
                                window,
                                current_view,
                                ui,
                                move |app, window, current_view, ui, annotator_state| {
                                    let active_tool =
                                        annotator_state.current_annotation_tool.as_ref().unwrap();

                                    let supports_set_stroke_type =
                                        active_tool.supports_set_stroke_type();
                                    let supports_set_stroke_color =
                                        active_tool.supports_set_stroke_color();
                                    let supports_set_fill_color =
                                        active_tool.supports_set_fill_color();
                                    let supports_set_font_color =
                                        active_tool.supports_set_font_color();

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

                                    let show_color_selector = supports_set_stroke_color
                                        || supports_set_fill_color
                                        || supports_set_font_color;

                                    if show_color_selector {
                                        create_color_selector(
                                            app,
                                            window,
                                            current_view,
                                            annotator_state,
                                            ui,
                                        );
                                    }

                                    if !supports_set_stroke_type && !show_color_selector {
                                        current_view.set_visible(false);
                                        return;
                                    }

                                    let color_selector_width = 393;
                                    let mut new_width = color_selector_width;
                                    if supports_set_stroke_type {
                                        new_width += STROKE_TYPE_SELECTOR_WIDTH + 10;
                                    }
                                    let new_size =
                                        LogicalSize::new(new_width, current_view.size().height);
                                    window
                                        .window_context
                                        .commands
                                        .push_back(Command::ResizeView(
                                            current_view.id(),
                                            new_size,
                                        ));
                                },
                            );
                        });
                    });
            })
        }),
        None,
    );
}

fn create_color_selector(
    _app: &mut Application,
    _window: &mut AppWindow,
    _current_view: &mut dyn View,
    annotator_state: &mut AnnotatorState,
    ui: &mut Ui,
) {
    let button_width = 18f32;
    let tool = annotator_state.current_annotation_tool.as_mut().unwrap();

    // 1. 暂时不打算区分线条颜色和填充颜色（也就是如果一个工具同时支持线条颜色和填充颜色（而且启用了填充颜色），那么二者总是保持一致）
    // 2. 线条颜色和字体颜色不会同时出现

    let current_color = if tool.supports_get_stroke_color() {
        tool.stroke_color()
    } else if tool.supports_get_fill_color() {
        tool.fill_color().unwrap()
    } else {
        tool.font_color()
    };

    let candidate_colors = &annotator_state.candidate_colors;
    for color in candidate_colors {
        // 马克笔支持的是带透明通道的颜色，其它工具支持的是不带透明通道的颜色
        if tool.tool_name() != ToolName::MarkerPen {
            if ui
                .add(ColorButton::new(
                    *color,
                    button_width,
                    button_width,
                    current_color == *color,
                ))
                .clicked()
            {
                if tool.supports_set_stroke_color() {
                    tool.set_stroke_color(*color);
                }
                if tool.supports_set_fill_color() {
                    if tool.fill_color().is_some() {
                        tool.set_fill_color(*color);
                    }
                }
                if tool.supports_set_font_color() {
                    tool.set_font_color(*color);
                }
            }
        } else {
            let with_alpha_color =
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 76);
            if ui
                .add(ColorButton::new(
                    *color,
                    button_width,
                    button_width,
                    current_color == with_alpha_color,
                ))
                .clicked()
            {
                if tool.supports_set_stroke_color() {
                    tool.set_stroke_color(with_alpha_color);
                }
                if tool.supports_set_fill_color() {
                    if tool.fill_color().is_some() {
                        tool.set_fill_color(with_alpha_color);
                    }
                }
                if tool.supports_set_font_color() {
                    tool.set_font_color(with_alpha_color);
                }
            }
        }
    }

    let new_current_color = if tool.supports_get_stroke_color() {
        tool.stroke_color()
    } else if tool.supports_set_fill_color() {
        tool.fill_color().unwrap()
    } else {
        tool.font_color()
    };

    if current_color != new_current_color {
        annotator_state
            .annotations_stack
            .last_mut()
            .map(|annotation| {
                if annotation.was_created_by(tool) && annotation.activation().is_active() {
                    if annotation.supports_get_stroke_color() {
                        annotation.set_stroke_color(tool.stroke_color());
                    }
                    if annotation.supports_set_fill_color() {
                        if annotation.fill_color().is_some() && tool.fill_color().is_some() {
                            annotation.set_fill_color(tool.fill_color().unwrap());
                        }
                    }
                    if tool.supports_set_font_color() {
                        tool.set_font_color(tool.font_color());
                    }
                }
            });
    }
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
