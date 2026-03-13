use crate::annotator::cursor::{Circle, Crosshair, CustomCursor};
use crate::annotator::{ActivationSupport, Annotation, AnnotationActivationSupport, AnnotationStyle, AnnotationToolCommon, AnnotatorState, UnsubmittedAnnotationHandler, FillColorSupport, SharedAnnotatorState, StrokeColorSupport, StrokeType, StrokeTypeSupport, StrokeWidthSupport, WheelHandler};
use egui::{pos2, Color32, CursorIcon, Pos2, Rect, Response, Sense, Stroke, Ui, Widget};
use std::cell::RefCell;
use std::rc::Weak;

#[derive(Debug, Copy, Clone)]
pub struct PencilStyle {
    /// 线条颜色和宽度
    stroke: Stroke,
    /// 线条类型
    stroke_type: StrokeType,
}

impl StrokeWidthSupport for PencilStyle {
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

impl StrokeColorSupport for PencilStyle {
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

impl StrokeTypeSupport for PencilStyle {
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

impl FillColorSupport for PencilStyle {
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

impl AnnotationStyle for PencilStyle {}

impl Default for PencilStyle {
    fn default() -> Self {
        Self {
            stroke: Stroke::new(1., Color32::from_rgb(255, 0, 0)),
            stroke_type: StrokeType::SolidLine,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MarkerPenStyle {
    /// 线条颜色和宽度
    pub stroke: Stroke,
    /// 线条类型
    pub stroke_type: StrokeType,
}

impl StrokeWidthSupport for MarkerPenStyle {
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

impl StrokeColorSupport for MarkerPenStyle {
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

impl StrokeTypeSupport for MarkerPenStyle {
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

impl FillColorSupport for MarkerPenStyle {
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

impl AnnotationStyle for MarkerPenStyle {}

impl Default for MarkerPenStyle {
    fn default() -> Self {
        Self {
            stroke: Stroke::new(20., Color32::from_rgba_unmultiplied(255, 0, 0, 76)),
            stroke_type: StrokeType::SolidLine,
        }
    }
}

/// 基于自由曲线的标注
#[derive(Debug, Clone)]
pub struct FreeLineBasedAnnotation<S>
where
    S: AnnotationStyle,
{
    /// 构成曲线的点
    points: Vec<Pos2>,
    /// 样式
    style: S,
    /// 激活状态
    activation: ActivationSupport,
}

impl<S> FreeLineBasedAnnotation<S>
where
    S: AnnotationStyle,
{
    pub fn new(
        points: Vec<Pos2>,
        style: S,
        activation: ActivationSupport,
    ) -> Self {
        Self {
            points,
            style,
            activation,
        }
    }

    pub fn points(&self) -> &Vec<Pos2> {
        &self.points
    }
}

impl<S> StrokeWidthSupport for FreeLineBasedAnnotation<S>
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

impl<S> StrokeColorSupport for FreeLineBasedAnnotation<S>
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

impl<S> StrokeTypeSupport for FreeLineBasedAnnotation<S>
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

impl<S> FillColorSupport for FreeLineBasedAnnotation<S>
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

impl<S> AnnotationActivationSupport for FreeLineBasedAnnotation<S>
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

pub type PencilAnnotation = FreeLineBasedAnnotation<PencilStyle>;
pub type MarkerPenAnnotation = FreeLineBasedAnnotation<MarkerPenStyle>;

impl Into<Annotation> for PencilAnnotation {
    fn into(self) -> Annotation {
        Annotation::Pencil(self)
    }
}

impl Into<Annotation> for MarkerPenAnnotation {
    fn into(self) -> Annotation {
        Annotation::MarkerPen(self)
    }
}

impl<S> Widget for &mut FreeLineBasedAnnotation<S>
where
    S: AnnotationStyle, 
{
    fn ui(self, ui: &mut Ui) -> Response {
        let mut left = None;
        let mut top = None;
        let mut right = None;
        let mut bottom = None;
        for point in &self.points {
            if let Some(ref mut l) = left {
                if point.x < *l {
                    *l = point.x;
                }
            } else {
                left = Some(point.x);
            }

            if let Some(ref mut r) = right {
                if point.x > *r {
                    *r = point.x;
                }
            } else {
                right = Some(point.x);
            }

            if let Some(ref mut t) = top {
                if point.y < *t {
                    *t = point.y;
                }
            } else {
                top = Some(point.y);
            }

            if let Some(ref mut b) = bottom {
                if point.y > *b {
                    *b = point.y;
                }
            } else {
                bottom = Some(point.y);
            }
        }
        let rect = Rect::from_two_pos(pos2(left.unwrap(), top.unwrap()), pos2(right.unwrap(), bottom.unwrap()));
        let response = ui.allocate_rect(rect, Sense::hover());
        let painter = ui.painter();
        painter.line(self.points.clone(), Stroke::new(self.style.stroke_width(), self.style.stroke_color()));
        response
    }
}

pub struct FreeLineBasedToolState<S>
where
    S: AnnotationStyle + Default,
{
    /// 样式
    style: S,
    /// 当前的标注
    current_annotation: Option<FreeLineBasedAnnotation<S>>,
    /// 当拖动鼠标的时候需要执行的操作
    drag_action: DragAction,
}

impl<S> Default for FreeLineBasedToolState<S>
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

pub struct FreeLineBasedTool<S>
where
    S: AnnotationStyle + Default,
{
    annotator_state: Weak<RefCell<AnnotatorState>>,
    tool_state: FreeLineBasedToolState<S>,
}

impl<S> FreeLineBasedTool<S>
where
    S: AnnotationStyle + Default,
{
    pub fn new(annotator_state: Weak<RefCell<AnnotatorState>>) -> FreeLineBasedTool<S> {
        let tool_state = FreeLineBasedToolState::default();
        Self {
            annotator_state,
            tool_state,
        }
    }
}

impl<S> StrokeWidthSupport for FreeLineBasedTool<S>
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

impl<S> StrokeColorSupport for FreeLineBasedTool<S>
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

impl<S> StrokeTypeSupport for FreeLineBasedTool<S>
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

impl<S> FillColorSupport for FreeLineBasedTool<S>
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

impl<S> AnnotationToolCommon for FreeLineBasedTool<S>
where
    S: AnnotationStyle + Default,
{
    fn annotator_state(&self) -> SharedAnnotatorState {
        self.annotator_state.upgrade().unwrap()
    }
}

impl <S> UnsubmittedAnnotationHandler for FreeLineBasedTool<S>
where
    S: AnnotationStyle + Default,
{
}

/// 限制最大的线条宽度
const MAX_STROKE_WIDTH: f32 = 62.;



pub type PencilTool = FreeLineBasedTool<PencilStyle>;
pub type MarkerPenTool = FreeLineBasedTool<MarkerPenStyle>;

impl PencilTool {
    fn update_cursor_icon(&self, ui: &mut Ui) {
        let Some(pointer_pos) = ui.ctx().pointer_hover_pos() else {
            return;
        };

        ui.ctx().set_cursor_icon(CursorIcon::None);
        // 绘制自定义光标
        Crosshair::new(
            pointer_pos,
            Color32::RED,
            self.tool_state.style.stroke_width(),
        ).paint_with(ui.painter());
    }
}

impl MarkerPenTool {
    fn update_cursor_icon(&self, ui: &mut Ui) {
        let Some(pointer_pos) = ui.ctx().pointer_hover_pos() else {
            return;
        };

        ui.ctx().set_cursor_icon(CursorIcon::None);
        // 绘制自定义光标
        Circle::new(
            pointer_pos,
            Color32::RED,
            self.tool_state.style.stroke_width(),
        ).paint_with(ui.painter());
    }
}

impl WheelHandler for PencilTool
{
    fn on_scroll_delta_changed(&mut self, value: f32) {
        if !self.supports_set_stroke_width() {
            return;
        }
        let mut stroke_width = self.stroke_width();
        if value > 0. {
            if stroke_width > 1. {
                stroke_width -= 1.;
            }
        } else {
            if stroke_width < MAX_STROKE_WIDTH {
                stroke_width += 1.0;
            }
        }
        self.set_stroke_width(stroke_width);
    }
}

impl WheelHandler for MarkerPenTool
{
    fn on_scroll_delta_changed(&mut self, value: f32) {
        if !self.supports_set_stroke_width() {
            return;
        }
        let mut stroke_width = self.stroke_width();
        if value > 0. {
            if stroke_width > 20. {
                stroke_width -= 1.;
            }
        } else {
            if stroke_width < MAX_STROKE_WIDTH {
                stroke_width += 1.0;
            }
        }
        self.set_stroke_width(stroke_width);
    }
}


macro_rules! impl_widget_for {
    ($($tool:ty=>$annotation:ty),*) => {
        $(
impl Widget for &mut $tool  {
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
            let points = vec![drag_started_pos];
            let annotation = <$annotation>::new(points, self.tool_state.style, ActivationSupport::NotSupported);
            self.tool_state.current_annotation = Some(annotation);
        }

        if response.dragged() {
            // 拖动中
            if let Some(annotation) = self.tool_state.current_annotation.as_mut() {
                annotation.points.push(pointer_pos);
                ui.add(annotation);
            }
        }

        if response.drag_stopped() {
            // 拖动结束
            let mut annotation = self.tool_state.current_annotation.take().unwrap();
            annotation.points.push(pointer_pos);

            self.annotator_state
                .upgrade()
                .unwrap()
                .borrow_mut()
                .annotations_stack
                .push(annotation.into());
        }
        response
    }
}

   )*
  }
}

impl_widget_for!(PencilTool=>PencilAnnotation, MarkerPenTool=>MarkerPenAnnotation);

enum DragAction {
    AdjustStartPoint,
    AdjustEndPoint,
    None,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum HitTargetForFreeLine {
    Outside,
    StartPoint,
    EndPoint,
}

impl HitTargetForFreeLine {
    fn get_drag_action(&self) -> DragAction {
        match self {
            HitTargetForFreeLine::Outside => DragAction::None,
            HitTargetForFreeLine::StartPoint => DragAction::AdjustStartPoint,
            HitTargetForFreeLine::EndPoint => DragAction::AdjustEndPoint,
        }
    }

    fn get_cursor(&self) -> Option<CursorIcon> {
        match self {
            HitTargetForFreeLine::Outside => None,
            HitTargetForFreeLine::StartPoint => Some(CursorIcon::ResizeNwSe),
            HitTargetForFreeLine::EndPoint => Some(CursorIcon::ResizeNwSe),
        }
    }
}
