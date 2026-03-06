use crate::annotator::cursor::Crosshair;
use crate::annotator::{
    Annotation, AnnotatorState, DragAction, HitTarget, HitTest, PainterExt, StrokeType,
};
use egui::{Color32, CursorIcon, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Widget};
use std::cell::RefCell;
use std::rc::Weak;
use std::time::{Duration, Instant};

#[derive(Debug, Copy, Clone)]
pub struct RectangleStyle {
    /// 线条颜色和宽度
    pub stroke: Stroke,
    /// 线条类型
    pub stroke_type: StrokeType,
    /// 填充颜色
    pub fill_color: Option<Color32>,
}

impl Default for RectangleStyle {
    fn default() -> Self {
        Self {
            stroke: Stroke::new(1., Color32::from_rgb(255, 0, 0)),
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
        Self {
            rect,
            style,
            active,
        }
    }
    
    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_color(&mut self, color: Color32) {
        self.style.stroke.color = color;
        if let Some(color) = self.style.fill_color {
            self.style.fill_color = Some(color);
        }
    }

    pub fn set_stroke_type(&mut self, stroke_type: StrokeType){
        self.style.stroke_type = stroke_type;
    }
}

impl Widget for &mut RectangleState {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.allocate_rect(self.rect, Sense::hover());
        let painter = ui.painter();
        if let Some(fill_color) = self.style.fill_color {
            painter.rectangle(
                &self.rect,
                fill_color,
                self.style.stroke,
                StrokeKind::Middle,
                self.style.stroke_type,
            );
        } else {
            painter.rectangle(
                &self.rect,
                Color32::TRANSPARENT,
                self.style.stroke,
                StrokeKind::Middle,
                self.style.stroke_type,
            );
        }

        if self.active {
            painter.small_rects(&self.rect);
        }
        response
    }
}

pub struct RectangleToolState {
    /// 矩形的样式配置
    pub style: RectangleStyle,
    /// 当前的标注
    current_annotation: Option<RectangleState>,
    /// 当拖动鼠标的时候需要执行的操作
    drag_action: DragAction,
}

impl Default for RectangleToolState {
    fn default() -> Self {
        Self {
            style: Default::default(),
            current_annotation: None,
            drag_action: DragAction::None,
        }
    }
}

pub struct RectangleTool {
    annotator_state: Weak<RefCell<AnnotatorState>>,
    tool_state: RectangleToolState,
}

const MAX_STROKE_WIDTH: f32 = 62.;

fn hit_test(annotation: &Option<&RectangleState>, pointer_pos: &Pos2) -> Option<HitTarget> {
    match annotation {
        Some(rectangle_state) => Some(
            rectangle_state
                .rect
                .hit_test(&pointer_pos, rectangle_state.style.stroke.width),
        ),
        _ => None,
    }
}

impl RectangleTool {
    pub fn new(annotator_state: Weak<RefCell<AnnotatorState>>) -> Self {
        Self {
            annotator_state,
            tool_state: Default::default(),
        }
    }

    fn increase_stroke(&mut self) {
        let tool_state = &mut self.tool_state;
        if tool_state.style.stroke.width + 1. < MAX_STROKE_WIDTH {
            tool_state.style.stroke.width += 1.;
        }
        let new_width = tool_state.style.stroke.width;
        self.update_stroke_width_for_stack_top_annotation(new_width);
    }

    fn decrease_stroke(&mut self) {
        let tool_state = &mut self.tool_state;
        if tool_state.style.stroke.width - 1. > 0. {
            tool_state.style.stroke.width -= 1.;
        }
        let new_width = tool_state.style.stroke.width;
        self.update_stroke_width_for_stack_top_annotation(new_width);
    }

    fn update_stroke_width_for_stack_top_annotation(&mut self, new_width: f32){
        self.peek_rectangle_annotation_mut(|mut annotation| {
            if let Some(annotation) = annotation.as_mut() {
                if annotation.is_active() {
                    annotation.style.stroke.width = new_width;
                }
            }
            None::<()>
        });
    }

    pub fn stroke_type(&self) -> StrokeType {
        self.tool_state.style.stroke_type
    }

    pub fn set_stroke_type(&mut self, stroke_type: StrokeType) {
        self.tool_state.style.stroke_type = stroke_type;
    }
    
    pub fn color(&self) -> Color32 {
        self.tool_state.style.stroke.color
    }
    
    pub fn set_color(&mut self, color: Color32) {
        self.tool_state.style.stroke.color = color;
        if self.tool_state.style.fill_color.is_some() {
            self.tool_state.style.fill_color = Some(color);
        }
    }

    fn peek_rectangle_annotation<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&RectangleState>) -> Option<R>,
    {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        let annotator_state = annotator_state.borrow();
        // 从标注栈的栈顶中获取最近的一个矩形标注
        let rectangle_annotation_on_stack_top = annotator_state
            .annotations_stack
            .last()
            .map(|annotation| match annotation {
                Annotation::Rectangle(rectangle_state) => Some(rectangle_state),
                _ => None,
            })
            .flatten();
        func(rectangle_annotation_on_stack_top)
    }

    fn peek_rectangle_annotation_mut<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&mut RectangleState>) -> Option<R>,
    {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        let mut annotator_state = annotator_state.borrow_mut();
        // 从标注栈的栈顶中获取最近的一个矩形标注
        let rectangle_annotation_on_stack_top = annotator_state
            .annotations_stack
            .last_mut()
            .map(|annotation| match annotation {
                Annotation::Rectangle(rectangle_state) => Some(rectangle_state),
                _ => None,
            })
            .flatten();
        func(rectangle_annotation_on_stack_top)
    }

    fn pop_rectangle_annotation(&self) -> Option<RectangleState> {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        // 从标注栈的栈顶中获取最近的一个矩形标注
        annotator_state
            .borrow_mut()
            .annotations_stack
            .pop()
            .map(|annotation| match annotation {
                Annotation::Rectangle(rectangle_state) => Some(rectangle_state),
                _ => None,
            })
            .flatten()
    }

    fn handle_wheel_event(&mut self, ui: &mut Ui) {
        // 滚动鼠标滚轮调整线条大小
        let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta);
        if scroll_delta != egui::Vec2::ZERO {
            ui.ctx().memory_mut(|memory| {
                let id = Id::from("RectangleAnnotationTool.wheel.instant");
                let now = Instant::now();
                if let Some(previous_scroll) = memory.data.get_temp::<Instant>(id) {
                    let duration = now.checked_duration_since(previous_scroll);
                    if let Some(duration) = duration {
                        if duration > Duration::from_millis(300) {
                            if scroll_delta.y > 0. {
                                self.decrease_stroke();
                            } else if scroll_delta.y < 0. {
                                self.increase_stroke();
                            }
                            memory.data.insert_temp(id, now);
                        }
                    }
                } else {
                    if scroll_delta.y > 0. {
                        self.decrease_stroke();
                    } else if scroll_delta.y < 0. {
                        self.increase_stroke();
                    }
                    memory.data.insert_temp(id, Instant::now());
                }
            });
        }
    }

    fn update_cursor_icon(&self, ui: &mut Ui) {
        let Some(pointer_pos) = ui.ctx().pointer_hover_pos() else {
            return;
        };
        // 从标注栈的栈顶中获取最近的一个矩形标注
        let hit_target = self.peek_rectangle_annotation(|rectangle_annotation_on_stack_top| {
            // 判断当前鼠标是否位于此矩形标注上
            hit_test(&rectangle_annotation_on_stack_top, &pointer_pos)
        });

        if let Some(hit_target) = hit_target {
            let cursor_icon = hit_target.get_cursor();
            if let Some(cursor_icon) = cursor_icon {
                ui.ctx().set_cursor_icon(cursor_icon);
            } else {
                ui.ctx().set_cursor_icon(CursorIcon::None);
                // 绘制自定义光标
                Crosshair::new(
                    pointer_pos,
                    Color32::RED,
                    self.tool_state.style.stroke.width,
                )
                .paint_with(ui.painter());
            }
        } else {
            ui.ctx().set_cursor_icon(CursorIcon::None);
            // 绘制自定义光标
            Crosshair::new(
                pointer_pos,
                Color32::RED,
                self.tool_state.style.stroke.width,
            )
            .paint_with(ui.painter());
        }
    }
}

impl Widget for &mut RectangleTool {
    fn ui(self, ui: &mut Ui) -> Response {
        let sense_area = Rect::from_min_size(Pos2::ZERO, ui.available_size());
        let response = ui.allocate_rect(sense_area, Sense::click_and_drag());

        let Some(pointer_pos) = ui.ctx().pointer_hover_pos() else {
            return response;
        };

        // 滚动鼠标滚轮调整线条大小
        self.handle_wheel_event(ui);

        // 检测鼠标碰撞并绘制光标
        self.update_cursor_icon(ui);

        if response.drag_started() {
            // 拖动开始
            // 从标注栈的栈顶中获取最近的一个矩形标注
            let drag_started_pos =
                ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
            
            let hit_target = self.peek_rectangle_annotation(|rectangle_annotation_on_stack_top| {
                // 判断当前鼠标是否位于此矩形标注上
                hit_test(&rectangle_annotation_on_stack_top, &drag_started_pos)
            });

            match hit_target {
                Some(hit_target)
                    if hit_target != HitTarget::Inside && hit_target != HitTarget::Outside =>
                {
                    // 调整现有的标注
                    let mut annotation = self.pop_rectangle_annotation().unwrap();
                    annotation.activate();
                    self.tool_state.current_annotation = Some(annotation);
                    self.tool_state.drag_action = hit_target.get_drag_action();
                }
                Some(hit_target)
                    if hit_target == HitTarget::Inside || hit_target == HitTarget::Outside =>
                {
                    self.peek_rectangle_annotation_mut(|mut rectangle_annotation_on_stack_top| {
                        // 把栈顶的矩形标注设为非激活状态
                        rectangle_annotation_on_stack_top.as_mut().unwrap().deactivate();
                        None::<()>
                    });
                }
                _ => {}
            }
        } else if response.clicked() {
            self.peek_rectangle_annotation_mut(|mut rectangle_annotation_on_stack_top| {
                // 把栈顶的矩形标注设为非激活状态
                if let Some(annotation) = rectangle_annotation_on_stack_top.as_mut() {
                    annotation.deactivate();
                }

                None::<()>
            });
        }

        if response.dragged() {
            // 拖动中
            if let Some(rectangle_state) = &mut self.tool_state.current_annotation {
                match self.tool_state.drag_action {
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
                ui.add(rectangle_state);
            } else {
                let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
                let rect = Rect::from_two_pos(drag_started_pos, pointer_pos);
                let mut rectangle_state = RectangleState::new(rect, self.tool_state.style, true);
                self.tool_state.current_annotation = Some(rectangle_state.clone());
                self.tool_state.drag_action = DragAction::None;
                ui.add(&mut rectangle_state);
            }
        }

        if response.drag_stopped() {
            // 拖动结束
            self.tool_state.drag_action = DragAction::None;
            let current_annotation = self.tool_state.current_annotation.take().unwrap();
            self.annotator_state
                .upgrade()
                .unwrap()
                .borrow_mut()
                .annotations_stack
                .push(Annotation::Rectangle(current_annotation));
        }
        response
    }
}
