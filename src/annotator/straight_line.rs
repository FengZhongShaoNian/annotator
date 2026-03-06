use crate::annotator::cursor::Crosshair;
use crate::annotator::{Annotation, AnnotatorState, DEFAULT_SIZE_FOR_SMALL_RECT, DragAction, HitTarget, HitTest, PainterExt, SmallRect, StrokeType, spacing_for_dotted_line, radius_for_dotted_line, dash_len_for_dashed_line, gap_len_for_dashed_line};
use egui::{
    Color32, CursorIcon, Id, Pos2, Rect, Response, Sense, Shape, Stroke, StrokeKind, Ui, Widget,
};
use std::cell::RefCell;
use std::rc::Weak;
use std::time::{Duration, Instant};

#[derive(Debug, Copy, Clone)]
pub struct StraightLineStyle {
    /// 线条颜色和宽度
    pub stroke: Stroke,
    /// 线条类型
    pub stroke_type: StrokeType,
}

impl Default for StraightLineStyle {
    fn default() -> Self {
        Self {
            stroke: Stroke::new(1., Color32::from_rgb(255, 0, 0)),
            stroke_type: StrokeType::SolidLine,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StraightLineState {
    /// 起点
    start_pos: Pos2,
    /// 终点
    end_pos: Pos2,
    /// 样式
    style: StraightLineStyle,
    /// 该标注是否处于活动状态
    active: bool,
}

impl StraightLineState {
    pub fn new(start_pos: Pos2, end_pos: Pos2, style: StraightLineStyle, active: bool) -> Self {
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

    pub fn set_stroke_type(&mut self, stroke_type: StrokeType) {
        self.style.stroke_type = stroke_type;
    }
}

impl Widget for &mut StraightLineState {
    fn ui(self, ui: &mut Ui) -> Response {
        let rect = Rect::from_two_pos(self.start_pos, self.end_pos);
        let response = ui.allocate_rect(rect, Sense::hover());
        let painter = ui.painter();

        match self.style.stroke_type {
            StrokeType::SolidLine => {
                painter.line_segment([self.start_pos, self.end_pos], self.style.stroke);
            }
            StrokeType::DashedLine => {
                let dash_len = dash_len_for_dashed_line(self.style.stroke.width);
                let gap_len = gap_len_for_dashed_line(self.style.stroke.width);
                let shape =
                    Shape::dashed_line(&[self.start_pos, self.end_pos], self.style.stroke, dash_len, gap_len);
                painter.add(shape);
            }
            StrokeType::DottedLine => {
                let spacing = spacing_for_dotted_line(self.style.stroke.width); // 点间距
                let radius = radius_for_dotted_line(self.style.stroke.width); // 点半径
                let shape = Shape::dotted_line(
                    &[self.start_pos, self.end_pos],
                    self.style.stroke.color,
                    spacing,
                    radius,
                );
                painter.add(shape);
            }
        }

        if self.active {
            painter.small_rect(&self.start_pos);
            painter.small_rect(&self.end_pos);
        }
        response
    }
}

pub struct StraightLineToolState {
    /// 线段的样式配置
    pub style: StraightLineStyle,
    /// 当前的标注
    current_annotation: Option<StraightLineState>,
    /// 当拖动鼠标的时候需要执行的操作
    drag_action: DragActionForStraightLine,
}

impl Default for StraightLineToolState {
    fn default() -> Self {
        Self {
            style: Default::default(),
            current_annotation: None,
            drag_action: DragActionForStraightLine::None,
        }
    }
}

pub struct StraightLineTool {
    annotator_state: Weak<RefCell<AnnotatorState>>,
    tool_state: StraightLineToolState,
}

const MAX_STROKE_WIDTH: f32 = 62.;

#[derive(PartialEq)]
enum HitTargetForStraightLine {
    Outside,
    StartPoint,
    EndPoint,
}

fn hit_test(annotation: &Option<&StraightLineState>, pointer_pos: &Pos2) -> Option<HitTargetForStraightLine> {
    match annotation {
        Some(straight_line_state) => {
            let hit_target = straight_line_state
                .start_pos
                .rect(DEFAULT_SIZE_FOR_SMALL_RECT.0, DEFAULT_SIZE_FOR_SMALL_RECT.1)
                .hit_test(pointer_pos, straight_line_state.style.stroke.width);
            if hit_target != HitTarget::Outside {
                return Some(HitTargetForStraightLine::StartPoint);
            }
            let hit_target = straight_line_state
                .end_pos
                .rect(DEFAULT_SIZE_FOR_SMALL_RECT.0, DEFAULT_SIZE_FOR_SMALL_RECT.1)
                .hit_test(pointer_pos, straight_line_state.style.stroke.width);
            if hit_target != HitTarget::Outside {
                Some(HitTargetForStraightLine::EndPoint)
            } else {
                Some(HitTargetForStraightLine::Outside)
            }
        }
        _ => None,
    }
}

enum DragActionForStraightLine {
    AdjustStartPoint,
    AdjustEndPoint,
    None
}

impl HitTargetForStraightLine {
    fn get_drag_action(&self) -> DragActionForStraightLine {
        match self {
            HitTargetForStraightLine::Outside => DragActionForStraightLine::None,
            HitTargetForStraightLine::StartPoint => DragActionForStraightLine::AdjustStartPoint,
            HitTargetForStraightLine::EndPoint => DragActionForStraightLine::AdjustEndPoint,
        }
    }

    fn get_cursor(&self) -> Option<CursorIcon> {
        match self {
            HitTargetForStraightLine::Outside => None,
            HitTargetForStraightLine::StartPoint => Some(CursorIcon::ResizeNwSe),
            HitTargetForStraightLine::EndPoint => Some(CursorIcon::ResizeNwSe),
        }
    }
}

impl StraightLineTool {
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
    }

    fn peek_straight_line_annotation<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&StraightLineState>) -> Option<R>,
    {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        let annotator_state = annotator_state.borrow();
        // 从标注栈的栈顶中获取最近的一个直线标注
        let rectangle_annotation_on_stack_top = annotator_state
            .annotations_stack
            .last()
            .map(|annotation| match annotation {
                Annotation::StraightLine(straight_line_state) => Some(straight_line_state),
                _ => None,
            })
            .flatten();
        func(rectangle_annotation_on_stack_top)
    }

    fn peek_straight_line_annotation_mut<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&mut StraightLineState>) -> Option<R>,
    {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        let mut annotator_state = annotator_state.borrow_mut();
        // 从标注栈的栈顶中获取最近的一个直线标注
        let straight_line_state_annotation_on_stack_top = annotator_state
            .annotations_stack
            .last_mut()
            .map(|annotation| match annotation {
                Annotation::StraightLine(straight_line_state) => Some(straight_line_state),
                _ => None,
            })
            .flatten();
        func(straight_line_state_annotation_on_stack_top)
    }

    fn pop_rectangle_annotation(&self) -> Option<StraightLineState> {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        // 从标注栈的栈顶中获取最近的一个直线标注
        annotator_state
            .borrow_mut()
            .annotations_stack
            .pop()
            .map(|annotation| match annotation {
                Annotation::StraightLine(straight_line_state) => Some(straight_line_state),
                _ => None,
            })
            .flatten()
    }

    fn handle_wheel_event(&mut self, ui: &mut Ui) {
        // 滚动鼠标滚轮调整线条大小
        let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta);
        if scroll_delta != egui::Vec2::ZERO {
            ui.ctx().memory_mut(|memory| {
                let id = Id::from("StraightLineAnnotationTool.wheel.instant");
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
        // 从标注栈的栈顶中获取最近的一个直线标注
        let hit_target = self.peek_straight_line_annotation(|rectangle_annotation_on_stack_top| {
            // 判断当前鼠标是否位于此直线标注上
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

impl Widget for &mut StraightLineTool {
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
            // 从标注栈的栈顶中获取最近的一个直线标注
            let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();

            let hit_target =
                self.peek_straight_line_annotation(|rectangle_annotation_on_stack_top| {
                    // 判断当前鼠标是否位于此直线标注上
                    hit_test(&rectangle_annotation_on_stack_top, &drag_started_pos)
                });

            match hit_target {
                Some(hit_target)
                    if hit_target != HitTargetForStraightLine::Outside =>
                {
                    // 调整现有的标注
                    let mut annotation = self.pop_rectangle_annotation().unwrap();
                    annotation.activate();
                    self.tool_state.current_annotation = Some(annotation);
                    self.tool_state.drag_action = hit_target.get_drag_action();
                }
                Some(hit_target)
                    if hit_target == HitTargetForStraightLine::StartPoint || hit_target == HitTargetForStraightLine::EndPoint =>
                {
                    self.peek_straight_line_annotation_mut(
                        |mut annotation_on_stack_top| {
                            // 把栈顶的直线标注设为非激活状态
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
                        // 把栈顶的直线标注设为非激活状态
                        if let Some(annotation) = annotation_on_stack_top.as_mut() {
                            annotation.deactivate();
                        }

                        None::<()>
                    });
                }
            }
        } else if response.clicked() {
            self.peek_straight_line_annotation_mut(|mut annotation_on_stack_top| {
                // 把栈顶的直线标注设为非激活状态
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
                    DragActionForStraightLine::AdjustStartPoint => {
                        straight_line_state.start_pos = pointer_pos;
                    }
                    DragActionForStraightLine::AdjustEndPoint => {
                        straight_line_state.end_pos = pointer_pos;
                    }
                    DragActionForStraightLine::None => {
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
                    StraightLineState::new(drag_started_pos, pointer_pos, self.tool_state.style, true);
                self.tool_state.current_annotation = Some(straight_line_state.clone());
                self.tool_state.drag_action = DragActionForStraightLine::None;
                ui.add(&mut straight_line_state);
            }
        }

        if response.drag_stopped() {
            // 拖动结束
            self.tool_state.drag_action = DragActionForStraightLine::None;
            let current_annotation = self.tool_state.current_annotation.take().unwrap();
            self.annotator_state
                .upgrade()
                .unwrap()
                .borrow_mut()
                .annotations_stack
                .push(Annotation::StraightLine(current_annotation));
        }
        response
    }
}
