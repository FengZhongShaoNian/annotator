use crate::annotator::cursor::Crosshair;
use crate::annotator::rectangle_based::{EllipseTool, HitTarget, HitTest, RectangleTool};
use crate::annotator::{
    ActivationState, ActivationSupport, Annotation, AnnotationCommon, AnnotationStyle,
    AnnotationToolCommon, WheelHandler, AnnotatorState, DEFAULT_SIZE_FOR_SMALL_RECT,
    FillColorSupport, PainterExt, SharedAnnotatorState, SmallRect, StackTopAccessor,
    StrokeColorSupport, StrokeType, StrokeTypeSupport, StrokeWidthSupport,
    dash_len_for_dashed_line, gap_len_for_dashed_line, radius_for_dotted_line,
    spacing_for_dotted_line,
};
use crate::{impl_stack_top_access_for, impl_stroke_width_handler_for};
use egui::{Color32, CursorIcon, Pos2, Rect, Response, Sense, Shape, Stroke, Ui, Widget, vec2};
use std::cell::RefCell;
use std::rc::Weak;

#[derive(Debug, Copy, Clone)]
pub struct StraightLineStyle {
    /// 线条颜色和宽度
    stroke: Stroke,
    /// 线条类型
    stroke_type: StrokeType,
}

impl StrokeWidthSupport for StraightLineStyle {
    fn supports_get_stroke_width(&self) -> bool {
        true
    }

    fn stroke_width(&self) -> f32 {
        self.stroke.width
    }

    fn supports_set_stroke_width(&self) -> bool {
        true
    }

    fn set_stroke_width(&mut self, stroke_width: f32) {
        self.stroke.width = stroke_width;
    }
}

impl StrokeColorSupport for StraightLineStyle {
    fn supports_get_stroke_color(&self) -> bool {
        true
    }

    fn stroke_color(&self) -> Color32 {
        self.stroke.color
    }

    fn supports_set_stroke_color(&self) -> bool {
        true
    }

    fn set_stroke_color(&mut self, color: Color32) {
        self.stroke.color = color;
    }
}

impl StrokeTypeSupport for StraightLineStyle {
    fn supports_get_stroke_type(&self) -> bool {
        true
    }

    fn stroke_type(&self) -> StrokeType {
        self.stroke_type
    }

    fn supports_set_stroke_type(&self) -> bool {
        true
    }

    fn set_stroke_type(&mut self, stroke_type: StrokeType) {
        self.stroke_type = stroke_type;
    }
}

impl FillColorSupport for StraightLineStyle {
    fn supports_get_fill_color(&self) -> bool {
        false
    }

    fn fill_color(&self) -> Option<Color32> {
        None
    }

    fn supports_set_fill_color(&self) -> bool {
        false
    }

    fn set_fill_color(&mut self, _color: Color32) {
        unimplemented!()
    }
}

impl AnnotationStyle for StraightLineStyle {}

impl Default for StraightLineStyle {
    fn default() -> Self {
        Self {
            stroke: Stroke::new(1., Color32::from_rgb(255, 0, 0)),
            stroke_type: StrokeType::SolidLine,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ArrowStyle {
    /// 线条颜色和宽度
    pub stroke: Stroke,
    /// 线条类型
    pub stroke_type: StrokeType,
}

impl StrokeWidthSupport for ArrowStyle {
    fn supports_get_stroke_width(&self) -> bool {
        true
    }

    fn stroke_width(&self) -> f32 {
        self.stroke.width
    }

    fn supports_set_stroke_width(&self) -> bool {
        true
    }

    fn set_stroke_width(&mut self, stroke_width: f32) {
        self.stroke.width = stroke_width;
    }
}

impl StrokeColorSupport for ArrowStyle {
    fn supports_get_stroke_color(&self) -> bool {
        true
    }

    fn stroke_color(&self) -> Color32 {
        self.stroke.color
    }

    fn supports_set_stroke_color(&self) -> bool {
        true
    }

    fn set_stroke_color(&mut self, color: Color32) {
        self.stroke.color = color;
    }
}

impl StrokeTypeSupport for ArrowStyle {
    fn supports_get_stroke_type(&self) -> bool {
        true
    }

    fn stroke_type(&self) -> StrokeType {
        self.stroke_type
    }

    fn supports_set_stroke_type(&self) -> bool {
        false
    }

    fn set_stroke_type(&mut self, _stroke_type: StrokeType) {
        unimplemented!()
    }
}

impl FillColorSupport for ArrowStyle {
    fn supports_get_fill_color(&self) -> bool {
        false
    }

    fn fill_color(&self) -> Option<Color32> {
        None
    }

    fn supports_set_fill_color(&self) -> bool {
        false
    }

    fn set_fill_color(&mut self, _color: Color32) {
        unimplemented!()
    }
}

impl AnnotationStyle for ArrowStyle {}

impl Default for ArrowStyle {
    fn default() -> Self {
        Self {
            stroke: Stroke::new(1., Color32::from_rgb(255, 0, 0)),
            stroke_type: StrokeType::SolidLine,
        }
    }
}

/// 基于矩形的标注
#[derive(Debug, Clone)]
pub struct StraightLineBasedAnnotation<S>
where
    S: AnnotationStyle,
{
    /// 起点
    start_position: Pos2,
    /// 终点
    end_position: Pos2,
    /// 样式
    style: S,
    /// 激活状态
    activation: ActivationSupport,
}

impl<S> StraightLineBasedAnnotation<S>
where
    S: AnnotationStyle,
{
    pub fn new(
        start_position: Pos2,
        end_position: Pos2,
        style: S,
        activation: ActivationSupport,
    ) -> Self {
        Self {
            start_position,
            end_position,
            style,
            activation,
        }
    }

    pub fn start_position(&self) -> &Pos2 {
        &self.start_position
    }

    pub fn end_position(&self) -> &Pos2 {
        &self.end_position
    }

    pub fn hit_test(&self, pointer_position: &Pos2) -> HitTargetForStraightLine {
        let stroke_width = self.stroke_width();
        let start_position = self.start_position();
        let hit_target = start_position
            .rect(DEFAULT_SIZE_FOR_SMALL_RECT.0, DEFAULT_SIZE_FOR_SMALL_RECT.1)
            .hit_test(&pointer_position, stroke_width);
        if hit_target != HitTarget::Outside {
            return HitTargetForStraightLine::StartPoint;
        }
        let end_position = self.end_position();
        let hit_target = end_position
            .rect(DEFAULT_SIZE_FOR_SMALL_RECT.0, DEFAULT_SIZE_FOR_SMALL_RECT.1)
            .hit_test(&pointer_position, stroke_width);
        if hit_target != HitTarget::Outside {
            return HitTargetForStraightLine::EndPoint;
        }
        HitTargetForStraightLine::Outside
    }
}

impl<S> StrokeWidthSupport for StraightLineBasedAnnotation<S>
where
    S: AnnotationStyle,
{
    fn supports_get_stroke_width(&self) -> bool {
        self.style.supports_get_stroke_width()
    }

    fn stroke_width(&self) -> f32 {
        self.style.stroke_width()
    }

    fn supports_set_stroke_width(&self) -> bool {
        self.style.supports_set_stroke_width()
    }

    fn set_stroke_width(&mut self, stroke_width: f32) {
        self.style.set_stroke_width(stroke_width)
    }
}

impl<S> StrokeColorSupport for StraightLineBasedAnnotation<S>
where
    S: AnnotationStyle,
{
    fn supports_get_stroke_color(&self) -> bool {
        self.style.supports_get_stroke_color()
    }

    fn stroke_color(&self) -> Color32 {
        self.style.stroke_color()
    }

    fn supports_set_stroke_color(&self) -> bool {
        self.style.supports_set_stroke_color()
    }

    fn set_stroke_color(&mut self, color: Color32) {
        self.style.set_stroke_color(color);
    }
}

impl<S> StrokeTypeSupport for StraightLineBasedAnnotation<S>
where
    S: AnnotationStyle,
{
    fn supports_get_stroke_type(&self) -> bool {
        self.style.supports_get_stroke_type()
    }

    fn stroke_type(&self) -> StrokeType {
        self.style.stroke_type()
    }

    fn supports_set_stroke_type(&self) -> bool {
        self.style.supports_set_stroke_type()
    }

    fn set_stroke_type(&mut self, stroke_type: StrokeType) {
        self.style.set_stroke_type(stroke_type);
    }
}

impl<S> FillColorSupport for StraightLineBasedAnnotation<S>
where
    S: AnnotationStyle,
{
    fn supports_get_fill_color(&self) -> bool {
        self.style.supports_get_fill_color()
    }

    fn fill_color(&self) -> Option<Color32> {
        self.style.fill_color()
    }

    fn supports_set_fill_color(&self) -> bool {
        self.style.supports_set_fill_color()
    }

    fn set_fill_color(&mut self, color: Color32) {
        self.style.set_fill_color(color);
    }
}

impl<S> AnnotationCommon for StraightLineBasedAnnotation<S>
where
    S: AnnotationStyle,
{
    fn activation(&self) -> &ActivationSupport {
        &self.activation
    }

    fn activation_mut(&mut self) -> &mut ActivationSupport {
        &mut self.activation
    }
}

pub type StraightLineAnnotation = StraightLineBasedAnnotation<StraightLineStyle>;
pub type ArrowAnnotation = StraightLineBasedAnnotation<ArrowStyle>;

impl Into<Annotation> for StraightLineAnnotation {
    fn into(self) -> Annotation {
        Annotation::StraightLine(self)
    }
}

impl Into<Annotation> for ArrowAnnotation {
    fn into(self) -> Annotation {
        Annotation::Arrow(self)
    }
}

impl Widget for &mut StraightLineAnnotation {
    fn ui(self, ui: &mut Ui) -> Response {
        let rect = Rect::from_two_pos(self.start_position, self.end_position);
        let response = ui.allocate_rect(rect, Sense::hover());
        let painter = ui.painter();

        match self.stroke_type() {
            StrokeType::SolidLine => {
                painter.line_segment([self.start_position, self.end_position], self.style.stroke);
            }
            StrokeType::DashedLine => {
                let dash_len = dash_len_for_dashed_line(self.style.stroke.width);
                let gap_len = gap_len_for_dashed_line(self.style.stroke.width);
                let shape = Shape::dashed_line(
                    &[self.start_position, self.end_position],
                    self.style.stroke,
                    dash_len,
                    gap_len,
                );
                painter.add(shape);
            }
            StrokeType::DottedLine => {
                let spacing = spacing_for_dotted_line(self.style.stroke.width); // 点间距
                let radius = radius_for_dotted_line(self.style.stroke.width); // 点半径
                let shape = Shape::dotted_line(
                    &[self.start_position, self.end_position],
                    self.style.stroke.color,
                    spacing,
                    radius,
                );
                painter.add(shape);
            }
        }

        if self.activation.is_active() {
            painter.small_rect(&self.start_position);
            painter.small_rect(&self.end_position);
        }
        response
    }
}

impl Widget for &mut ArrowAnnotation {
    fn ui(self, ui: &mut Ui) -> Response {
        let rect = Rect::from_two_pos(self.start_position, self.end_position);
        let response = ui.allocate_rect(rect, Sense::hover());
        let painter = ui.painter();

        painter.simple_arrow(
            self.start_position,
            vec2(
                self.end_position.x - self.start_position.x,
                self.end_position.y - self.start_position.y,
            ),
            self.style.stroke,
        );

        if self.activation().is_active() {
            painter.small_rect(&self.start_position);
            painter.small_rect(&self.end_position);
        }
        response
    }
}

pub struct StraightLineBasedToolState<S>
where
    S: AnnotationStyle + Default,
{
    /// 样式
    style: S,
    /// 当前的标注
    current_annotation: Option<StraightLineBasedAnnotation<S>>,
    /// 当拖动鼠标的时候需要执行的操作
    drag_action: DragAction,
}

impl<S> Default for StraightLineBasedToolState<S>
where
    S: AnnotationStyle + Default,
{
    fn default() -> Self {
        Self {
            style: Default::default(),
            current_annotation: None,
            drag_action: DragAction::None,
        }
    }
}

const NOTHING: Option<()> = None::<()>;

impl StackTopAccessor<StraightLineAnnotation> for AnnotatorState {
    fn peek_annotation<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&StraightLineAnnotation>) -> Option<R>,
    {
        // 从标注栈的栈顶中获取最近的一个直线标注
        let straight_line_annotation_on_stack_top = self
            .annotations_stack
            .last()
            .map(|annotation| match annotation {
                Annotation::StraightLine(straight_line_annotation) => {
                    Some(straight_line_annotation)
                }
                _ => None,
            })
            .flatten();
        func(straight_line_annotation_on_stack_top)
    }

    fn peek_annotation_mut<F, R>(&mut self, func: F) -> Option<R>
    where
        F: Fn(Option<&mut StraightLineAnnotation>) -> Option<R>,
    {
        // 从标注栈的栈顶中获取最近的一个直线标注
        let straight_line_annotation_on_stack_top = self
            .annotations_stack
            .last_mut()
            .map(|annotation| match annotation {
                Annotation::StraightLine(straight_line_annotation) => {
                    Some(straight_line_annotation)
                }
                _ => None,
            })
            .flatten();
        func(straight_line_annotation_on_stack_top)
    }

    fn pop_annotation(&mut self) -> Option<StraightLineAnnotation> {
        // 从标注栈的栈顶中获取最近的一个直线标注
        self.annotations_stack
            .pop()
            .map(|annotation| match annotation {
                Annotation::StraightLine(straight_line_annotation) => {
                    Some(straight_line_annotation)
                }
                _ => None,
            })
            .flatten()
    }
}

impl StackTopAccessor<ArrowAnnotation> for AnnotatorState {
    fn peek_annotation<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&ArrowAnnotation>) -> Option<R>,
    {
        // 从标注栈的栈顶中获取最近的一个箭头标注
        let arrow_annotation_on_stack_top = self
            .annotations_stack
            .last()
            .map(|annotation| match annotation {
                Annotation::Arrow(arrow_annotation) => Some(arrow_annotation),
                _ => None,
            })
            .flatten();
        func(arrow_annotation_on_stack_top)
    }

    fn peek_annotation_mut<F, R>(&mut self, func: F) -> Option<R>
    where
        F: Fn(Option<&mut ArrowAnnotation>) -> Option<R>,
    {
        // 从标注栈的栈顶中获取最近的一个箭头标注
        let arrow_annotation_on_stack_top = self
            .annotations_stack
            .last_mut()
            .map(|annotation| match annotation {
                Annotation::Arrow(arrow_annotation) => Some(arrow_annotation),
                _ => None,
            })
            .flatten();
        func(arrow_annotation_on_stack_top)
    }

    fn pop_annotation(&mut self) -> Option<ArrowAnnotation> {
        // 从标注栈的栈顶中获取最近的一个箭头标注
        self.annotations_stack
            .pop()
            .map(|annotation| match annotation {
                Annotation::Arrow(arrow_annotation) => Some(arrow_annotation),
                _ => None,
            })
            .flatten()
    }
}

pub struct StraightLineBasedTool<S>
where
    S: AnnotationStyle + Default,
{
    annotator_state: Weak<RefCell<AnnotatorState>>,
    tool_state: StraightLineBasedToolState<S>,
}

impl<S> StraightLineBasedTool<S>
where
    S: AnnotationStyle + Default,
{
    pub fn new(annotator_state: Weak<RefCell<AnnotatorState>>) -> StraightLineBasedTool<S> {
        let tool_state = StraightLineBasedToolState::default();
        Self {
            annotator_state,
            tool_state,
        }
    }
}

impl<S> StrokeWidthSupport for StraightLineBasedTool<S>
where
    S: AnnotationStyle + Default,
{
    fn supports_get_stroke_width(&self) -> bool {
        self.tool_state.style.supports_get_stroke_width()
    }

    fn stroke_width(&self) -> f32 {
        self.tool_state.style.stroke_width()
    }

    fn supports_set_stroke_width(&self) -> bool {
        self.tool_state.style.supports_set_stroke_width()
    }

    fn set_stroke_width(&mut self, stroke_width: f32) {
        self.tool_state.style.set_stroke_width(stroke_width);
    }
}

impl<S> StrokeColorSupport for StraightLineBasedTool<S>
where
    S: AnnotationStyle + Default,
{
    fn supports_get_stroke_color(&self) -> bool {
        self.tool_state.style.supports_get_stroke_color()
    }

    fn stroke_color(&self) -> Color32 {
        self.tool_state.style.stroke_color()
    }

    fn supports_set_stroke_color(&self) -> bool {
        self.tool_state.style.supports_set_stroke_color()
    }

    fn set_stroke_color(&mut self, color: Color32) {
        self.tool_state.style.set_stroke_color(color);
    }
}

impl<S> StrokeTypeSupport for StraightLineBasedTool<S>
where
    S: AnnotationStyle + Default,
{
    fn supports_get_stroke_type(&self) -> bool {
        self.tool_state.style.supports_get_stroke_type()
    }

    fn stroke_type(&self) -> StrokeType {
        self.tool_state.style.stroke_type()
    }

    fn supports_set_stroke_type(&self) -> bool {
        self.tool_state.style.supports_set_stroke_type()
    }

    fn set_stroke_type(&mut self, stroke_type: StrokeType) {
        self.tool_state.style.set_stroke_type(stroke_type);
    }
}

impl<S> FillColorSupport for StraightLineBasedTool<S>
where
    S: AnnotationStyle + Default,
{
    fn supports_get_fill_color(&self) -> bool {
        self.tool_state.style.supports_get_fill_color()
    }

    fn fill_color(&self) -> Option<Color32> {
        self.tool_state.style.fill_color()
    }

    fn supports_set_fill_color(&self) -> bool {
        self.tool_state.style.supports_set_fill_color()
    }

    fn set_fill_color(&mut self, color: Color32) {
        self.tool_state.style.set_fill_color(color);
    }
}

impl<S> AnnotationToolCommon for StraightLineBasedTool<S>
where
    S: AnnotationStyle + Default,
{
    fn annotator_state(&self) -> SharedAnnotatorState {
        self.annotator_state.upgrade().unwrap()
    }
}

pub type StraightLineTool = StraightLineBasedTool<StraightLineStyle>;
pub type ArrowTool = StraightLineBasedTool<ArrowStyle>;


impl_stack_top_access_for!(StraightLineTool=>StraightLineAnnotation, ArrowTool=>ArrowAnnotation);

/// 限制最大的线条宽度
const MAX_STROKE_WIDTH_FOR_STRAIGHT_LINE: f32 = 62.;
const MAX_STROKE_WIDTH_FOR_ARROW: f32 = 6.;

impl_stroke_width_handler_for!(StraightLineTool => MAX_STROKE_WIDTH_FOR_STRAIGHT_LINE, ArrowTool => MAX_STROKE_WIDTH_FOR_ARROW);

macro_rules! impl_widget_for {
    ($($tool:ty=>$annotation:ty),*) => {
        $(


impl $tool {
    fn update_cursor_icon(&self, ui: &mut Ui) {
        let Some(pointer_pos) = ui.ctx().pointer_hover_pos() else {
            return;
        };
        // 从标注栈的栈顶中获取最近的一个标注
        let hit_target =
            self.peek_annotation(|annotation_on_stack_top: Option<&$annotation>| {
                // 判断当前鼠标是否位于此标注上
                match annotation_on_stack_top {
                    None => None,
                    Some(annotation) => Some(annotation.hit_test(&pointer_pos)),
                }
            });

        if let Some(hit_target) = hit_target {
            let cursor_icon = hit_target.get_cursor();
            if let Some(cursor_icon) = cursor_icon {
                ui.ctx().set_cursor_icon(cursor_icon);
            } else {
                ui.ctx().set_cursor_icon(CursorIcon::None);
                // 绘制自定义光标
                Crosshair::new(pointer_pos, Color32::RED, self.stroke_width())
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

impl Widget for &mut $tool {
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
            let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
            let hit_target =
                self.peek_annotation(|annotation_on_stack_top: Option<&$annotation>| {
                    // 判断当前鼠标是否位于此标注上
                    match annotation_on_stack_top {
                        None => None,
                        Some(annotation) => Some(annotation.hit_test(&drag_started_pos)),
                    }
                });

            match hit_target {
                Some(hit_target) if hit_target != HitTargetForStraightLine::Outside => {
                    let support_activate = self
                        .peek_annotation(|annotation_on_stack_top: Option<&$annotation>| {
                            Some(
                                annotation_on_stack_top
                                    .unwrap()
                                    .activation()
                                    .supports_activate(),
                            )
                        })
                        .unwrap();

                    if support_activate {
                        // 调整现有的标注
                        let mut annotation = self.pop_annotation().unwrap();
                        annotation.activation_mut().activate();
                        self.tool_state.current_annotation = Some(annotation);
                        self.tool_state.drag_action = hit_target.get_drag_action();
                    }
                }
                Some(_hit_target) => {
                    self.peek_annotation_mut(|mut annotation_on_stack_top| {
                        // 把栈顶的标注设为非激活状态
                        annotation_on_stack_top
                            .as_mut()
                            .unwrap()
                            .activation_mut()
                            .deactivate();
                        None::<()>
                    });
                }
                _ => {}
            }
        } else if response.clicked() {
            self.peek_annotation_mut(|mut annotation_on_stack_top| {
                // 把栈顶的标注设为非激活状态
                if let Some(annotation) = annotation_on_stack_top.as_mut() {
                    annotation.activation_mut().deactivate();
                }

                None::<()>
            });
        }

        if response.dragged() {
            // 拖动中
            if let Some(annotation) = &mut self.tool_state.current_annotation {
                match self.tool_state.drag_action {
                    DragAction::AdjustStartPoint => {
                        annotation.start_position = pointer_pos;
                    }
                    DragAction::AdjustEndPoint => {
                        annotation.end_position = pointer_pos;
                    }
                    DragAction::None => {
                        let drag_started_pos =
                            ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
                        annotation.start_position = drag_started_pos;
                        annotation.end_position = pointer_pos;
                    }
                }
                ui.add(annotation);
            } else {
                let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
                let mut annotation = <$annotation>::new(
                    drag_started_pos,
                    pointer_pos,
                    self.tool_state.style,
                    ActivationSupport::Supported(ActivationState::new(true)),
                );
                self.tool_state.current_annotation = Some(annotation.clone());
                self.tool_state.drag_action = DragAction::None;
                ui.add(&mut annotation);
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
                .push(current_annotation.into());
        }
        response
    }
}


        )*
    };
}

impl_widget_for!(StraightLineTool => StraightLineAnnotation, ArrowTool => ArrowAnnotation);

enum DragAction {
    AdjustStartPoint,
    AdjustEndPoint,
    None,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum HitTargetForStraightLine {
    Outside,
    StartPoint,
    EndPoint,
}

impl HitTargetForStraightLine {
    fn get_drag_action(&self) -> DragAction {
        match self {
            HitTargetForStraightLine::Outside => DragAction::None,
            HitTargetForStraightLine::StartPoint => DragAction::AdjustStartPoint,
            HitTargetForStraightLine::EndPoint => DragAction::AdjustEndPoint,
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
