use crate::annotator::cursor::Crosshair;
use crate::annotator::{Annotation, AnnotatorState, DragAction, HitTarget, HitTest, PainterExt, SmallRect, StrokeType, ToolType};
use egui::epaint::{EllipseShape, RectShape};
use egui::{
    Color32, CornerRadius, CursorIcon, Pos2, Rangef, Rect, Response, Sense, Shape, Stroke,
    StrokeKind, Ui, Widget, pos2, vec2,
};
use log::debug;
use std::ops::Add;

#[derive(Debug, Copy, Clone)]
pub struct EllipseStyle {
    /// 线条颜色和宽度
    stroke: Stroke,
    /// 填充颜色
    fill_color: Option<Color32>,
}

impl Default for EllipseStyle {
    fn default() -> Self {
        Self {
            stroke: Stroke::new(1., Color32::from_rgb(255, 0, 0)),
            fill_color: None,
        }
    }
}

/// 椭圆标注
#[derive(Debug, Clone)]
pub struct EllipseState {
    /// 区域
    rect: Rect,
    /// 样式
    style: EllipseStyle,
    /// 该标注是否处于活动状态
    active: bool,
}

impl EllipseState {
    pub fn new(rect: Rect, style: EllipseStyle, active: bool) -> Self {
        Self {
            rect,
            style,
            active,
        }
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

impl Annotation for EllipseState {
    fn show(&self, ui: &mut Ui) -> Response {
        ui.add(self.clone())
    }
}

impl Widget for EllipseState {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.allocate_rect(self.rect, Sense::hover());
        let painter = ui.painter();
        let fill = if let Some(fill_color) = self.style.fill_color {
            fill_color
        } else {
            Color32::TRANSPARENT
        };

        let ellipse_shape = EllipseShape {
            center: self.rect.center(),
            fill,
            stroke: self.style.stroke,
            radius: vec2(self.rect.width() / 2., self.rect.height() / 2.),
        };

        painter.add(ellipse_shape);

        if self.active {
            // 绘制虚线矩形框以及外框上的各个角以及边上的小矩形
            painter.small_rects(&self.rect);
        }
        response
    }
}

pub struct EllipseAnnotationToolState {
    /// 矩形的样式配置
    pub style: EllipseStyle,
    /// 当前的标注
    current_annotation: Option<EllipseState>,
    /// 当拖动鼠标的时候需要执行的操作
    drag_action: DragAction,
}

impl Default for EllipseAnnotationToolState {
    fn default() -> Self {
        Self {
            style: Default::default(),
            current_annotation: None,
            drag_action: DragAction::None,
        }
    }
}

pub struct EllipseAnnotationTool<'a> {
    annotator_state: &'a mut AnnotatorState,
}

impl<'a> EllipseAnnotationTool<'a> {
    pub fn new(annotator_state: &'a mut AnnotatorState) -> Self {
        Self { annotator_state }
    }

    fn hit_test(annotation: &Option<&EllipseState>, pos: Pos2) -> Option<HitTarget> {
        match annotation {
            Some(rectangle_state) => Some(
                rectangle_state
                    .rect
                    .hit_test(&pos, rectangle_state.style.stroke.width),
            ),
            _ => None,
        }
    }
}

impl Widget for EllipseAnnotationTool<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let sense_area = Rect::from_min_size(Pos2::ZERO, ui.available_size());
        let response = ui.allocate_rect(sense_area, Sense::click_and_drag());

        let Some(pointer_pos) = ui.ctx().pointer_hover_pos() else {
            return response;
        };

        let annotator_state = self.annotator_state;
        // 从标注栈的栈顶中获取最近的一个椭圆标注
        let last_annotation = annotator_state
            .annotations_stack
            .last()
            .map(|annotation| annotation.downcast_ref::<EllipseState>())
            .flatten();

        {
            // 检测鼠标碰撞并绘制光标

            // 判断当前鼠标是否位于此椭圆标注上
            let hit_target = Self::hit_test(&last_annotation, pointer_pos);

            if let Some(hit_target) = hit_target {
                let cursor_icon = hit_target.get_cursor();
                if let Some(cursor_icon) = cursor_icon {
                    ui.ctx().set_cursor_icon(cursor_icon);
                } else {
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

        let tool_state = &mut annotator_state.ellipse_annotation_tool_state;

        if response.drag_started() {
            // 拖动开始
            let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
            let hit_target = Self::hit_test(&last_annotation, drag_started_pos);
            match hit_target {
                Some(hit_target)
                    if hit_target != HitTarget::Inside && hit_target != HitTarget::Outside =>
                {
                    // 调整现有的标注
                    let mut annotation = annotator_state
                        .annotations_stack
                        .pop()
                        .map(|annotation| annotation.downcast::<EllipseState>().ok())
                        .flatten()
                        .map(|state| state.clone())
                        .unwrap();
                    annotation.active = true;
                    tool_state.current_annotation = Some(*annotation);
                    tool_state.drag_action = hit_target.get_drag_action();
                }
                _ => {
                    if last_annotation.is_some() {
                        // 把栈顶的椭圆标注设为非激活状态
                        annotator_state
                            .annotations_stack
                            .last_mut()
                            .map(|annotation| {
                                let mut rectangle_state = annotation.downcast_mut::<EllipseState>();
                                if let Some(rectangle_state) = rectangle_state.as_mut() {
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
                        let drag_started_pos =
                            ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
                        rectangle_state.rect = Rect::from_two_pos(drag_started_pos, pointer_pos);
                    }
                }
                ui.add(rectangle_state.clone());
            } else {
                let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
                let rect = Rect::from_two_pos(drag_started_pos, pointer_pos);
                let rectangle_state = EllipseState::new(rect, tool_state.style, true);
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
