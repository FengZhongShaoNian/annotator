use crate::annotator::{Annotation, AnnotatorState, DragAction, HitTarget, StrokeType, ToolType};
use egui::{pos2, vec2, Color32, CornerRadius, CursorIcon, Pos2, Rangef, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, Widget};
use log::debug;
use crate::annotator::cursor::Crosshair;

#[derive(Debug, Copy, Clone)]
pub struct RectangleStyle {
    /// 线条颜色和宽度
    stroke: Stroke,
    /// 线条类型
    stroke_type: StrokeType,
    /// 填充颜色
    fill_color: Option<Color32>,
}

impl Default for RectangleStyle {
    fn default() -> Self {
        Self {
            stroke: Stroke::new(1., Color32::from_rgb(255, 0,0)),
            stroke_type: StrokeType::SolidLine,
            fill_color: None,
        }
    }
}

/// 矩形标注
#[derive(Debug, Clone)]
pub struct RectangleState {
    /// 区域
    rect: Rect,
    /// 样式
    style: RectangleStyle,
    /// 该标注是否处于活动状态
    active: bool,
}

impl RectangleState {
    pub fn new(rect: Rect, style: RectangleStyle, active: bool) -> Self {
        Self { rect, style, active }
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

impl Annotation for RectangleState {
    type State = Self;

    fn show(&self, ui: &mut Ui) -> Response{
        ui.add(self.clone())
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

impl Widget for RectangleState {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.allocate_rect(self.rect, Sense::hover());
        let painter = ui.painter();
        if let Some(fill_color) = self.style.fill_color {
            painter.rect(self.rect, 0, fill_color, self.style.stroke, StrokeKind::Middle);
        } else {
            painter.rect(self.rect, 0, Color32::TRANSPARENT, self.style.stroke, StrokeKind::Middle);
        }

        if self.active {
            // 绘制各个角以及边上的小矩形
            let width = 6f32;
            let height = 6f32;
            let top_left_pos = self.rect.left_top();
            let top_right_pos = self.rect.right_top();
            let bottom_right_pos = self.rect.right_bottom();
            let bottom_left_pos = self.rect.left_bottom();

            let center_left_edge = pos2(top_left_pos.x, top_left_pos.y + self.rect.height() / 2f32);
            let center_right_edge = pos2(top_right_pos.x, top_right_pos.y + self.rect.height() / 2f32);
            let center_top_edge = pos2(top_left_pos.x + self.rect.width() / 2f32, top_left_pos.y);
            let center_bottom_edge = pos2(bottom_left_pos.x + self.rect.width() / 2f32, bottom_left_pos.y);

            painter.rect(top_left_pos.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);
            painter.rect(top_right_pos.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);
            painter.rect(bottom_right_pos.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);
            painter.rect(bottom_left_pos.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);

            painter.rect(center_left_edge.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);
            painter.rect(center_right_edge.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);
            painter.rect(center_top_edge.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);
            painter.rect(center_bottom_edge.rect(width, height), 0, Color32::TRANSPARENT, Stroke::new(1f32, Color32::WHITE), StrokeKind::Middle);

        }
        response
    }
}

pub struct RectangleAnnotationToolState {
    /// 矩形的样式配置
    pub style: RectangleStyle,
    /// 当前的标注
    current_annotation: Option<RectangleState>,
    /// 当拖动鼠标的时候需要执行的操作
    drag_action: DragAction,
}

impl Default for RectangleAnnotationToolState {
    fn default() -> Self {
        Self {
            style: Default::default(),
            current_annotation: None,
            drag_action: DragAction::None,
        }
    }
}

pub struct RectangleAnnotationTool<'a> {
    annotator_state: &'a mut AnnotatorState,
}

impl<'a> RectangleAnnotationTool<'a> {
    pub fn new(
        annotator_state: &'a mut AnnotatorState,
    ) -> Self {
        Self {
            annotator_state,
        }
    }

    fn hit_test(annotation: &Option<&RectangleState>, pos: Pos2) -> Option<HitTarget>{
        match annotation {
            Some(rectangle_state)  => {
                Some(rectangle_state.rect.hit_test(&pos, rectangle_state.style.stroke.width))
            }
            _ => {
                None
            }
        }
    }
}


impl Widget for RectangleAnnotationTool<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let sense_area = Rect::from_min_size(Pos2::ZERO, ui.available_size());
        let response = ui.allocate_rect(sense_area, Sense::click_and_drag());

        let Some(pointer_pos) = ui.ctx().pointer_hover_pos() else {
            return response;
        };

        let annotator_state = self.annotator_state;
        // 从标注栈的栈顶中获取最近的一个矩形标注
        let last_annotation = annotator_state.annotations_stack
            .last()
            .map(|annotation| annotation.downcast_ref::<RectangleState>())
            .flatten();

        { // 检测鼠标碰撞并绘制光标

            // 判断当前鼠标是否位于此矩形标注上
            let hit_target = Self::hit_test(&last_annotation, pointer_pos);

            if let Some(hit_target) = hit_target {
                let cursor_icon = hit_target.get_cursor();
                if let Some(cursor_icon) = cursor_icon {
                    ui.ctx().set_cursor_icon(cursor_icon);
                }else {
                    ui.ctx().set_cursor_icon(CursorIcon::None);
                    // 绘制自定义光标
                    Crosshair::new(pointer_pos, Color32::RED, 1.0).paint_with(ui.painter());
                }
            } else {
                ui.ctx().set_cursor_icon(CursorIcon::None);
                // 绘制自定义光标
                Crosshair::new(pointer_pos, Color32::RED, 1.0).paint_with(ui.painter());
            }
        }


        let tool_state = &mut annotator_state.rectangle_annotation_tool_state;

        if response.drag_started() {
            // 拖动开始
            let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
            let hit_target = Self::hit_test(&last_annotation, drag_started_pos);
            match hit_target {
                Some(hit_target) if hit_target != HitTarget::Inside && hit_target != HitTarget::Outside => {
                    // 调整现有的标注
                    let mut annotation = annotator_state.annotations_stack.pop()
                        .map(|annotation| annotation.downcast::<RectangleState>().ok())
                        .flatten()
                        .map(|state| state.clone())
                        .unwrap();
                    annotation.active = true;
                    tool_state.current_annotation = Some(*annotation);
                    tool_state.drag_action = hit_target.get_drag_action();
                }
                _ => {
                    if last_annotation.is_some() {
                        // 把栈顶的矩形标注设为非激活状态
                        annotator_state.annotations_stack
                            .last_mut()
                            .map(|annotation| {
                                let mut rectangle_state = annotation.downcast_mut::<RectangleState>();
                                if let Some(rectangle_state) =  rectangle_state.as_mut() {
                                    rectangle_state.active = false;
                                }
                            });
                    }
                }
            }
        }

        if response.dragged() {
            // 拖动中
            if let Some(rectangle_state) = tool_state.current_annotation.as_mut() {
                match tool_state.drag_action {
                    DragAction::AdjustTopEdge => {
                        rectangle_state.rect.min.y = pointer_pos.y;
                    }
                    DragAction::AdjustBottomEdge => {
                        rectangle_state.rect.max.y = pointer_pos.y;
                    }
                    DragAction::AdjustLeftEdge => {
                        rectangle_state.rect.min.x = pointer_pos.x;
                    }
                    DragAction::AdjustRightEdge => {
                        rectangle_state.rect.max.x = pointer_pos.x;
                    }
                    DragAction::AdjustTopLeftCorner => {
                        rectangle_state.rect.min = pointer_pos;
                    }
                    DragAction::AdjustTopRightCorner => {
                        rectangle_state.rect.min.y = pointer_pos.y;
                        rectangle_state.rect.max.x = pointer_pos.x;
                    }
                    DragAction::AdjustBottomRightCorner => {
                        rectangle_state.rect.max = pointer_pos;
                    }
                    DragAction::AdjustBottomLeftCorner => {
                        rectangle_state.rect.min.x = pointer_pos.x;
                        rectangle_state.rect.max.y = pointer_pos.y;
                    }

                    DragAction::None => {
                        let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
                        rectangle_state.rect = Rect::from_two_pos(drag_started_pos, pointer_pos);
                    }
                }
                ui.add(rectangle_state.clone());
            }else {
                let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
                let rect = Rect::from_two_pos(drag_started_pos, pointer_pos);
                let rectangle_state = RectangleState::new(rect, tool_state.style, true);
                tool_state.current_annotation = Some(rectangle_state.clone());
                tool_state.drag_action = DragAction::None;
                ui.add(rectangle_state);
            }
        }

        if response.drag_stopped() {
            // 拖动结束
            tool_state.drag_action = DragAction::None;
            let annotation = tool_state.current_annotation.take().unwrap();
            annotator_state.annotations_stack.push(Box::new(annotation));
        }
        response
    }
}


