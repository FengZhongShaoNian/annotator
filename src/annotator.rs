mod cursor;
pub mod rectangle;
pub(crate) mod svg_button;
pub(crate) mod ellipse;

use crate::annotator::ellipse::EllipseAnnotationToolState;
use crate::annotator::rectangle::RectangleAnnotationToolState;
use crate::global::Global;
use egui::{pos2, vec2, Color32, CornerRadius, CursorIcon, Painter, Pos2, Rect, Response, Shape, Stroke, StrokeKind, TextureHandle, Ui, Widget};
use std::any::Any;
use std::cmp::max;
use std::ops::Add;
use crate::view::ViewId;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum StrokeType {
    /// 实线
    SolidLine,

    /// 虚线
    DashedLine,

    /// 点线
    DottedLine
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
    
    None
}


/// 标注工具的类型
#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
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

pub trait Annotation: Widget {
    fn show(&self, ui: &mut Ui) -> Response;
}

/// 当前标注状态
#[derive(Default)]
pub struct AnnotatorState {
    /// 是否隐藏主工具条
    pub hide_primary_toolbar: bool,

    /// 背景图片的纹理句柄
    pub background_texture_handle: Option<TextureHandle>,

    /// 界面上显示的标注内容
    pub annotations_stack: Vec<Box<dyn Any>>,

    /// "重做"栈：因"撤销"操作而从annotations_stack中弹出的内容会被放入这里，以支持重做
    pub redo_stack: Vec<Box<dyn Any>>,

    /// 矩形标注工具的状态
    pub rectangle_annotation_tool_state: RectangleAnnotationToolState,
    
    /// 椭圆标注工具的状态
    pub ellipse_annotation_tool_state: EllipseAnnotationToolState,

    /// 当前激活的标注工具
    pub current_annotation_tool: Option<ToolType>,
}

impl AnnotatorState {
    pub fn primary_toolbar_id() -> ViewId {
        "primary-toolbar".into()
    }
    pub fn secondly_toolbar_id() -> ViewId {
        "secondly-toolbar".into()
    }
}

impl Global for AnnotatorState {}


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
        }else {
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
        assert_eq!(
            rect.hit_test(&pos2(50.0, 0.0), 1.0),
            HitTarget::TopEdge
        );

        // 测试上边 - 偏移3像素内 (tolerance=6, half=3)
        assert_eq!(
            rect.hit_test(&pos2(50.0, 2.9), 1.0),
            HitTarget::TopEdge
        );

        // 测试下边
        assert_eq!(
            rect.hit_test(&pos2(50.0, 50.0), 1.0),
            HitTarget::BottomEdge
        );

        // 测试左边
        assert_eq!(
            rect.hit_test(&pos2(0.0, 25.0), 1.0),
            HitTarget::LeftEdge
        );

        // 测试右边
        assert_eq!(
            rect.hit_test(&pos2(100.0, 25.0), 1.0),
            HitTarget::RightEdge
        );
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
        assert_eq!(
            rect.hit_test(&pos2(50.0, 25.0), 1.0),
            HitTarget::Inside
        );

        // 内部但不是中心
        assert_eq!(
            rect.hit_test(&pos2(10.0, 10.0), 1.0),
            HitTarget::Inside
        );

        // 完全外部
        assert_eq!(
            rect.hit_test(&pos2(-10.0, 25.0), 1.0),
            HitTarget::Outside
        );

        assert_eq!(
            rect.hit_test(&pos2(50.0, -10.0), 1.0),
            HitTarget::Outside
        );

        assert_eq!(
            rect.hit_test(&pos2(150.0, 25.0), 1.0),
            HitTarget::Outside
        );
    }

    // 测试边缘扩展区域
    #[test]
    fn test_hit_test_edge_extended() {
        let rect = test_rect();

        // 上边扩展区域 (y: -3 to 3)
        assert_eq!(
            rect.hit_test(&pos2(50.0, -2.9), 1.0),
            HitTarget::TopEdge
        );

        // 超出扩展区域
        assert_eq!(
            rect.hit_test(&pos2(50.0, -3.1), 1.0),
            HitTarget::Outside
        );

        // 下边扩展区域 (y: 47 to 53)
        assert_eq!(
            rect.hit_test(&pos2(50.0, 52.9), 1.0),
            HitTarget::BottomEdge
        );

        // 左边扩展区域 (x: -3 to 3)
        assert_eq!(
            rect.hit_test(&pos2(-2.9, 25.0), 1.0),
            HitTarget::LeftEdge
        );

        // 右边扩展区域 (x: 97 to 103)
        assert_eq!(
            rect.hit_test(&pos2(102.9, 25.0), 1.0),
            HitTarget::RightEdge
        );
    }

    // 测试不同 stroke_width 值
    #[test]
    fn test_hit_test_different_stroke_width() {
        let rect = test_rect();

        // stroke_width=1.0，tolerance=6
        assert_eq!(
            rect.hit_test(&pos2(50.0, 2.9), 1.0),
            HitTarget::TopEdge
        );

        assert_eq!(
            rect.hit_test(&pos2(50.0, 3.1), 1.0),
            HitTarget::Inside  // 在扩展区域外，但在矩形内
        );

        // stroke_width=10.0，tolerance=10，half=5
        // 现在扩展区域更大
        assert_eq!(
            rect.hit_test(&pos2(50.0, 4.9), 10.0),
            HitTarget::TopEdge
        );

        assert_eq!(
            rect.hit_test(&pos2(50.0, 5.1), 10.0),
            HitTarget::Inside  // 在扩展区域外，但在矩形内
        );

        // stroke_width=20.0，tolerance=20，half=10
        // 扩展区域非常大
        assert_eq!(
            rect.hit_test(&pos2(50.0, 9.9), 20.0),
            HitTarget::TopEdge
        );

        // 注意：当扩展区域非常大时，甚至可能覆盖到矩形内部
        // 测试一个在矩形内部但在扩展区域内的点
        assert_eq!(
            rect.hit_test(&pos2(50.0, 8.0), 20.0),
            HitTarget::TopEdge
        );
    }

    // 测试边界条件和特殊情况
    #[test]
    fn test_hit_test_edge_cases() {
        let rect = test_rect();

        // 点在边上但x坐标超出矩形范围（但在扩展区域内）
        assert_eq!(
            rect.hit_test(&pos2(-2.9, 0.0), 1.0),
            HitTarget::TopLeftCorner  // 同时在上边和左边
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
        assert_eq!(
            rect2.hit_test(&pos2(0.0, -50.0), 1.0),
            HitTarget::TopEdge
        );
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

pub trait PainterExt {
    /// 为一个矩形绘制各个角以及边上的小矩形
    fn small_rects(&self, rect: &Rect);

    /// 绘制矩形
    fn rectangle(&self, rect: &Rect,
                        fill_color: impl Into<Color32>,
                        stroke: impl Into<Stroke>,
                        stroke_kind: StrokeKind,
                        stroke_type: StrokeType);
}

impl PainterExt for Painter {
    fn small_rects(&self, rect: &Rect) {
        let painter = self;

        let width = 6f32;
        let height = 6f32;
        let top_left_pos = rect.left_top();
        let top_right_pos = rect.right_top();
        let bottom_right_pos = rect.right_bottom();
        let bottom_left_pos = rect.left_bottom();

        let center_left_edge = pos2(top_left_pos.x, top_left_pos.y + rect.height() / 2f32);
        let center_right_edge = pos2(top_right_pos.x, top_right_pos.y + rect.height() / 2f32);
        let center_top_edge = pos2(top_left_pos.x + rect.width() / 2f32, top_left_pos.y);
        let center_bottom_edge = pos2(bottom_left_pos.x + rect.width() / 2f32, bottom_left_pos.y);

        painter.rect(top_left_pos.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);
        painter.rect(top_right_pos.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);
        painter.rect(bottom_right_pos.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);
        painter.rect(bottom_left_pos.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);

        painter.rect(center_left_edge.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);
        painter.rect(center_right_edge.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);
        painter.rect(center_top_edge.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);
        painter.rect(center_bottom_edge.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);
    }

    fn rectangle(&self,
                        rect: &Rect,
                        fill_color: impl Into<Color32>,
                        stroke: impl Into<Stroke>,
                        stroke_kind: StrokeKind,
                        stroke_type: StrokeType) {

        let stroke = stroke.into();
        let path = match stroke_kind {
            StrokeKind::Middle => {
                let half_stroke_width = stroke.width / 2.0;
                let stroke_rect = rect.expand(half_stroke_width);
                vec![
                    stroke_rect.left_top(),
                    stroke_rect.right_top(),
                    stroke_rect.right_bottom(),
                    stroke_rect.left_bottom(),
                    stroke_rect.left_top(),
                ]
            }
            StrokeKind::Inside => {
                let stroke_width = stroke.width;
                let stroke_rect = rect.expand(-stroke_width);
                vec![
                    stroke_rect.left_top(),
                    stroke_rect.right_top(),
                    stroke_rect.right_bottom(),
                    stroke_rect.left_bottom(),
                    stroke_rect.left_top(),
                ]
            }
            StrokeKind::Outside => {
                let stroke_width = stroke.width;
                let stroke_rect = rect.expand(stroke_width);
                vec![
                    stroke_rect.left_top(),
                    stroke_rect.right_top(),
                    stroke_rect.right_bottom(),
                    stroke_rect.left_bottom(),
                    stroke_rect.left_top(),
                ]
            }
        };

        let shapes = match stroke_type {
            StrokeType::SolidLine => {
                vec![
                    Shape::line(vec![path[0], path[1]], stroke),
                    Shape::line(vec![path[1], path[2]], stroke),
                    Shape::line(vec![path[2], path[3]], stroke),
                    Shape::line(vec![path[3], path[0]], stroke),
                ]
            }
            StrokeType::DashedLine => {
                Shape::dashed_line(&path, stroke, max(5, (stroke.width*2.).ceil() as i32) as f32, max(5, (stroke.width*2.).ceil() as i32) as f32)
            }
            StrokeType::DottedLine => {
                Shape::dotted_line(&path, stroke.color, max(5, (stroke.width*2.).ceil() as i32) as f32, stroke.width/2.)
            }
        };

        self.add(Shape::rect_filled(rect.clone(), 0., fill_color));
        self.add(shapes);
    }
}