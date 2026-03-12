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
use egui::{Area, Color32, CursorIcon, Id, Margin, Pos2, Rect, Response, Sense, Stroke, StrokeKind, TextEdit, Ui, Widget, vec2, FontSelection, FontId};
use log::info;
use std::cell::RefCell;
use std::rc::Weak;
use std::sync::atomic::{AtomicUsize, Ordering};
use crate::annotator::cursor::{CustomCursor, Move};

#[derive(Debug, Clone)]
pub struct TextStyle {
    /// 文本颜色
    text_color: Color32,
    /// 字体族和字体大小
    font: FontId,
    /// 文本和边框的间距
    padding: Margin,
    /// 边框的线条样式
    stroke: Stroke,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            text_color: Color32::RED,
            font: FontId::proportional(18.),
            padding: Margin::same(10),
            stroke: Stroke::new(1.0, Color32::WHITE),
        }
    }
}

impl StrokeWidthSupport for TextStyle {
    fn supports_get_stroke_width(&self) -> bool {
        true
    }

    fn stroke_width(&self) -> f32 {
        self.stroke.width
    }

    fn supports_set_stroke_width(&self) -> bool {
        false
    }

    fn set_stroke_width(&mut self, _stroke_width: f32) {
        unimplemented!()
    }
}

impl StrokeColorSupport for TextStyle {
    fn supports_get_stroke_color(&self) -> bool {
        true
    }

    fn stroke_color(&self) -> Color32 {
        self.stroke.color
    }

    fn supports_set_stroke_color(&self) -> bool {
        false
    }

    fn set_stroke_color(&mut self, _color: Color32) {
        unimplemented!()
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
    /// 仅用于记录文本标注组件的矩形区域信息，不应该从外部修改它
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

    fn max_line_width(&self, ui: &mut Ui) -> f32 {
        let galley = ui.fonts_mut(|fonts_view| {
            fonts_view.layout(
                self.text.clone(),
                self.style.font.clone(),
                self.style.text_color,
                f32::INFINITY, // 不因宽度折行，保留原始换行
            )
        });
        galley
            .rows
            .iter()
            .map(|row| row.rect().width())
            .fold(0.0, f32::max)
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

const MIN_TEXT_WIDTH: f32 = 30.0;

impl Widget for &mut TextAnnotation {
    fn ui(self, ui: &mut Ui) -> Response {
        let max_text_width = self.max_line_width(ui);
        let style = self.style.clone();
        let row_height = ui.fonts_mut(|f| f.row_height(&style.font));
        let text_width = max_text_width.max(MIN_TEXT_WIDTH);
        let inner_response = Area::new(self.id)
            .current_pos(self.pos() - vec2(style.padding.leftf(), style.padding.topf() + row_height/2.0))
            .movable(false)
            .show(ui.ctx(), |ui| {
                let frame = egui::Frame::default().inner_margin(style.padding);
                let response = frame.show(ui, |ui| {
                    let text_color = self.style.text_color;
                    let interactive = self.activation.is_active();
                    let text_edit = TextEdit::multiline(self.text_mut())
                        .frame(false)
                        .text_color(text_color)
                        .font(style.font)
                        .background_color(Color32::GREEN)
                        .interactive(interactive)
                        .margin(Margin::same(0))
                        .desired_width(f32::INFINITY)
                        .desired_rows(1);
                    let response = ui.add_sized(vec2(text_width, row_height + style.padding.sum().y), text_edit);
                    if self.activation.is_active() {
                        response.request_focus();
                    }
                });
                let rect = response.response.rect;
                self.rect = Some(rect);
                if self.activation.is_active() {
                    let painter = ui.painter();
                    painter.rect_stroke(rect, 0., self.style.stroke, StrokeKind::Middle);
                    painter.small_rects(&rect);
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
    custom_cursor: Option<Box<dyn CustomCursor>>,
}

impl TextTool {
    pub fn new(annotator_state: Weak<RefCell<AnnotatorState>>) -> Self {
        Self {
            annotator_state,
            tool_state: TextToolState::default(),
            custom_cursor: None,
        }
    }

    /// 如果栈顶不存在TextAnnotation或者TextAnnotation暂时不知道rect，则返回None
    /// 否则返回碰撞检测结果
    fn hit_test_for_annotation_on_stack_top(&self, pointer_pos: &Pos2) -> Option<HitTarget> {
        self.peek_annotation(|annotation_on_stack_top: Option<&TextAnnotation>| {
            // 判断当前鼠标是否位于此标注上
            match annotation_on_stack_top {
                None => None,
                Some(annotation) => Self::hit_test_for_annotation(annotation, pointer_pos),
            }
        })
    }

    fn hit_test_for_annotation(
        annotation: &TextAnnotation,
        pointer_pos: &Pos2,
    ) -> Option<HitTarget> {
        // 判断当前鼠标是否位于此标注上
        let stroke_width = annotation.stroke_width();
        if let Some(rect) = annotation.rect() {
            Some(rect.hit_test(&pointer_pos, stroke_width))
        } else {
            None
        }
    }

    fn update_cursor_icon(&mut self, ui: &mut Ui) {
        let Some(pointer_pos) = ui.ctx().pointer_hover_pos() else {
            return;
        };
        // 从标注栈的栈顶中获取最近的一个标注
        if let Some(annotation) = self.tool_state.current_annotation.as_ref() {
            let hit_target = Self::hit_test_for_annotation(annotation, &pointer_pos);
            if let Some(hit_target) = hit_target {
                // 鼠标位于边框上
                if hit_target != HitTarget::Outside && hit_target != HitTarget::Inside {
                    ui.ctx().set_cursor_icon(CursorIcon::Move);
                } else {
                    ui.ctx().set_cursor_icon(CursorIcon::Text);
                }
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

        self.update_cursor_icon(ui);

        if response.drag_started() {
            info!("drag started!");
        } else if response.clicked() {
            println!("clicked!");
            let pointer_pos = response.hover_pos().unwrap();
            let hit_target = self.hit_test_for_annotation_on_stack_top(&pointer_pos);
            if hit_target.is_none() || hit_target.unwrap() != HitTarget::Inside {
                if let Some(mut annotation) = self.tool_state.current_annotation.take() {
                    annotation.activation.deactivate();
                    let annotation_state = self.annotator_state.upgrade().unwrap();
                    annotation_state
                        .borrow_mut()
                        .annotations_stack
                        .push(annotation.into());
                }
                let annotation = TextAnnotation::new(
                    generate_unique_text_annotation_id(),
                    self.tool_state.style.clone(),
                    String::new(),
                    pointer_pos,
                    ActivationSupport::Supported(ActivationState::active()),
                );
                self.tool_state.current_annotation = Some(annotation);
            }
        } else if response.dragged() {
            println!("dragged!");
            if let Some(annotation) = self.tool_state.current_annotation.as_mut() {
                let new_pos = annotation.pos + response.drag_delta();
                annotation.set_pos(new_pos);
                ui.add(annotation);
            }
        } else {
            if let Some(annotation) = self.tool_state.current_annotation.as_mut() {
                ui.add(annotation);
            }
        }

        if let Some(cursor) = self.custom_cursor.as_ref() {
            cursor.paint_with(ui.painter());
        }

        response
    }
}
