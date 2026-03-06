use crate::annotator::{AnnotatorState, SharedAnnotatorState, StrokeType};
use crate::application::Application;
use crate::dpi::LogicalSize;
use crate::global::{ReadGlobal, ReadGlobalMut};
use crate::view::xdg_popup_view::TriggerType;
use crate::view::{AppView, View};
use crate::window::{AppWindow, WindowConfiguration};
use egui::{Color32, Frame, Id, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, Widget, pos2, vec2, Margin};
use std::any::TypeId;
use std::ops::{Add, Sub};
use std::sync::Arc;
use wayland_protocols::xdg::shell::client::xdg_positioner;

pub type BuildDropdownButtonFn = dyn Fn(&Id, &mut Ui, &mut AnnotatorState) -> Response;
pub type BuildDropdownAreaFn =
    dyn Fn(&mut Application, &mut AppWindow, &mut dyn View, &mut AnnotatorState, &mut Ui);

pub struct DropdownBox<'a, 'w, 's, 'v> {
    pub id: Id,
    pub app: &'a mut Application,
    pub window: &'w mut AppWindow,
    pub current_view: &'v mut dyn View,
    pub annotator_state: &'s mut AnnotatorState,
    pub build_drop_down_box_button_fn: Arc<Box<BuildDropdownButtonFn>>,
    pub build_drop_down_area_fn: Arc<Box<BuildDropdownAreaFn>>,
    pub drop_down_area_size: LogicalSize<u32>,
}

impl Widget for DropdownBox<'_, '_, '_, '_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let build_button_fn = self.build_drop_down_box_button_fn;
        let response = build_button_fn(&self.id, ui, self.annotator_state);
        if response.clicked() {
            let build_drop_down_area_fn = self.build_drop_down_area_fn;
            let popup_view_id = format!("drop-down-box-area-{}", self.id.value());

            let positioner = self.window.create_positioner(&self.app.global_state);

            let current_view_position = self.current_view.position().unwrap();

            let button_rect = &response.rect;

            // 弹出框的尺寸
            positioner.set_size(
                self.drop_down_area_size.width as i32,
                self.drop_down_area_size.height as i32,
            );

            // 按钮的左上角在窗口中的位置
            let button_top_left = button_rect.left_top().add(vec2(
                current_view_position.x as f32,
                current_view_position.y as f32,
            ));

            // 父表面内的锚点矩形
            positioner.set_anchor_rect(
                button_top_left.x.round() as i32,
                button_top_left.y.round() as i32 + button_rect.height() as i32,
                button_rect.width().round() as i32,
                button_rect.height().round() as i32,
            );

            // 指定锚定矩形的哪一条边或角与弹出窗口对齐
            positioner.set_anchor(xdg_positioner::Anchor::BottomLeft);
            // 弹窗相对于锚点的伸展方向
            positioner.set_gravity(xdg_positioner::Gravity::BottomRight);
            positioner.set_offset(0, -20);
            // 空间不足时的自动调整策略
            positioner.set_constraint_adjustment(xdg_positioner::ConstraintAdjustment::all());
            self.window.create_xdg_popup_view(
                popup_view_id.into(),
                &self.app.global_state,
                TriggerType::MousePress,
                positioner,
                Box::new(move |input, egui_ctx, app, window, current_view| {
                    // 构建 UI 的具体内容
                    egui_ctx.run(input, |ctx| {
                        egui::CentralPanel::default()
                            .frame(Frame::new().fill(Color32::from_hex("#393b40").unwrap()).inner_margin(Margin::same(5)))
                            .show(ctx, |ui| {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                                ui.spacing_mut().item_spacing = vec2(1.0, 0.0);

                                let annotator_state = window
                                    .window_context
                                    .globals_by_type
                                    .take::<SharedAnnotatorState>()
                                    .unwrap();

                                build_drop_down_area_fn(
                                    app,
                                    window,
                                    current_view,
                                    &mut annotator_state.borrow_mut(),
                                    ui,
                                );

                                window
                                    .window_context
                                    .globals_by_type
                                    .insert(TypeId::of::<SharedAnnotatorState>(), annotator_state);
                            });
                    })
                }),
            );
        }
        response
    }
}

pub fn create_stroke_type_selector(
    id: Id,
    app: &mut Application,
    window: &mut AppWindow,
    current_view: &mut dyn View,
    annotator_state: &mut AnnotatorState,
    ui: &mut Ui,
) {
    let dropdown = DropdownBox {
        id,
        app,
        window,
        current_view,
        annotator_state,
        drop_down_area_size: LogicalSize::new(100, 120),
        build_drop_down_box_button_fn: Arc::new(Box::new(|_id, ui, annotator_state| {
            let tool = annotator_state.current_annotation_tool.as_ref().unwrap();
            let stroke_type = tool.stroke_type().unwrap();
            let (rect, response) = ui.allocate_exact_size(vec2(100., 20.), Sense::click());
            if response.hovered() {
                ui.painter().rect(
                    rect,
                    3.,
                    Color32::from_hex("#191a1c").unwrap(),
                    Stroke::new(1., Color32::WHITE),
                    StrokeKind::Middle,
                );
            } else {
                ui.painter().rect(
                    rect,
                    3.,
                    Color32::from_hex("#3a3b41").unwrap(),
                    Stroke::new(1., Color32::WHITE),
                    StrokeKind::Middle,
                );
            }
            let padding = 7.;
            let dropdown_arrow_rect_size = vec2(8., 6.);
            let content_rect = rect.shrink(padding);
            let line_rect = Rect::from_min_size(
                content_rect.min,
                vec2(
                    content_rect.width() - dropdown_arrow_rect_size.x - padding,
                    content_rect.height(),
                ),
            );

            let dropdown_arrow_rect = Rect::from_min_size(
                pos2(line_rect.right() + padding, line_rect.top()),
                dropdown_arrow_rect_size,
            );
            ui.painter().add(Shape::convex_polygon(
                vec![
                    dropdown_arrow_rect.left_top(),
                    dropdown_arrow_rect.right_top(),
                    dropdown_arrow_rect.center_bottom(),
                ],
                Color32::WHITE,
                Stroke::new(1.0, Color32::WHITE),
            ));

            let line = get_center_line_segment(&line_rect);
            match stroke_type {
                StrokeType::SolidLine => {
                    ui.painter()
                        .line_segment(line, Stroke::new(1., Color32::WHITE));
                }
                StrokeType::DashedLine => {
                    let shape = Shape::dashed_line(&line, Stroke::new(1., Color32::WHITE), 6., 3.);
                    ui.painter().add(shape);
                }
                StrokeType::DottedLine => {
                    let shape = Shape::dotted_line(&line, Color32::WHITE, 6., 3.);
                    ui.painter().add(shape);
                }
            }
            response
        })),
        build_drop_down_area_fn: Arc::new(Box::new(
            |_app, _window, current_view, annotator_state, ui| {
                ui.vertical_centered(|ui| {
                    let tool = annotator_state.current_annotation_tool.as_mut().unwrap();
                    if ui.add(StrokeTypeButton::new(90., 32., StrokeType::SolidLine)).clicked() {
                        tool.set_stroke_type(StrokeType::SolidLine);
                        current_view.close();
                    }
                    if ui.add(StrokeTypeButton::new(90., 32., StrokeType::DashedLine)).clicked() {
                        tool.set_stroke_type(StrokeType::DashedLine);
                        current_view.close();
                    }
                    if ui.add(StrokeTypeButton::new(90., 32., StrokeType::DottedLine)).clicked() {
                        tool.set_stroke_type(StrokeType::DottedLine);
                        current_view.close();
                    }
                });
            },
        )),
    };
    ui.add(dropdown);
}

struct StrokeTypeButton {
    width: f32,
    height: f32,
    stroke_type: StrokeType
}

impl StrokeTypeButton {
    pub fn new(width: f32, height: f32, stroke_type: StrokeType) -> Self {
        Self {
            width,
            height,
            stroke_type
        }
    }
}

impl Widget for StrokeTypeButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(self.width, self.height), Sense::click());
        if response.hovered() {
            ui.painter().rect_filled(
                rect,
                3.,
                Color32::from_hex("#0860f2").unwrap(),
            );
        } else {
            ui.painter().rect_filled(
                rect,
                3.,
                Color32::from_hex("#3a3b41").unwrap(),
            );
        }
        let line_rect = rect.shrink(6.);
        let line = get_center_line_segment(&line_rect);

        match self.stroke_type {
            StrokeType::SolidLine => {
                ui.painter()
                    .line_segment(line, Stroke::new(1., Color32::WHITE));
            }
            StrokeType::DashedLine => {
                let shape = Shape::dashed_line(&line, Stroke::new(1., Color32::WHITE), 6., 3.);
                ui.painter().add(shape);
            }
            StrokeType::DottedLine => {
                let shape = Shape::dotted_line(&line, Color32::WHITE, 8., 2.);
                ui.painter().add(shape);
            }
        }

        response
    }
}

fn get_center_line_segment(rect: &Rect) -> [Pos2; 2] {
    let line_start = pos2(rect.left(), rect.left_center().y);
    let line_end = pos2(rect.right(), rect.right_center().y);
    [line_start, line_end]
}
