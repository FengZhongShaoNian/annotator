use crate::annotator::cursor::Crosshair;
use crate::annotator::{Annotation, AnnotatorState, DragAction, HitTarget, HitTest, PainterExt};
use egui::epaint::EllipseShape;
use egui::{vec2, Color32, CursorIcon, Id, Pos2, Rect, Response, Sense, Stroke, Ui, Widget};
use std::cell::RefCell;
use std::rc::Weak;
use std::time::{Duration, Instant};

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

impl Widget for &mut EllipseState {
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

pub struct EllipseToolState {
    /// 矩形的样式配置
    pub style: EllipseStyle,
    /// 当前的标注
    current_annotation: Option<EllipseState>,
    /// 当拖动鼠标的时候需要执行的操作
    drag_action: DragAction,
}

impl Default for EllipseToolState {
    fn default() -> Self {
        Self {
            style: Default::default(),
            current_annotation: None,
            drag_action: DragAction::None,
        }
    }
}

pub struct EllipseTool {
    annotator_state: Weak<RefCell<AnnotatorState>>,
    tool_state: EllipseToolState,
}


const MAX_STROKE_WIDTH: f32 = 62.;

fn hit_test(annotation: &Option<&EllipseState>, pointer_pos: &Pos2) -> Option<HitTarget> {
    match annotation {
        Some(ellipse_state) => Some(
            ellipse_state
                .rect
                .hit_test(&pointer_pos, ellipse_state.style.stroke.width),
        ),
        _ => None,
    }
}

impl EllipseTool {
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
    }

    fn decrease_stroke(&mut self) {
        let tool_state = &mut self.tool_state;
        if tool_state.style.stroke.width - 1. > 0. {
            tool_state.style.stroke.width -= 1.;
        }
    }

    fn peek_ellipse_annotation<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&EllipseState>) -> Option<R>,
    {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        let annotator_state = annotator_state.borrow();
        // 从标注栈的栈顶中获取最近的一个椭圆标注
        let ellipse_annotation_on_stack_top = annotator_state
            .annotations_stack
            .last()
            .map(|annotation| match annotation {
                Annotation::Ellipse(ellipse_state) => Some(ellipse_state),
                _ => None,
            })
            .flatten();
        func(ellipse_annotation_on_stack_top)
    }

    fn peek_ellipse_annotation_mut<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&mut EllipseState>) -> Option<R>,
    {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        let mut annotator_state = annotator_state.borrow_mut();
        // 从标注栈的栈顶中获取最近的一个椭圆标注
        let ellipse_annotation_on_stack_top = annotator_state
            .annotations_stack
            .last_mut()
            .map(|annotation| match annotation {
                Annotation::Ellipse(ellipse_state) => Some(ellipse_state),
                _ => None,
            })
            .flatten();
        func(ellipse_annotation_on_stack_top)
    }

    fn pop_rectangle_annotation(&self) -> Option<EllipseState> {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        // 从标注栈的栈顶中获取最近的一个椭圆标注
        annotator_state
            .borrow_mut()
            .annotations_stack
            .pop()
            .map(|annotation| match annotation {
                Annotation::Ellipse(ellipse_state) => Some(ellipse_state),
                _ => None,
            })
            .flatten()
    }

    fn handle_wheel_event(&mut self, ui: &mut Ui) {
        // 滚动鼠标滚轮调整线条大小
        let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta);
        if scroll_delta != egui::Vec2::ZERO {
            ui.ctx().memory_mut(|memory| {
                let id = Id::from("EllipseAnnotationTool.wheel.instant");
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
        // 从标注栈的栈顶中获取最近的一个椭圆标注
        let hit_target = self.peek_ellipse_annotation(|ellipse_annotation_on_stack_top| {
            // 判断当前鼠标是否位于此椭圆标注上
            hit_test(&ellipse_annotation_on_stack_top, &pointer_pos)
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

impl Widget for &mut EllipseTool {
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
            // 从标注栈的栈顶中获取最近的一个椭圆标注

            let drag_started_pos =
                ui.ctx().input(|i| i.pointer.press_origin()).unwrap();

            let hit_target = self.peek_ellipse_annotation(|ellipse_annotation_on_stack_top| {
                // 判断当前鼠标是否位于此椭圆标注上
                hit_test(&ellipse_annotation_on_stack_top, &drag_started_pos)
            });

            match hit_target {
                Some(hit_target)
                if hit_target != HitTarget::Inside && hit_target != HitTarget::Outside =>
                    {
                        // 调整现有的标注
                        let mut annotation = self.pop_rectangle_annotation().unwrap();
                        annotation.active = true;
                        self.tool_state.current_annotation = Some(annotation);
                        self.tool_state.drag_action = hit_target.get_drag_action();
                    }
                Some(hit_target)
                if hit_target == HitTarget::Inside || hit_target == HitTarget::Outside =>
                    {
                        self.peek_ellipse_annotation_mut(|mut ellipse_annotation_on_stack_top| {
                            // 把栈顶的椭圆标注设为非激活状态
                            ellipse_annotation_on_stack_top.as_mut().unwrap().active = false;
                            None::<()>
                        });
                    }
                _ => {}
            }
        } else if response.clicked() {
            self.peek_ellipse_annotation_mut(|mut ellipse_annotation_on_stack_top| {
                // 把栈顶的椭圆标注设为非激活状态
                ellipse_annotation_on_stack_top.as_mut().unwrap().active = false;
                None::<()>
            });
        }

        if response.dragged() {
            // 拖动中
            if let Some(ellipse_state) = &mut self.tool_state.current_annotation {
                match self.tool_state.drag_action {
                    DragAction::AdjustTopEdge => {
                        ellipse_state.rect.min.y = pointer_pos.y;
                    }
                    DragAction::AdjustBottomEdge => {
                        ellipse_state.rect.max.y = pointer_pos.y;
                    }
                    DragAction::AdjustLeftEdge => {
                        ellipse_state.rect.min.x = pointer_pos.x;
                    }
                    DragAction::AdjustRightEdge => {
                        ellipse_state.rect.max.x = pointer_pos.x;
                    }
                    DragAction::AdjustTopLeftCorner => {
                        ellipse_state.rect.min = pointer_pos;
                    }
                    DragAction::AdjustTopRightCorner => {
                        ellipse_state.rect.min.y = pointer_pos.y;
                        ellipse_state.rect.max.x = pointer_pos.x;
                    }
                    DragAction::AdjustBottomRightCorner => {
                        ellipse_state.rect.max = pointer_pos;
                    }
                    DragAction::AdjustBottomLeftCorner => {
                        ellipse_state.rect.min.x = pointer_pos.x;
                        ellipse_state.rect.max.y = pointer_pos.y;
                    }

                    DragAction::None => {
                        let drag_started_pos =
                            ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
                        ellipse_state.rect = Rect::from_two_pos(drag_started_pos, pointer_pos);
                    }
                }
                ui.add(ellipse_state);
            } else {
                let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
                let rect = Rect::from_two_pos(drag_started_pos, pointer_pos);
                let mut ellipse_state = EllipseState::new(rect, self.tool_state.style, true);
                self.tool_state.current_annotation = Some(ellipse_state.clone());
                self.tool_state.drag_action = DragAction::None;
                ui.add(&mut ellipse_state);
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
                .push(Annotation::Ellipse(current_annotation));
        }
        response
    }
}
