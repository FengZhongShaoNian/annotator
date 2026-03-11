use crate::annotator::{AnnotatorState, Paint};
use crate::annotator::{
    FillColorSupport, StrokeColorSupport, StrokeType, StrokeTypeSupport, StrokeWidthSupport,
};
use crate::{declare_not_support_fill_color, declare_not_support_stroke_type};
use delegate::delegate;
use egui::{Area, Color32, Id, Painter, Pos2, Rect, Response, Sense, TextEdit, Ui, Widget, vec2};
use std::cell::RefCell;
use std::rc::Weak;

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
    style: TextStyle,
    text: String,
    pos: Pos2,
}

impl TextAnnotation {
    pub fn new(style: TextStyle, text: String, pos: Pos2) -> Self {
        Self { style, text, pos }
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
}

impl Paint for TextAnnotation {
    fn paint_with(&mut self, painter: &Painter) {
        todo!()
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

impl Widget for &mut TextTool {
    fn ui(mut self, ui: &mut Ui) -> Response {
        let sense_area = Rect::from_min_size(Pos2::ZERO, ui.available_size());
        let response = ui.allocate_rect(sense_area, Sense::click_and_drag());

        let Some(pointer_pos) = ui.ctx().pointer_hover_pos() else {
            return response;
        };

        if response.clicked() {
            let annotation =
                TextAnnotation::new(self.tool_state.style, String::from("Hello World"), pointer_pos);
            self.tool_state.current_annotation = Some(annotation);
            Area::new(Id::from("text_edit"))
                .movable(true)
                .current_pos(pointer_pos)
                .show(ui.ctx(), |ui| {
                    ui.add(
                        TextEdit::multiline(
                            self.tool_state
                                .current_annotation
                                .as_mut()
                                .unwrap()
                                .text_mut(),
                        )
                        .text_color(Color32::RED)
                        .background_color(Color32::TRANSPARENT)
                        .min_size(vec2(100.0, 40.0)),
                    );
                });
        }else {
            if let Some(annotation) = self.tool_state.current_annotation.as_mut() {
                Area::new(Id::from("text_edit"))
                    .movable(true)
                    .current_pos(annotation.pos())
                    .show(ui.ctx(), |ui| {
                        ui.add(
                            TextEdit::multiline(
                                self.tool_state
                                    .current_annotation
                                    .as_mut()
                                    .unwrap()
                                    .text_mut(),
                            )
                                .frame(false)
                                .text_color(Color32::RED)
                                .background_color(Color32::TRANSPARENT)
                                .min_size(vec2(100.0, 40.0)),
                        );
                    });
            }
        }


        response
    }
}
