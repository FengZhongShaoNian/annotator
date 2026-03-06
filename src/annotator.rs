mod arrow;
mod blur;
mod cursor;
pub mod drop_down_box;
pub mod ellipse;
pub mod eraser;
pub mod marker_pen;
pub mod mosaic;
pub mod pencil;
pub mod rectangle;
pub mod serial_number;
pub mod straight_line;
pub(crate) mod svg_button;
pub mod text;
pub mod watermark;

use crate::annotator::arrow::ArrowState;
use crate::annotator::blur::BlurState;
use crate::annotator::ellipse::{EllipseState, EllipseTool, EllipseToolState};
use crate::annotator::eraser::EraserState;
use crate::annotator::marker_pen::MarkerPentate;
use crate::annotator::mosaic::MosaicState;
use crate::annotator::pencil::PencilState;
use crate::annotator::rectangle::{RectangleState, RectangleTool, RectangleToolState};
use crate::annotator::serial_number::SerialNumberState;
use crate::annotator::straight_line::{StraightLineState, StraightLineTool};
use crate::annotator::text::TextState;
use crate::annotator::watermark::WaterMarkState;
use crate::global::Global;
use crate::view::ViewId;
use egui::ahash::HashMap;
use egui::{
    Color32, CornerRadius, CursorIcon, Painter, Pos2, Rect, Response, Shape, Stroke, StrokeKind,
    TextureHandle, Ui, Widget, pos2, vec2,
};
use image::RgbaImage;
use std::any::Any;
use std::cell::RefCell;
use std::cmp::max;
use std::ops::{Add, Sub};
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum StrokeType {
    /// 实线
    SolidLine,

    /// 虚线
    DashedLine,

    /// 点线
    DottedLine,
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

impl HitTarget {
    /// 根据HitTarget获取对应的光标
    pub fn get_cursor(&self) -> Option<CursorIcon> {
        match self {
            HitTarget::TopLeftCorner => Some(CursorIcon::ResizeNorthWest),
            HitTarget::TopRightCorner => Some(CursorIcon::ResizeNorthEast),
            HitTarget::BottomRightCorner => Some(CursorIcon::ResizeSouthEast),
            HitTarget::BottomLeftCorner => Some(CursorIcon::ResizeSouthWest),
            HitTarget::TopEdge => Some(CursorIcon::ResizeNorth),
            HitTarget::RightEdge => Some(CursorIcon::ResizeEast),
            HitTarget::BottomEdge => Some(CursorIcon::ResizeSouth),
            HitTarget::LeftEdge => Some(CursorIcon::ResizeWest),
            _ => None,
        }
    }

    pub fn get_drag_action(&self) -> DragAction {
        match self {
            HitTarget::TopLeftCorner => DragAction::AdjustTopLeftCorner,
            HitTarget::TopRightCorner => DragAction::AdjustTopRightCorner,
            HitTarget::BottomRightCorner => DragAction::AdjustBottomRightCorner,
            HitTarget::BottomLeftCorner => DragAction::AdjustBottomLeftCorner,
            HitTarget::TopEdge => DragAction::AdjustTopEdge,
            HitTarget::RightEdge => DragAction::AdjustRightEdge,
            HitTarget::BottomEdge => DragAction::AdjustBottomEdge,
            HitTarget::LeftEdge => DragAction::AdjustLeftEdge,
            _ => DragAction::None,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum DragAction {
    // 调整边
    AdjustTopEdge,
    AdjustBottomEdge,
    AdjustLeftEdge,
    AdjustRightEdge,

    // 调整角
    AdjustTopLeftCorner,
    AdjustTopRightCorner,
    AdjustBottomLeftCorner,
    AdjustBottomRightCorner,

    None,
}

pub enum Annotation {
    /// 矩形
    Rectangle(RectangleState),

    /// 椭圆
    Ellipse(EllipseState),

    /// 直线
    StraightLine(StraightLineState),

    /// 箭头
    Arrow(ArrowState),

    /// 铅笔
    Pencil(PencilState),

    /// 记号笔
    MarkerPen(MarkerPentate),

    /// 马赛克
    Mosaic(MosaicState),

    /// 模糊
    Blur(BlurState),

    /// 文本
    Text(TextState),

    /// 序号
    SerialNumber(SerialNumberState),

    /// 水印
    Watermark(WaterMarkState),

    /// 橡皮擦
    Eraser(EraserState),
}

impl Annotation {
    pub fn activate(&mut self) {
        match self {
            Annotation::Rectangle(rectangle_state) => {
                rectangle_state.activate();
            }
            Annotation::Ellipse(ellipse_state) => {
                ellipse_state.activate();
            }
            Annotation::StraightLine(straight_line_state) => {
                straight_line_state.activate();
            }
            Annotation::Arrow(arrow_state) => {}
            Annotation::Pencil(pencil_state) => {}
            Annotation::MarkerPen(marker_pen_state) => {}
            Annotation::Mosaic(mosaic_state) => {}
            Annotation::Blur(blur_state) => {}
            Annotation::Text(text_state) => {}
            Annotation::SerialNumber(serial_number_state) => {}
            Annotation::Watermark(watermark_state) => {}
            Annotation::Eraser(eraser_state) => {}
        }
    }

    pub fn deactivate(&mut self) {
        match self {
            Annotation::Rectangle(rectangle_state) => {
                rectangle_state.deactivate();
            }
            Annotation::Ellipse(ellipse_state) => {
                ellipse_state.deactivate();
            }
            Annotation::StraightLine(straight_line_state) => {
                straight_line_state.deactivate();
            }
            Annotation::Arrow(arrow_state) => {}
            Annotation::Pencil(pencil_state) => {}
            Annotation::MarkerPen(marker_pen_state) => {}
            Annotation::Mosaic(mosaic_state) => {}
            Annotation::Blur(blur_state) => {}
            Annotation::Text(text_state) => {}
            Annotation::SerialNumber(serial_number_state) => {}
            Annotation::Watermark(watermark_state) => {}
            Annotation::Eraser(eraser_state) => {}
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            Annotation::Rectangle(rectangle_state) => rectangle_state.is_active(),
            Annotation::Ellipse(ellipse_state) => ellipse_state.is_active(),
            Annotation::StraightLine(straight_line_state) => straight_line_state.is_active(),
            Annotation::Arrow(arrow_state) => {
                todo!()
            }
            Annotation::Pencil(pencil_state) => {
                todo!()
            }
            Annotation::MarkerPen(marker_pen_state) => {
                todo!()
            }
            Annotation::Mosaic(mosaic_state) => {
                todo!()
            }
            Annotation::Blur(blur_state) => {
                todo!()
            }
            Annotation::Text(text_state) => {
                todo!()
            }
            Annotation::SerialNumber(serial_number_state) => {
                todo!()
            }
            Annotation::Watermark(watermark_state) => {
                todo!()
            }
            Annotation::Eraser(eraser_state) => {
                todo!()
            }
        }
    }

    pub fn was_created_by(&mut self, tool: &AnnotationTool) -> bool {
        let tool_name = tool.tool_name();
        match self {
            Annotation::Rectangle(_) => tool_name == ToolName::Rectangle,
            Annotation::Ellipse(ellipse_state) => tool_name == ToolName::Ellipse,
            Annotation::StraightLine(_) => tool_name == ToolName::StraightLine,
            Annotation::Arrow(_) => tool_name == ToolName::Arrow,
            Annotation::Pencil(_) => tool_name == ToolName::Pencil,
            Annotation::MarkerPen(_) => tool_name == ToolName::MarkerPen,
            Annotation::Mosaic(_) => tool_name == ToolName::Mosaic,
            Annotation::Blur(_) => tool_name == ToolName::Blur,
            Annotation::Text(_) => tool_name == ToolName::Text,
            Annotation::SerialNumber(_) => tool_name == ToolName::SerialNumber,
            Annotation::Watermark(_) => tool_name == ToolName::Watermark,
            Annotation::Eraser(_) => tool_name == ToolName::Eraser,
        }
    }

    pub fn set_color(&mut self, color: Color32) {
        match self {
            Annotation::Rectangle(rectangle_state) => {
                rectangle_state.set_color(color);
            }
            Annotation::Ellipse(ellipse_state) => {
                ellipse_state.set_color(color);
            }
            Annotation::StraightLine(straight_line_state) => {}
            Annotation::Arrow(arrow_state) => {}
            Annotation::Pencil(pencil_state) => {}
            Annotation::MarkerPen(marker_pen_state) => {}
            Annotation::Mosaic(mosaic_state) => {}
            Annotation::Blur(blur_state) => {}
            Annotation::Text(text_state) => {}
            Annotation::SerialNumber(serial_number_state) => {}
            Annotation::Watermark(watermark_state) => {}
            Annotation::Eraser(eraser_state) => {}
        }
    }

    pub fn set_stroke_type(&mut self, stroke_type: StrokeType) {
        match self {
            Annotation::Rectangle(rectangle_state) => {
                rectangle_state.set_stroke_type(stroke_type);
            }
            Annotation::Ellipse(ellipse_state) => {}
            Annotation::StraightLine(straight_line_state) => {
                straight_line_state.set_stroke_type(stroke_type);
            }
            Annotation::Arrow(arrow_state) => {}
            Annotation::Pencil(pencil_state) => {}
            Annotation::MarkerPen(marker_pen_state) => {}
            Annotation::Mosaic(mosaic_state) => {}
            Annotation::Blur(blur_state) => {}
            Annotation::Text(text_state) => {}
            Annotation::SerialNumber(serial_number_state) => {}
            Annotation::Watermark(watermark_state) => {}
            Annotation::Eraser(eraser_state) => {}
        }
    }
}

impl Widget for &mut Annotation {
    fn ui(self, ui: &mut Ui) -> Response {
        match self {
            Annotation::Rectangle(rectangle_state) => ui.add(rectangle_state),
            Annotation::Ellipse(ellipse_state) => ui.add(ellipse_state),
            Annotation::StraightLine(straight_line_state) => ui.add(straight_line_state),
            Annotation::Arrow(arrow_state) => ui.add(arrow_state),
            Annotation::Pencil(pencil_state) => ui.add(pencil_state),
            Annotation::MarkerPen(marker_pen_state) => ui.add(marker_pen_state),
            Annotation::Mosaic(mosaic_state) => ui.add(mosaic_state),
            Annotation::Blur(blur_state) => ui.add(blur_state),
            Annotation::Text(text_state) => ui.add(text_state),
            Annotation::SerialNumber(serial_number_state) => ui.add(serial_number_state),
            Annotation::Watermark(watermark_state) => ui.add(watermark_state),
            Annotation::Eraser(eraser_state) => ui.add(eraser_state),
        }
    }
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum ToolName {
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

/// 标注工具的类型
pub enum AnnotationTool {
    /// 矩形
    Rectangle(RectangleTool),

    /// 椭圆
    Ellipse(EllipseTool),

    /// 直线
    StraightLine(StraightLineTool),

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

impl AnnotationTool {
    pub fn tool_name(&self) -> ToolName {
        match self {
            AnnotationTool::Rectangle(_) => ToolName::Rectangle,
            AnnotationTool::Ellipse(_) => ToolName::Ellipse,
            AnnotationTool::StraightLine(_) => ToolName::StraightLine,
            AnnotationTool::Arrow => ToolName::Arrow,
            AnnotationTool::Pencil => ToolName::Pencil,
            AnnotationTool::MarkerPen => ToolName::MarkerPen,
            AnnotationTool::Mosaic => ToolName::Mosaic,
            AnnotationTool::Blur => ToolName::Blur,
            AnnotationTool::Text => ToolName::Text,
            AnnotationTool::SerialNumber => ToolName::SerialNumber,
            AnnotationTool::Watermark => ToolName::Watermark,
            AnnotationTool::Eraser => ToolName::Eraser,
        }
    }

    pub fn stroke_type(&self) -> Option<StrokeType> {
        match self {
            AnnotationTool::Rectangle(rectangle_tool) => Some(rectangle_tool.stroke_type()),
            AnnotationTool::Ellipse(_) => None,
            AnnotationTool::StraightLine(straight_line_tool) => {
                Some(straight_line_tool.stroke_type())
            }
            AnnotationTool::Arrow => {
                todo!("Arrow")
            }
            AnnotationTool::Pencil => {
                todo!("Pencil")
            }
            AnnotationTool::MarkerPen => {
                todo!("Marker Pen")
            }
            AnnotationTool::Mosaic => {
                todo!("Mosaic")
            }
            AnnotationTool::Blur => {
                todo!("Blur")
            }
            AnnotationTool::Text => {
                todo!("Text")
            }
            AnnotationTool::SerialNumber => {
                todo!("Serial Number")
            }
            AnnotationTool::Watermark => {
                todo!("Watermark")
            }
            AnnotationTool::Eraser => {
                todo!("Eraser")
            }
        }
    }

    pub fn set_stroke_type(&mut self, stroke_type: StrokeType) {
        match self {
            AnnotationTool::Rectangle(rectangle_tool) => {
                rectangle_tool.set_stroke_type(stroke_type);
            }
            AnnotationTool::Ellipse(_) => {}
            AnnotationTool::StraightLine(straight_line_tool) => {
                straight_line_tool.set_stroke_type(stroke_type);
            }
            AnnotationTool::Arrow => {
                todo!("Arrow")
            }
            AnnotationTool::Pencil => {
                todo!("Pencil")
            }
            AnnotationTool::MarkerPen => {
                todo!("Marker Pen")
            }
            AnnotationTool::Mosaic => {
                todo!("Mosaic")
            }
            AnnotationTool::Blur => {
                todo!("Blur")
            }
            AnnotationTool::Text => {
                todo!("Text")
            }
            AnnotationTool::SerialNumber => {
                todo!("Serial Number")
            }
            AnnotationTool::Watermark => {
                todo!("Watermark")
            }
            AnnotationTool::Eraser => {
                todo!("Eraser")
            }
        }
    }

    pub fn color(&self) -> Option<Color32> {
        match self {
            AnnotationTool::Rectangle(rectangle_tool) => Some(rectangle_tool.color()),
            AnnotationTool::Ellipse(ellipse_tool) => Some(ellipse_tool.color()),
            AnnotationTool::StraightLine(straight_line_tool) => Some(straight_line_tool.color()),
            AnnotationTool::Arrow => {
                todo!("Arrow")
            }
            AnnotationTool::Pencil => {
                todo!("Pencil")
            }
            AnnotationTool::MarkerPen => {
                todo!("Marker Pen")
            }
            AnnotationTool::Mosaic => {
                todo!("Mosaic")
            }
            AnnotationTool::Blur => {
                todo!("Blur")
            }
            AnnotationTool::Text => {
                todo!("Text")
            }
            AnnotationTool::SerialNumber => {
                todo!("Serial Number")
            }
            AnnotationTool::Watermark => {
                todo!("Watermark")
            }
            AnnotationTool::Eraser => {
                todo!("Eraser")
            }
        }
    }

    pub fn set_color(&mut self, color: Color32) {
        match self {
            AnnotationTool::Rectangle(rectangle_tool) => {
                rectangle_tool.set_color(color);
            }
            AnnotationTool::Ellipse(ellipse_tool) => {
                ellipse_tool.set_color(color);
            }
            AnnotationTool::StraightLine(straight_line_tool) => {
                straight_line_tool.set_color(color);
            }
            AnnotationTool::Arrow => {
                todo!("Arrow")
            }
            AnnotationTool::Pencil => {
                todo!("Pencil")
            }
            AnnotationTool::MarkerPen => {
                todo!("Marker Pen")
            }
            AnnotationTool::Mosaic => {
                todo!("Mosaic")
            }
            AnnotationTool::Blur => {
                todo!("Blur")
            }
            AnnotationTool::Text => {
                todo!("Text")
            }
            AnnotationTool::SerialNumber => {
                todo!("Serial Number")
            }
            AnnotationTool::Watermark => {
                todo!("Watermark")
            }
            AnnotationTool::Eraser => {
                todo!("Eraser")
            }
        }
    }
}

impl Widget for &mut AnnotationTool {
    fn ui(self, ui: &mut Ui) -> Response {
        match self {
            AnnotationTool::Rectangle(rectangle_tool) => rectangle_tool.ui(ui),
            AnnotationTool::Ellipse(ellipse_tool) => ellipse_tool.ui(ui),
            AnnotationTool::StraightLine(straight_line_tool) => straight_line_tool.ui(ui),
            AnnotationTool::Arrow => {
                todo!("Arrow")
            }
            AnnotationTool::Pencil => {
                todo!("Pencil")
            }
            AnnotationTool::MarkerPen => {
                todo!("Marker Pen")
            }
            AnnotationTool::Mosaic => {
                todo!("Mosaic")
            }
            AnnotationTool::Blur => {
                todo!("Blur")
            }
            AnnotationTool::Text => {
                todo!("Text")
            }
            AnnotationTool::SerialNumber => {
                todo!("Serial Number")
            }
            AnnotationTool::Watermark => {
                todo!("Watermark")
            }
            AnnotationTool::Eraser => {
                todo!("Eraser")
            }
        }
    }
}

/// 当前标注状态
#[derive(Default)]
pub struct AnnotatorState {
    /// 是否隐藏主工具条
    pub hide_primary_toolbar: bool,

    /// 背景图片
    pub background_image: Arc<RgbaImage>,

    /// 背景图片的纹理句柄
    pub background_texture_handle: Option<TextureHandle>,

    /// 标注工具
    pub annotation_tools: HashMap<ToolName, AnnotationTool>,

    /// 界面上显示的标注内容
    pub annotations_stack: Vec<Annotation>,

    /// "重做"栈：因"撤销"操作而从annotations_stack中弹出的内容会被放入这里，以支持重做
    pub redo_stack: Vec<Annotation>,

    /// 当前激活的标注工具
    pub current_annotation_tool: Option<AnnotationTool>,
}

pub type SharedAnnotatorState = Rc<RefCell<AnnotatorState>>;

impl Global for SharedAnnotatorState {}

pub trait SharedAnnotatorStateUtil {
    fn with_current_annotation_tool<F>(&self, func: F)
    where
        F: FnOnce(&mut AnnotationTool);
}

impl SharedAnnotatorStateUtil for SharedAnnotatorState {
    fn with_current_annotation_tool<F>(&self, func: F)
    where
        F: FnOnce(&mut AnnotationTool),
    {
        let mut annotator_state_mut_ref = self.borrow_mut();
        let mut current_annotation_tool = annotator_state_mut_ref
            .current_annotation_tool
            .take()
            .unwrap();
        drop(annotator_state_mut_ref);

        func(&mut current_annotation_tool);

        let mut annotator_state_mut_ref = self.borrow_mut();
        annotator_state_mut_ref
            .current_annotation_tool
            .replace(current_annotation_tool);
    }
}

impl AnnotatorState {
    pub fn annotator_panel_id() -> ViewId {
        "annotator-panel".into()
    }
    pub fn primary_toolbar_id() -> ViewId {
        "primary-toolbar".into()
    }
    pub fn secondly_toolbar_id() -> ViewId {
        "secondly-toolbar".into()
    }

    pub fn activate_annotation_tool(&mut self, tool_name: ToolName) {
        if let Some(active_tool) = &self.current_annotation_tool {
            if active_tool.tool_name() == tool_name {
                return;
            }
        }
        let tool = self
            .annotation_tools
            .remove(&tool_name)
            .expect(&format!("{:?}Tool does not exist", tool_name));
        if let Some(previous_tool) = self.current_annotation_tool.replace(tool) {
            self.annotation_tools
                .insert(previous_tool.tool_name(), previous_tool);
        }
    }

    pub fn deactivate_annotation_tool(&mut self) {
        if let Some(previous_tool) = self.current_annotation_tool.take() {
            self.annotation_tools
                .insert(previous_tool.tool_name(), previous_tool);
        }
    }
}

trait SmallRect {
    /// 将一个点扩展成一个小矩形
    fn rect(&self, width: f32, height: f32) -> Rect;
}

impl SmallRect for Pos2 {
    fn rect(&self, width: f32, height: f32) -> Rect {
        let pos = self;
        let half_width = width / 2f32;
        let half_height = height / 2f32;
        let top_left_pos = pos2(pos.x - half_width, pos.y - half_height);
        let right_bottom_pos = pos2(pos.x + half_width, pos.y + half_height);
        Rect::from_two_pos(top_left_pos, right_bottom_pos)
    }
}

pub trait HitTest {
    /// 对矩形做碰撞检测
    /// 矩形的stroke_kind固定为StrokeKind::Middle
    fn hit_test(&self, pointer_pos: &Pos2, stroke_width: f32) -> HitTarget;
}

impl HitTest for Rect {
    /// 对矩形的4条边做碰撞检测（不会特别精准），返回发生碰撞的边
    /// 矩形的stroke_kind固定为StrokeKind::Middle
    fn hit_test(&self, pointer_pos: &Pos2, stroke_width: f32) -> HitTarget {
        // 允许一定的误差
        let tolerance = 6.;

        let tolerance = if tolerance > stroke_width {
            tolerance
        } else {
            stroke_width
        };
        let half = tolerance / 2.;

        let mut edges = Vec::new();

        // 矩形：
        //  (min.x, min.y)   (max.x, min.y)
        //
        //  (min.x, max.y)   (max.x, max.y)

        let min_pos = self.min;
        let max_pos = self.max;

        // 把每一条边当作一个小矩形来对待
        // 上边
        let small_rect = Rect::from_two_pos(
            pos2(min_pos.x - half, min_pos.y - half),
            pos2(max_pos.x + half, min_pos.y + half),
        );
        if small_rect.contains(*pointer_pos) {
            edges.push(HitTarget::TopEdge);
        }

        // 右边
        let small_rect = Rect::from_two_pos(
            pos2(max_pos.x - half, min_pos.y - half),
            pos2(max_pos.x + half, max_pos.y + half),
        );
        if small_rect.contains(*pointer_pos) {
            edges.push(HitTarget::RightEdge);
        }

        // 下边
        let small_rect = Rect::from_two_pos(
            pos2(min_pos.x - half, max_pos.y - half),
            pos2(max_pos.x + half, max_pos.y + half),
        );
        if small_rect.contains(*pointer_pos) {
            edges.push(HitTarget::BottomEdge);
        }

        // 左边
        let small_rect = Rect::from_two_pos(
            pos2(min_pos.x - half, min_pos.y - half),
            pos2(min_pos.x + half, max_pos.y + half),
        );
        if small_rect.contains(*pointer_pos) {
            edges.push(HitTarget::LeftEdge);
        }

        if edges.contains(&HitTarget::TopEdge) && edges.contains(&HitTarget::LeftEdge) {
            return HitTarget::TopLeftCorner;
        }

        if edges.contains(&HitTarget::TopEdge) && edges.contains(&HitTarget::RightEdge) {
            return HitTarget::TopRightCorner;
        }

        if edges.contains(&HitTarget::BottomEdge) && edges.contains(&HitTarget::RightEdge) {
            return HitTarget::BottomRightCorner;
        }

        if edges.contains(&HitTarget::BottomEdge) && edges.contains(&HitTarget::LeftEdge) {
            return HitTarget::BottomLeftCorner;
        }

        if edges.is_empty() {
            if self.contains(*pointer_pos) {
                return HitTarget::Inside;
            }
        } else {
            return *edges.first().unwrap();
        }

        HitTarget::Outside
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 创建测试用的矩形 (x: 0-100, y: 0-50)
    fn test_rect() -> Rect {
        Rect::from_two_pos(pos2(0.0, 0.0), pos2(100.0, 50.0))
    }

    // 测试基本边界情况
    #[test]
    fn test_hit_test_basic_edges() {
        let rect = test_rect();

        // 测试上边 - 中点
        assert_eq!(rect.hit_test(&pos2(50.0, 0.0), 1.0), HitTarget::TopEdge);

        // 测试上边 - 偏移3像素内 (tolerance=6, half=3)
        assert_eq!(rect.hit_test(&pos2(50.0, 2.9), 1.0), HitTarget::TopEdge);

        // 测试下边
        assert_eq!(rect.hit_test(&pos2(50.0, 50.0), 1.0), HitTarget::BottomEdge);

        // 测试左边
        assert_eq!(rect.hit_test(&pos2(0.0, 25.0), 1.0), HitTarget::LeftEdge);

        // 测试右边
        assert_eq!(rect.hit_test(&pos2(100.0, 25.0), 1.0), HitTarget::RightEdge);
    }

    // 测试角点检测
    #[test]
    fn test_hit_test_corners() {
        let rect = test_rect();

        // 左上角
        assert_eq!(
            rect.hit_test(&pos2(0.0, 0.0), 1.0),
            HitTarget::TopLeftCorner
        );

        // 右上角
        assert_eq!(
            rect.hit_test(&pos2(100.0, 0.0), 1.0),
            HitTarget::TopRightCorner
        );

        // 左下角
        assert_eq!(
            rect.hit_test(&pos2(0.0, 50.0), 1.0),
            HitTarget::BottomLeftCorner
        );

        // 右下角
        assert_eq!(
            rect.hit_test(&pos2(100.0, 50.0), 1.0),
            HitTarget::BottomRightCorner
        );
    }

    // 测试角点区域扩展 (tolerance=6, half=3)
    #[test]
    fn test_hit_test_corner_extended() {
        let rect = test_rect();

        // 左上角扩展区域 (x: -3 to 3, y: -3 to 3)
        assert_eq!(
            rect.hit_test(&pos2(2.9, 2.9), 1.0),
            HitTarget::TopLeftCorner
        );

        // 右上角扩展区域 (x: 97 to 103, y: -3 to 3)
        assert_eq!(
            rect.hit_test(&pos2(97.1, 2.9), 1.0),
            HitTarget::TopRightCorner
        );
    }

    // 测试内部和外部
    #[test]
    fn test_hit_test_inside_outside() {
        let rect = test_rect();

        // 内部中心点
        assert_eq!(rect.hit_test(&pos2(50.0, 25.0), 1.0), HitTarget::Inside);

        // 内部但不是中心
        assert_eq!(rect.hit_test(&pos2(10.0, 10.0), 1.0), HitTarget::Inside);

        // 完全外部
        assert_eq!(rect.hit_test(&pos2(-10.0, 25.0), 1.0), HitTarget::Outside);

        assert_eq!(rect.hit_test(&pos2(50.0, -10.0), 1.0), HitTarget::Outside);

        assert_eq!(rect.hit_test(&pos2(150.0, 25.0), 1.0), HitTarget::Outside);
    }

    // 测试边缘扩展区域
    #[test]
    fn test_hit_test_edge_extended() {
        let rect = test_rect();

        // 上边扩展区域 (y: -3 to 3)
        assert_eq!(rect.hit_test(&pos2(50.0, -2.9), 1.0), HitTarget::TopEdge);

        // 超出扩展区域
        assert_eq!(rect.hit_test(&pos2(50.0, -3.1), 1.0), HitTarget::Outside);

        // 下边扩展区域 (y: 47 to 53)
        assert_eq!(rect.hit_test(&pos2(50.0, 52.9), 1.0), HitTarget::BottomEdge);

        // 左边扩展区域 (x: -3 to 3)
        assert_eq!(rect.hit_test(&pos2(-2.9, 25.0), 1.0), HitTarget::LeftEdge);

        // 右边扩展区域 (x: 97 to 103)
        assert_eq!(rect.hit_test(&pos2(102.9, 25.0), 1.0), HitTarget::RightEdge);
    }

    // 测试不同 stroke_width 值
    #[test]
    fn test_hit_test_different_stroke_width() {
        let rect = test_rect();

        // stroke_width=1.0，tolerance=6
        assert_eq!(rect.hit_test(&pos2(50.0, 2.9), 1.0), HitTarget::TopEdge);

        assert_eq!(
            rect.hit_test(&pos2(50.0, 3.1), 1.0),
            HitTarget::Inside // 在扩展区域外，但在矩形内
        );

        // stroke_width=10.0，tolerance=10，half=5
        // 现在扩展区域更大
        assert_eq!(rect.hit_test(&pos2(50.0, 4.9), 10.0), HitTarget::TopEdge);

        assert_eq!(
            rect.hit_test(&pos2(50.0, 5.1), 10.0),
            HitTarget::Inside // 在扩展区域外，但在矩形内
        );

        // stroke_width=20.0，tolerance=20，half=10
        // 扩展区域非常大
        assert_eq!(rect.hit_test(&pos2(50.0, 9.9), 20.0), HitTarget::TopEdge);

        // 注意：当扩展区域非常大时，甚至可能覆盖到矩形内部
        // 测试一个在矩形内部但在扩展区域内的点
        assert_eq!(rect.hit_test(&pos2(50.0, 8.0), 20.0), HitTarget::TopEdge);
    }

    // 测试边界条件和特殊情况
    #[test]
    fn test_hit_test_edge_cases() {
        let rect = test_rect();

        // 点在边上但x坐标超出矩形范围（但在扩展区域内）
        assert_eq!(
            rect.hit_test(&pos2(-2.9, 0.0), 1.0),
            HitTarget::TopLeftCorner // 同时在上边和左边
        );

        // 点在角的扩展区域边缘
        assert_eq!(
            rect.hit_test(&pos2(3.0, 3.0), 1.0),
            HitTarget::TopLeftCorner
        );

        // 点刚好在扩展区域边界上
        assert_eq!(
            rect.hit_test(&pos2(103.0, 3.0), 1.0),
            HitTarget::TopRightCorner
        );

        // 负坐标测试
        let rect2 = Rect::from_two_pos(pos2(-50.0, -50.0), pos2(50.0, 50.0));
        assert_eq!(rect2.hit_test(&pos2(0.0, -50.0), 1.0), HitTarget::TopEdge);
    }

    // 性能测试：测试多个点
    #[test]
    fn test_hit_test_multiple_points() {
        let rect = test_rect();
        let stroke_width = 1.0;

        // 创建测试点网格
        for x in -10..=110 {
            for y in -10..=60 {
                let x = x as f32;
                let y = y as f32;
                let point = pos2(x, y);
                let _result = rect.hit_test(&point, stroke_width);
                // 这里我们不验证具体结果，只是确保不会panic
            }
        }
    }
}

/// 小矩形的默认宽度和高度
pub const DEFAULT_SIZE_FOR_SMALL_RECT: (f32, f32) = (6., 6.);

pub trait PainterExt {
    /// 将一个的扩展成一个小矩形并绘制它
    fn small_rect(&self, pos: &Pos2);

    /// 为一个矩形绘制各个角以及边上的小矩形
    fn small_rects(&self, rect: &Rect);

    /// 绘制矩形，支持填充和不同风格的边框
    ///
    /// # 参数
    /// - `painter`: egui 绘制器
    /// - `rect`: 矩形区域（填充区域）
    /// - `fill_color`: 填充颜色
    /// - `stroke`: 边框样式（颜色、宽度）
    /// - `stroke_kind`: 边框对齐方式（Inside / Outside / Middle）
    /// - `stroke_type`: 线条类型（实线 / 虚线 / 点线）
    fn rectangle(
        &self,
        rect: &Rect,
        fill_color: impl Into<Color32>,
        stroke: impl Into<Stroke>,
        stroke_kind: StrokeKind,
        stroke_type: StrokeType,
    );
}

impl PainterExt for Painter {
    fn small_rect(&self, pos: &Pos2) {
        let painter = self;
        let width = DEFAULT_SIZE_FOR_SMALL_RECT.0;
        let height = DEFAULT_SIZE_FOR_SMALL_RECT.1;
        painter.rect(
            pos.rect(width, height),
            0,
            Color32::TRANSPARENT,
            Stroke::new(1f32, Color32::WHITE),
            StrokeKind::Middle,
        );
    }

    fn small_rects(&self, rect: &Rect) {
        let painter = self;

        let top_left_pos = rect.left_top();
        let top_right_pos = rect.right_top();
        let bottom_right_pos = rect.right_bottom();
        let bottom_left_pos = rect.left_bottom();

        let center_left_edge = pos2(top_left_pos.x, top_left_pos.y + rect.height() / 2f32);
        let center_right_edge = pos2(top_right_pos.x, top_right_pos.y + rect.height() / 2f32);
        let center_top_edge = pos2(top_left_pos.x + rect.width() / 2f32, top_left_pos.y);
        let center_bottom_edge = pos2(bottom_left_pos.x + rect.width() / 2f32, bottom_left_pos.y);

        painter.small_rect(&top_left_pos);
        painter.small_rect(&top_right_pos);
        painter.small_rect(&bottom_right_pos);
        painter.small_rect(&bottom_left_pos);

        painter.small_rect(&center_left_edge);
        painter.small_rect(&center_right_edge);
        painter.small_rect(&center_top_edge);
        painter.small_rect(&center_bottom_edge);
    }

    fn rectangle(
        &self,
        rect: &Rect,
        fill_color: impl Into<Color32>,
        stroke: impl Into<Stroke>,
        stroke_kind: StrokeKind,
        stroke_type: StrokeType,
    ) {
        let painter = self;
        let fill_color = fill_color.into();
        let stroke = stroke.into();

        // 1. 绘制填充矩形
        painter.rect_filled(*rect, 0.0, fill_color);

        // 2. 根据对齐方式计算边框路径所在的矩形
        let half_width = stroke.width / 2.0;
        let path_rect = match stroke_kind {
            StrokeKind::Inside => rect.shrink(half_width),
            StrokeKind::Outside => rect.expand(half_width),
            StrokeKind::Middle => *rect,
        };

        // 3. 绘制边框
        match stroke_type {
            StrokeType::SolidLine => {
                // 实线直接用 rect_stroke
                painter.rect_stroke(path_rect, 0.0, stroke, stroke_kind);
            }
            StrokeType::DashedLine => {
                // 虚线：使用 dashed_line，自定义 dash 和 gap 长度
                let dash_len = dash_len_for_dashed_line(stroke.width);
                let gap_len = gap_len_for_dashed_line(stroke.width);
                draw_dashed_rect(painter, path_rect, stroke, dash_len, gap_len);
            }
            StrokeType::DottedLine => {
                // 点线：使用 dotted_line，根据线宽计算点间距和半径
                let spacing = spacing_for_dotted_line(stroke.width); // 点间距
                let radius = radius_for_dotted_line(stroke.width); // 点半径
                draw_dotted_rect(painter, path_rect, stroke.color, spacing, radius);
            }
        }
    }
}

pub fn dash_len_for_dashed_line(stroke_width: f32) -> f32 {
    let dash_len = if stroke_width * 3. < 6. {
        6.
    } else {
        stroke_width * 3.
    };
    dash_len
}

pub fn gap_len_for_dashed_line(stroke_width: f32) -> f32 {
    let gap_len = if stroke_width * 3. < 6. {
        6.
    } else {
        stroke_width * 3.
    };
    gap_len
}

pub fn spacing_for_dotted_line(stroke_width: f32) -> f32{
    let spacing = stroke_width * 2.0; // 点间距
    if spacing < 6. {
        6.
    } else {
        spacing
    }
}

pub fn radius_for_dotted_line(stroke_width: f32) -> f32{
    let radius = stroke_width / 2.0;
    radius
}

/// 绘制矩形的虚线边框
fn draw_dashed_rect(painter: &Painter, rect: Rect, stroke: Stroke, dash_len: f32, gap_len: f32) {
    let [left, right, top, bottom] = [rect.left(), rect.right(), rect.top(), rect.bottom()];

    let edges = [
        (Pos2::new(left, top), Pos2::new(right, top)), // 上边
        (Pos2::new(right, top), Pos2::new(right, bottom)), // 右边
        (Pos2::new(right, bottom), Pos2::new(left, bottom)), // 下边
        (Pos2::new(left, bottom), Pos2::new(left, top)), // 左边
    ];

    for (start, end) in edges {
        let shape = Shape::dashed_line(&[start, end], stroke, dash_len, gap_len);
        painter.add(shape);
    }
}

/// 绘制矩形的点线边框
fn draw_dotted_rect(painter: &Painter, rect: Rect, color: Color32, spacing: f32, radius: f32) {
    let [left, right, top, bottom] = [rect.left(), rect.right(), rect.top(), rect.bottom()];

    let edges = [
        (Pos2::new(left, top), Pos2::new(right, top)), // 上边
        (Pos2::new(right, top), Pos2::new(right, bottom)), // 右边
        (Pos2::new(right, bottom), Pos2::new(left, bottom)), // 下边
        (Pos2::new(left, bottom), Pos2::new(left, top)), // 左边
    ];

    for (start, end) in edges {
        // dotted_line 返回 Vec<Shape>，需要逐个添加
        let shapes = Shape::dotted_line(&[start, end], color, spacing, radius);
        for shape in shapes {
            painter.add(shape);
        }
    }
}
