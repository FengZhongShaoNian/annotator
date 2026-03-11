use crate::annotator::rectangle_based::{HitTarget, HitTest};
use crate::annotator::{
    ActivationState, ActivationSupport, Annotation, AnnotationActivationSupport,
    AnnotationToolCommon, AnnotatorState, PainterExt, SharedAnnotatorState, StackTopAccessor,
};
use crate::annotator::{
    FillColorSupport, StrokeColorSupport, StrokeType, StrokeTypeSupport, StrokeWidthSupport,
};
use crate::{
    declare_not_support_fill_color, declare_not_support_stroke_type, impl_stack_top_access_for,
};
use delegate::delegate;
use egui::{
    Area, Color32, CursorIcon, Id, Pos2, Rect, Response, Sense, TextEdit, Ui, Widget, vec2,
};
use std::cell::RefCell;
use std::rc::Weak;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug, Copy, Clone)]
pub struct TextStyle {
    font_color: Color32,
    font_size: f32,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_color: Color32::RED,
            font_size: 18.0,
        }
    }
}

impl StrokeWidthSupport for TextStyle {
    fn supports_get_stroke_width(&self) -> bool {
        true
    }

    fn stroke_width(&self) -> f32 {
        self.font_size
    }

    fn supports_set_stroke_width(&self) -> bool {
        true
    }

    fn set_stroke_width(&mut self, stroke_width: f32) {
        self.font_size = stroke_width;
    }
}

impl StrokeColorSupport for TextStyle {
    fn supports_get_stroke_color(&self) -> bool {
        true
    }

    fn stroke_color(&self) -> Color32 {
        self.font_color
    }

    fn supports_set_stroke_color(&self) -> bool {
        true
    }

    fn set_stroke_color(&mut self, color: Color32) {
        self.font_color = color;
    }
}

declare_not_support_fill_color!(TextStyle);
declare_not_support_stroke_type!(TextStyle);

#[derive(Clone)]
pub struct TextAnnotation {
    id: Id,
    style: TextStyle,
    text: String,
    pos: Pos2,
    activation: ActivationSupport,
    rect: Option<Rect>,
}

impl TextAnnotation {
    pub fn new(
        id: Id,
        style: TextStyle,
        text: String,
        pos: Pos2,
        activation: ActivationSupport,
    ) -> Self {
        Self {
            id,
            style,
            text,
            pos,
            activation,
            rect: None,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn text_mut(&mut self) -> &mut String {
        &mut self.text
    }

    pub fn pos(&self) -> Pos2 {
        self.pos
    }

    pub fn set_pos(&mut self, pos: Pos2) {
        self.pos = pos;
    }

    pub fn rect(&self) -> Option<&Rect> {
        self.rect.as_ref()
    }
}

impl Into<Annotation> for TextAnnotation {
    fn into(self) -> Annotation {
        Annotation::Text(self)
    }
}

impl Widget for &mut TextAnnotation {
    fn ui(self, ui: &mut Ui) -> Response {
        let inner_response = Area::new(self.id)
            .movable(true)
            .current_pos(self.pos())
            .show(ui.ctx(), |ui| {
                let response = ui.add(
                    TextEdit::multiline(self.text_mut())
                        .frame(true)
                        .text_color(Color32::RED)
                        .background_color(Color32::TRANSPARENT)
                        .min_size(vec2(100.0, 40.0)),
                );
                self.rect = Some(response.rect);
                if self.activation.is_active() {
                    let painter = ui.painter();
                    painter.small_rects(&response.rect);
                }
            });
        inner_response.response
    }
}

impl StrokeWidthSupport for TextAnnotation {
    delegate! {
        to self.style {
           /// 是否支持获取线条宽度
           fn supports_get_stroke_width(&self) -> bool;
           /// 获取线条宽度
           fn stroke_width(&self) -> f32;
           /// 是否支持设置线条宽度
           fn supports_set_stroke_width(&self) -> bool;
           /// 设置线条宽度
           fn set_stroke_width(&mut self, stroke_width: f32);
        }
    }
}

impl StrokeColorSupport for TextAnnotation {
    delegate! {
        to self.style {
            /// 是否支持获取线条颜色
            fn supports_get_stroke_color(&self) -> bool;
            /// 获取线条颜色
            fn stroke_color(&self) -> Color32;
            /// 是否支持设置线条颜色
            fn supports_set_stroke_color(&self) -> bool;
            /// 设置线条颜色
            fn set_stroke_color(&mut self, color: Color32);
        }
    }
}

impl AnnotationActivationSupport for TextAnnotation {
    fn activation(&self) -> &ActivationSupport {
        &self.activation
    }

    fn activation_mut(&mut self) -> &mut ActivationSupport {
        &mut self.activation
    }
}

declare_not_support_fill_color!(TextAnnotation);
declare_not_support_stroke_type!(TextAnnotation);

pub struct TextToolState {
    /// 样式
    style: TextStyle,
    /// 当前的标注
    current_annotation: Option<TextAnnotation>,
}

impl Default for TextToolState {
    fn default() -> Self {
        Self {
            style: Default::default(),
            current_annotation: None,
        }
    }
}

pub struct TextTool {
    annotator_state: Weak<RefCell<AnnotatorState>>,
    tool_state: TextToolState,
}

impl TextTool {
    pub fn new(annotator_state: Weak<RefCell<AnnotatorState>>) -> Self {
        Self {
            annotator_state,
            tool_state: TextToolState::default(),
        }
    }

    fn update_cursor_icon(&self, ui: &mut Ui) {
        let Some(pointer_pos) = ui.ctx().pointer_hover_pos() else {
            return;
        };
        // 从标注栈的栈顶中获取最近的一个标注
        let hit_target =
            self.peek_annotation(|annotation_on_stack_top: Option<&TextAnnotation>| {
                // 判断当前鼠标是否位于此标注上
                match annotation_on_stack_top {
                    None => None,
                    Some(annotation) => {
                        let stroke_width = annotation.stroke_width();
                        if let Some(rect) = annotation.rect() {
                            return Some(rect.hit_test(&pointer_pos, stroke_width));
                        }
                        None
                    }
                }
            });

        if let Some(hit_target) = hit_target {
            let cursor_icon = hit_target.get_cursor();
            if let Some(cursor_icon) = cursor_icon {
                ui.ctx().set_cursor_icon(cursor_icon);
            } else {
                ui.ctx().set_cursor_icon(CursorIcon::Text);
            }
        } else {
            ui.ctx().set_cursor_icon(CursorIcon::Text);
        }
    }
}

impl AnnotationToolCommon for TextTool {
    fn annotator_state(&self) -> SharedAnnotatorState {
        self.annotator_state.upgrade().unwrap()
    }
}

impl StrokeWidthSupport for TextTool {
    delegate! {
        to self.tool_state.style {
            /// 是否支持获取线条宽度
            fn supports_get_stroke_width(&self) -> bool;
            /// 获取线条宽度
            fn stroke_width(&self) -> f32;
            /// 是否支持设置线条宽度
            fn supports_set_stroke_width(&self) -> bool;
            /// 设置线条宽度
            fn set_stroke_width(&mut self, stroke_width: f32);
        }
    }
}

impl StrokeColorSupport for TextTool {
    delegate! {
        to self.tool_state.style {
            /// 是否支持获取线条颜色
            fn supports_get_stroke_color(&self) -> bool;
            /// 获取线条颜色
            fn stroke_color(&self) -> Color32;
            /// 是否支持设置线条颜色
            fn supports_set_stroke_color(&self) -> bool;
            /// 设置线条颜色
            fn set_stroke_color(&mut self, color: Color32);
        }
    }
}
declare_not_support_fill_color!(TextTool);
declare_not_support_stroke_type!(TextTool);

static COUNTER: AtomicUsize = AtomicUsize::new(0);
fn generate_unique_text_annotation_id() -> Id {
    format!(
        "text-annotation-{}",
        COUNTER.fetch_add(1, Ordering::Relaxed)
    )
    .into()
}

impl_stack_top_access_for!(TextTool=>TextAnnotation);

impl StackTopAccessor<TextAnnotation> for AnnotatorState {
    fn peek_annotation<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&TextAnnotation>) -> Option<R>,
    {
        // 从标注栈的栈顶中获取最近的一个文本标注
        let text_annotation_on_stack_top = self
            .annotations_stack
            .last()
            .map(|annotation| match annotation {
                Annotation::Text(text_annotation) => Some(text_annotation),
                _ => None,
            })
            .flatten();
        func(text_annotation_on_stack_top)
    }

    fn peek_annotation_mut<F, R>(&mut self, func: F) -> Option<R>
    where
        F: Fn(Option<&mut TextAnnotation>) -> Option<R>,
    {
        // 从标注栈的栈顶中获取最近的一个文本标注
        let text_annotation_on_stack_top = self
            .annotations_stack
            .last_mut()
            .map(|annotation| match annotation {
                Annotation::Text(text_annotation) => Some(text_annotation),
                _ => None,
            })
            .flatten();
        func(text_annotation_on_stack_top)
    }

    fn pop_annotation(&mut self) -> Option<TextAnnotation> {
        // 从标注栈的栈顶中获取最近的一个文本标注
        self.annotations_stack
            .pop()
            .map(|annotation| match annotation {
                Annotation::Text(text_annotation) => Some(text_annotation),
                _ => None,
            })
            .flatten()
    }
}

impl Widget for &mut TextTool {
    fn ui(self, ui: &mut Ui) -> Response {
        let sense_area = Rect::from_min_size(Pos2::ZERO, ui.available_size());
        let response = ui.allocate_rect(sense_area, Sense::click_and_drag());

        let Some(pointer_pos) = ui.ctx().pointer_hover_pos() else {
            return response;
        };

        self.update_cursor_icon(ui);

        if response.drag_started() {
            let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin().unwrap());
            // 从标注栈的栈顶中获取最近的一个标注
            let hit_target =
                self.peek_annotation(|annotation_on_stack_top: Option<&TextAnnotation>| {
                    // 判断当前鼠标是否位于此标注上
                    match annotation_on_stack_top {
                        None => None,
                        Some(annotation) => {
                            let stroke_width = annotation.stroke_width();
                            if let Some(rect) = annotation.rect() {
                                return Some(rect.hit_test(&drag_started_pos, stroke_width));
                            }
                            None
                        }
                    }
                });

            if let Some(hit_target) = hit_target {
                if hit_target != HitTarget::Inside && hit_target != HitTarget::Outside {
                    let support_activate = self
                        .peek_annotation(|annotation_on_stack_top: Option<&TextAnnotation>| {
                            Some(
                                annotation_on_stack_top
                                    .unwrap()
                                    .activation()
                                    .supports_activate(),
                            )
                        })
                        .unwrap();

                    if support_activate {
                        // 调整现有的标注
                        let mut annotation = self.pop_annotation().unwrap();
                        annotation.activation_mut().activate();
                        self.tool_state.current_annotation = Some(annotation);
                    }
                } else if hit_target == HitTarget::Inside || hit_target == HitTarget::Outside {
                    self.peek_annotation_mut(|mut annotation_on_stack_top| {
                        // 把栈顶的标注设为非激活状态
                        annotation_on_stack_top
                            .as_mut()
                            .unwrap()
                            .activation_mut()
                            .deactivate();
                        None::<()>
                    });
                }
            } else {
                self.peek_annotation_mut(|mut annotation_on_stack_top| {
                    // 把栈顶的标注设为非激活状态
                    annotation_on_stack_top
                        .as_mut()
                        .unwrap()
                        .activation_mut()
                        .deactivate();
                    None::<()>
                });
            }
        } else if response.dragged() {
            if let Some(annotation) = self.tool_state.current_annotation.as_mut() {
                annotation.set_pos(pointer_pos);
            }
        } else if response.dragged() {
            if let Some(annotation) = self.tool_state.current_annotation.take() {
                let annotation_state = self.annotator_state.upgrade().unwrap();
                annotation_state
                    .borrow_mut()
                    .annotations_stack
                    .push(annotation.into());
            }
        } else if response.clicked() {
            let annotation = TextAnnotation::new(
                generate_unique_text_annotation_id(),
                self.tool_state.style,
                String::from("Hello World"),
                pointer_pos,
                ActivationSupport::Supported(ActivationState::active()),
            );
            let annotation_state = self.annotator_state.upgrade().unwrap();
            annotation_state
                .borrow_mut()
                .annotations_stack
                .push(annotation.into());
        }

        response
    }
}
