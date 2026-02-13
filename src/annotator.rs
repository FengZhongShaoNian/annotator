pub mod rectangle;
mod cursor;

use std::any::Any;
use crate::annotator::rectangle::RectangleAnnotationToolState;
use crate::global::Global;
use egui::{
    Area, Color32, Pos2, Rect, Response, Sense, Stroke, TextureHandle, Ui, Vec2, Widget, pos2,
    vec2, widgets,
};
use rustc_hash::FxHashMap;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum StrokeType {
    /// 实线
    SolidLine,

    /// 虚线
    DashedLine,

    /// 点线
    DottedLine,

    /// 点划线
    DashDotLine,

    /// 双点划线
    DashDotDotLine,
}

/// 描述一个矩形的四条边
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Edges {
    Top,
    Right,
    Bottom,
    Left,
}

/// 描述一个矩形的4个角
#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum HitTarget {
    // 边
    TopEdge,
    BottomEdge,
    LeftEdge,
    RightEdge,

    // 角
    TopLeftCorner,
    TopRightCorner,
    BottomLeftCorner,
    BottomRightCorner,

    // 其他可能的情况
    Inside,
    Outside,
}

/// 标注工具的类型
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[derive(Hash)]
pub enum ToolType {
    /// 矩形
    Rectangle,

    /// 椭圆
    Ellipse,

    /// 直线
    StraightLine,

    /// 箭头
    Arrow,

    /// 铅笔
    Pencil,

    /// 记号笔
    MarkerPen,

    /// 马赛克
    Mosaic,

    /// 模糊
    Blur,

    /// 文本
    Text,

    /// 序号
    SerialNumber,

    /// 水印
    Watermark,

    /// 橡皮擦
    Eraser,
}

pub trait Annotation : Widget {
    type State;
    
    fn show(&self, ui: &mut Ui) -> Response;
}

/// 当前标注状态
#[derive(Default)]
pub struct AnnotatorState {
    /// 背景图片的纹理句柄
    pub background_texture_handle: Option<TextureHandle>,

    /// 界面上显示的标注内容
    pub annotations_stack: Vec<Box<dyn Any>>,

    /// "重做"栈：因"撤销"操作而从annotations_stack中弹出的内容会被放入这里，以支持重做
    pub redo_stack: Vec<Box<dyn Any>>,
    
    /// 矩形标注工具的状态
    pub rectangle_annotation_tool_state: RectangleAnnotationToolState,

    /// 当前激活的标注工具
    pub current_annotation_tool: Option<ToolType>,
}

impl Global for AnnotatorState {}
