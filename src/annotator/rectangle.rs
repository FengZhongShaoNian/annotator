use crate::annotator::{Annotation, AnnotatorState, DragAction, HitTarget, HitTest, SmallRect, StrokeType, ToolType};
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

    pub fn deactivate(&mut self){
        self.active = false;
    }
}

impl Annotation for RectangleState {
    fn show(&self, ui: &mut Ui) -> Response{
        ui.add(self.clone())
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


