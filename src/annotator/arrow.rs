use crate::annotator::cursor::Crosshair;
use crate::annotator::{dash_len_for_dashed_line, gap_len_for_dashed_line, radius_for_dotted_line, spacing_for_dotted_line, Annotation, AnnotatorState, HitTarget, HitTest, PainterExt, SmallRect, StrokeType, DEFAULT_SIZE_FOR_SMALL_RECT};
use egui::{vec2, Color32, CursorIcon, Id, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Widget};
use std::cell::RefCell;
use std::rc::Weak;
use std::time::{Duration, Instant};

#[derive(Debug, Copy, Clone)]
pub struct ArrowStyle {
    /// 线条颜色和宽度
    pub stroke: Stroke,
}

impl Default for ArrowStyle {
    fn default() -> Self {
        Self {
            stroke: Stroke::new(1., Color32::from_rgb(255, 0, 0)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArrowState {
    /// 起点
    start_pos: Pos2,
    /// 终点
    end_pos: Pos2,
    /// 样式
    style: ArrowStyle,
    /// 该标注是否处于活动状态
    active: bool,
}

impl ArrowState {
    pub fn new(start_pos: Pos2, end_pos: Pos2, style: ArrowStyle, active: bool) -> Self {
        Self {
            start_pos,
            end_pos,
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
    }

}

impl Widget for &mut ArrowState {
    fn ui(self, ui: &mut Ui) -> Response {
        let rect = Rect::from_two_pos(self.start_pos, self.end_pos);
        let response = ui.allocate_rect(rect, Sense::hover());
        let painter = ui.painter();

        painter.simple_arrow(self.start_pos, vec2(self.end_pos.x - self.start_pos.x, self.end_pos.y - self.start_pos.y), self.style.stroke);

        if self.active {
            painter.small_rect(&self.start_pos);
            painter.small_rect(&self.end_pos);
        }
        response
    }
}

pub struct ArrowToolState {
    /// 线段的样式配置
    pub style: ArrowStyle,
    /// 当前的标注
    current_annotation: Option<ArrowState>,
    /// 当拖动鼠标的时候需要执行的操作
    drag_action: DragActionForArrow,
}

impl Default for ArrowToolState {
    fn default() -> Self {
        Self {
            style: Default::default(),
            current_annotation: None,
            drag_action: DragActionForArrow::None,
        }
    }
}

pub struct ArrowTool {
    annotator_state: Weak<RefCell<AnnotatorState>>,
    tool_state: ArrowToolState,
}

const MAX_STROKE_WIDTH: f32 = 6.;

#[derive(PartialEq)]
enum HitTargetForArrow {
    Outside,
    StartPoint,
    EndPoint,
}

fn hit_test(annotation: &Option<&ArrowState>, pointer_pos: &Pos2) -> Option<HitTargetForArrow> {
    match annotation {
        Some(straight_line_state) => {
            let hit_target = straight_line_state
                .start_pos
                .rect(DEFAULT_SIZE_FOR_SMALL_RECT.0, DEFAULT_SIZE_FOR_SMALL_RECT.1)
                .hit_test(pointer_pos, straight_line_state.style.stroke.width);
            if hit_target != HitTarget::Outside {
                return Some(HitTargetForArrow::StartPoint);
            }
            let hit_target = straight_line_state
                .end_pos
                .rect(DEFAULT_SIZE_FOR_SMALL_RECT.0, DEFAULT_SIZE_FOR_SMALL_RECT.1)
                .hit_test(pointer_pos, straight_line_state.style.stroke.width);
            if hit_target != HitTarget::Outside {
                Some(HitTargetForArrow::EndPoint)
            } else {
                Some(HitTargetForArrow::Outside)
            }
        }
        _ => None,
    }
}

enum DragActionForArrow {
    AdjustStartPoint,
    AdjustEndPoint,
    None
}

impl HitTargetForArrow {
    fn get_drag_action(&self) -> DragActionForArrow {
        match self {
            HitTargetForArrow::Outside => DragActionForArrow::None,
            HitTargetForArrow::StartPoint => DragActionForArrow::AdjustStartPoint,
            HitTargetForArrow::EndPoint => DragActionForArrow::AdjustEndPoint,
        }
    }

    fn get_cursor(&self) -> Option<CursorIcon> {
        match self {
            HitTargetForArrow::Outside => None,
            HitTargetForArrow::StartPoint => Some(CursorIcon::ResizeNwSe),
            HitTargetForArrow::EndPoint => Some(CursorIcon::ResizeNwSe),
        }
    }
}

impl ArrowTool {
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

    fn update_stroke_width_for_stack_top_annotation(&mut self, new_width: f32) {
        self.peek_straight_line_annotation_mut(|mut annotation| {
            if let Some(annotation) = annotation.as_mut() {
                if annotation.is_active() {
                    annotation.style.stroke.width = new_width;
                }
            }
            None::<()>
        });
    }
    pub fn color(&self) -> Color32 {
        self.tool_state.style.stroke.color
    }

    pub fn set_color(&mut self, color: Color32) {
        self.tool_state.style.stroke.color = color;
    }

    fn peek_straight_line_annotation<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&ArrowState>) -> Option<R>,
    {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        let annotator_state = annotator_state.borrow();
        // 从标注栈的栈顶中获取最近的一个箭头标注
        let rectangle_annotation_on_stack_top = annotator_state
            .annotations_stack
            .last()
            .map(|annotation| match annotation {
                Annotation::Arrow(straight_line_state) => Some(straight_line_state),
                _ => None,
            })
            .flatten();
        func(rectangle_annotation_on_stack_top)
    }

    fn peek_straight_line_annotation_mut<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&mut ArrowState>) -> Option<R>,
    {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        let mut annotator_state = annotator_state.borrow_mut();
        // 从标注栈的栈顶中获取最近的一个箭头标注
        let straight_line_state_annotation_on_stack_top = annotator_state
            .annotations_stack
            .last_mut()
            .map(|annotation| match annotation {
                Annotation::Arrow(straight_line_state) => Some(straight_line_state),
                _ => None,
            })
            .flatten();
        func(straight_line_state_annotation_on_stack_top)
    }

    fn pop_rectangle_annotation(&self) -> Option<ArrowState> {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        // 从标注栈的栈顶中获取最近的一个箭头标注
        annotator_state
            .borrow_mut()
            .annotations_stack
            .pop()
            .map(|annotation| match annotation {
                Annotation::Arrow(straight_line_state) => Some(straight_line_state),
                _ => None,
            })
            .flatten()
    }

    fn handle_wheel_event(&mut self, ui: &mut Ui) {
        // 滚动鼠标滚轮调整线条大小
        let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta);
        if scroll_delta != egui::Vec2::ZERO {
            ui.ctx().memory_mut(|memory| {
                let id = Id::from("ArrowAnnotationTool.wheel.instant");
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
        // 从标注栈的栈顶中获取最近的一个箭头标注
        let hit_target = self.peek_straight_line_annotation(|rectangle_annotation_on_stack_top| {
            // 判断当前鼠标是否位于此箭头标注上
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

impl Widget for &mut ArrowTool {
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
            // 从标注栈的栈顶中获取最近的一个箭头标注
            let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();

            let hit_target =
                self.peek_straight_line_annotation(|rectangle_annotation_on_stack_top| {
                    // 判断当前鼠标是否位于此箭头标注上
                    hit_test(&rectangle_annotation_on_stack_top, &drag_started_pos)
                });

            match hit_target {
                Some(hit_target)
                if hit_target != HitTargetForArrow::Outside =>
                    {
                        // 调整现有的标注
                        let mut annotation = self.pop_rectangle_annotation().unwrap();
                        annotation.activate();
                        self.tool_state.current_annotation = Some(annotation);
                        self.tool_state.drag_action = hit_target.get_drag_action();
                    }
                Some(hit_target)
                if hit_target == HitTargetForArrow::StartPoint || hit_target == HitTargetForArrow::EndPoint =>
                    {
                        self.peek_straight_line_annotation_mut(
                            |mut annotation_on_stack_top| {
                                // 把栈顶的箭头标注设为非激活状态
                                annotation_on_stack_top
                                    .as_mut()
                                    .unwrap()
                                    .deactivate();
                                None::<()>
                            },
                        );
                    }
                _ => {
                    self.peek_straight_line_annotation_mut(|mut annotation_on_stack_top| {
                        // 把栈顶的箭头标注设为非激活状态
                        if let Some(annotation) = annotation_on_stack_top.as_mut() {
                            annotation.deactivate();
                        }

                        None::<()>
                    });
                }
            }
        } else if response.clicked() {
            self.peek_straight_line_annotation_mut(|mut annotation_on_stack_top| {
                // 把栈顶的箭头标注设为非激活状态
                if let Some(annotation) = annotation_on_stack_top.as_mut() {
                    annotation.deactivate();
                }

                None::<()>
            });
        }

        if response.dragged() {
            // 拖动中
            if let Some(straight_line_state) = &mut self.tool_state.current_annotation {
                match self.tool_state.drag_action {
                    DragActionForArrow::AdjustStartPoint => {
                        straight_line_state.start_pos = pointer_pos;
                    }
                    DragActionForArrow::AdjustEndPoint => {
                        straight_line_state.end_pos = pointer_pos;
                    }
                    DragActionForArrow::None => {
                        let drag_started_pos =
                            ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
                        straight_line_state.start_pos = drag_started_pos;
                        straight_line_state.end_pos = pointer_pos;
                    }
                }
                ui.add(straight_line_state);
            } else {
                let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
                let mut straight_line_state =
                    ArrowState::new(drag_started_pos, pointer_pos, self.tool_state.style, true);
                self.tool_state.current_annotation = Some(straight_line_state.clone());
                self.tool_state.drag_action = DragActionForArrow::None;
                ui.add(&mut straight_line_state);
            }
        }

        if response.drag_stopped() {
            // 拖动结束
            self.tool_state.drag_action = DragActionForArrow::None;
            let current_annotation = self.tool_state.current_annotation.take().unwrap();
            self.annotator_state
                .upgrade()
                .unwrap()
                .borrow_mut()
                .annotations_stack
                .push(Annotation::Arrow(current_annotation));
        }
        response
    }
}
