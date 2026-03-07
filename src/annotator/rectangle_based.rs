use crate::annotator::cursor::Crosshair;
use crate::annotator::{
    Activation, Annotation, AnnotatorState, FillColorSupport, PainterExt, StrokeColorSupport,
    StrokeType, StrokeTypeSupport, StrokeWidthSupport, ToolName,
};
use egui::epaint::EllipseShape;
use egui::{
    Color32, CursorIcon, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Widget, pos2,
    vec2,
};
use std::cell::RefCell;
use std::rc::Weak;

pub trait RectangleBasedAnnotationStyle:
StrokeWidthSupport + StrokeColorSupport + StrokeTypeSupport + FillColorSupport + Default
{
}

/// 基于矩形的标注
#[derive(Debug, Clone)]
pub struct RectangleBasedAnnotation<S>
where
    S: RectangleBasedAnnotationStyle,
{
    /// 区域
    rect: Rect,
    /// 样式
    style: S,
    /// 激活状态
    activation: Activation,
}

impl<S> RectangleBasedAnnotation<S>
where
    S: RectangleBasedAnnotationStyle,
{
    /// 当前的标注是否支持激活
    /// 某些类型的标注可能不支持激活，那么和激活相关的逻辑、依赖激活状态的逻辑将与此标注无关(意味着一个标注被放到栈顶后就不能再次编辑它了)
    pub fn support_activate(&self) -> bool {
        self.activation.support_activate()
    }

    /// 激活此标注
    pub fn activate(&mut self) {
        self.activation.activate();
    }

    /// 取消激活此标注
    pub fn deactivate(&mut self) {
        self.activation.deactivate();
    }

    /// 此标注是否处于激活状态
    pub fn is_active(&self) -> bool {
        self.activation.is_active()
    }
}

impl<S> StrokeWidthSupport for RectangleBasedAnnotation<S>
where
    S: RectangleBasedAnnotationStyle,
{
    fn stroke_width(&self) -> f32 {
        self.style.stroke_width()
    }

    fn set_stroke_width(&mut self, stroke_width: f32) {
        self.style.set_stroke_width(stroke_width);
    }
}

impl<S> StrokeColorSupport for RectangleBasedAnnotation<S>
where
    S: RectangleBasedAnnotationStyle,
{
    fn stroke_color(&self) -> Color32 {
        self.style.stroke_color()
    }

    fn set_stroke_color(&mut self, color: Color32) {
        self.style.set_stroke_color(color);
    }
}

impl<S> StrokeTypeSupport for RectangleBasedAnnotation<S>
where
    S: RectangleBasedAnnotationStyle,
{
    fn stroke_type(&self) -> StrokeType {
        self.style.stroke_type()
    }

    fn set_stroke_type(&mut self, stroke_type: StrokeType) {
        self.style.set_stroke_type(stroke_type);
    }
}

impl<S> FillColorSupport for RectangleBasedAnnotation<S>
where
    S: RectangleBasedAnnotationStyle,
{
    fn fill_color(&self) -> Option<Color32> {
        self.style.fill_color()
    }

    fn set_fill_color(&mut self, color: Color32) {
        self.style.set_fill_color(color);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct RectangleStyle {
    /// 线条颜色和宽度
    pub stroke: Stroke,
    /// 线条类型
    pub stroke_type: StrokeType,
    /// 填充颜色
    pub fill_color: Option<Color32>,
}

impl StrokeWidthSupport for RectangleStyle {
    fn stroke_width(&self) -> f32 {
        self.stroke.width
    }

    fn set_stroke_width(&mut self, stroke_width: f32) {
        self.stroke.width = stroke_width;
    }
}

impl StrokeColorSupport for RectangleStyle {
    fn stroke_color(&self) -> Color32 {
        self.stroke.color
    }

    fn set_stroke_color(&mut self, color: Color32) {
        self.stroke.color = color;
    }
}

impl StrokeTypeSupport for RectangleStyle {
    fn stroke_type(&self) -> StrokeType {
        self.stroke_type
    }

    fn set_stroke_type(&mut self, stroke_type: StrokeType) {
        self.stroke_type = stroke_type;
    }
}

impl FillColorSupport for RectangleStyle {
    fn fill_color(&self) -> Option<Color32> {
        self.fill_color
    }

    fn set_fill_color(&mut self, color: Color32) {
        self.fill_color = Some(color);
    }
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

#[derive(Debug, Copy, Clone)]
pub struct EllipseStyle {
    /// 线条颜色和宽度
    pub stroke: Stroke,
    /// 线条类型
    pub stroke_type: StrokeType,
    /// 填充颜色
    pub fill_color: Option<Color32>,
}

impl StrokeWidthSupport for EllipseStyle {
    fn stroke_width(&self) -> f32 {
        self.stroke.width
    }

    fn set_stroke_width(&mut self, stroke_width: f32) {
        self.stroke.width = stroke_width;
    }
}

impl StrokeColorSupport for EllipseStyle {
    fn stroke_color(&self) -> Color32 {
        self.stroke.color
    }

    fn set_stroke_color(&mut self, color: Color32) {
        self.stroke.color = color;
    }
}

impl StrokeTypeSupport for EllipseStyle {
    fn stroke_type(&self) -> StrokeType {
        self.stroke_type
    }

    fn set_stroke_type(&mut self, stroke_type: StrokeType) {
        self.stroke_type = stroke_type;
    }
}

impl FillColorSupport for EllipseStyle {
    fn fill_color(&self) -> Option<Color32> {
        self.fill_color
    }

    fn set_fill_color(&mut self, color: Color32) {
        self.fill_color = Some(color);
    }
}

impl Default for EllipseStyle {
    fn default() -> Self {
        Self {
            stroke: Stroke::new(1., Color32::from_rgb(255, 0, 0)),
            stroke_type: StrokeType::SolidLine,
            fill_color: None,
        }
    }
}

impl RectangleBasedAnnotationStyle for RectangleStyle {}

impl RectangleBasedAnnotationStyle for EllipseStyle {}

impl<S> RectangleBasedAnnotation<S>
where
    S: RectangleBasedAnnotationStyle,
{
    pub fn rect(&self) -> &Rect {
        &self.rect
    }

    pub fn set_color(&mut self, color: Color32) {
        self.set_stroke_color(color);
        // fill_color有值，意味着启用了fill_color
        if self.fill_color().is_some() {
            self.set_fill_color(color);
        }
    }

    pub fn get_color(&self) -> Color32 {
        self.stroke_color()
    }
}

pub type RectangleAnnotation = RectangleBasedAnnotation<RectangleStyle>;
pub type EllipseAnnotation = RectangleBasedAnnotation<EllipseStyle>;

impl RectangleAnnotation {
    pub fn new(rect: Rect, style: RectangleStyle, activation: Activation) -> Self {
        Self {
            rect,
            style,
            activation,
        }
    }
}

impl EllipseAnnotation {
    pub fn new(rect: Rect, style: EllipseStyle, activation: Activation) -> Self {
        Self {
            rect,
            style,
            activation,
        }
    }
}

impl Into<Annotation> for RectangleAnnotation {
    fn into(self) -> Annotation {
        Annotation::Rectangle(self)
    }
}

impl Into<Annotation> for EllipseAnnotation {
    fn into(self) -> Annotation {
        Annotation::Ellipse(self)
    }
}

impl Widget for &mut RectangleAnnotation {
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

        if self.activation.is_active() {
            painter.small_rects(&self.rect);
        }
        response
    }
}

impl Widget for &mut EllipseAnnotation {
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

        if self.activation.is_active() {
            // 绘制虚线矩形框以及外框上的各个角以及边上的小矩形
            painter.small_rects(&self.rect);
        }
        response
    }
}

pub struct RectangleBasedToolState<S>
where
    S: RectangleBasedAnnotationStyle + Default,
{
    /// 样式
    style: S,
    /// 当前的标注
    current_annotation: Option<RectangleBasedAnnotation<S>>,
    /// 当拖动鼠标的时候需要执行的操作
    drag_action: DragAction,
}

impl<S> Default for RectangleBasedToolState<S>
where
    S: RectangleBasedAnnotationStyle + Default,
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

trait StackTopAccessor<T> {
    fn peek_annotation<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&T>) -> Option<R>;

    fn peek_annotation_mut<F, R>(&mut self, func: F) -> Option<R>
    where
        F: Fn(Option<&mut T>) -> Option<R>;

    fn pop_annotation(&mut self) -> Option<T>;
}

impl StackTopAccessor<RectangleAnnotation> for AnnotatorState {
    fn peek_annotation<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&RectangleAnnotation>) -> Option<R>,
    {
        // 从标注栈的栈顶中获取最近的一个矩形标注
        let rectangle_annotation_on_stack_top = self
            .annotations_stack
            .last()
            .map(|annotation| match annotation {
                Annotation::Rectangle(rectangle_annotation) => Some(rectangle_annotation),
                _ => None,
            })
            .flatten();
        func(rectangle_annotation_on_stack_top)
    }

    fn peek_annotation_mut<F, R>(&mut self, func: F) -> Option<R>
    where
        F: Fn(Option<&mut RectangleAnnotation>) -> Option<R>,
    {
        // 从标注栈的栈顶中获取最近的一个矩形标注
        let rectangle_annotation_on_stack_top = self
            .annotations_stack
            .last_mut()
            .map(|annotation| match annotation {
                Annotation::Rectangle(rectangle_annotation) => Some(rectangle_annotation),
                _ => None,
            })
            .flatten();
        func(rectangle_annotation_on_stack_top)
    }

    fn pop_annotation(&mut self) -> Option<RectangleAnnotation> {
        // 从标注栈的栈顶中获取最近的一个矩形标注
        self.annotations_stack
            .pop()
            .map(|annotation| match annotation {
                Annotation::Rectangle(rectangle_annotation) => Some(rectangle_annotation),
                _ => None,
            })
            .flatten()
    }
}

impl StackTopAccessor<EllipseAnnotation> for AnnotatorState {
    fn peek_annotation<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&EllipseAnnotation>) -> Option<R>,
    {
        // 从标注栈的栈顶中获取最近的一个椭圆标注
        let ellipse_annotation_on_stack_top = self
            .annotations_stack
            .last()
            .map(|annotation| match annotation {
                Annotation::Ellipse(ellipse_annotation) => Some(ellipse_annotation),
                _ => None,
            })
            .flatten();
        func(ellipse_annotation_on_stack_top)
    }

    fn peek_annotation_mut<F, R>(&mut self, func: F) -> Option<R>
    where
        F: Fn(Option<&mut EllipseAnnotation>) -> Option<R>,
    {
        // 从标注栈的栈顶中获取最近的一个椭圆标注
        let ellipse_annotation_on_stack_top = self
            .annotations_stack
            .last_mut()
            .map(|annotation| match annotation {
                Annotation::Ellipse(ellipse_annotation) => Some(ellipse_annotation),
                _ => None,
            })
            .flatten();
        func(ellipse_annotation_on_stack_top)
    }

    fn pop_annotation(&mut self) -> Option<EllipseAnnotation> {
        // 从标注栈的栈顶中获取最近的一个椭圆标注
        self.annotations_stack
            .pop()
            .map(|annotation| match annotation {
                Annotation::Ellipse(ellipse_annotation) => Some(ellipse_annotation),
                _ => None,
            })
            .flatten()
    }
}

trait Tool {
    fn tool_name(&self) -> ToolName;
}

pub struct RectangleBasedTool<S>
where
    S: RectangleBasedAnnotationStyle + Default,
{
    annotator_state: Weak<RefCell<AnnotatorState>>,
    tool_state: RectangleBasedToolState<S>,
    tool_name: ToolName,
}
impl<S> RectangleBasedTool<S>
where
    S: RectangleBasedAnnotationStyle + Default,
{
    pub fn set_color(&mut self, color: Color32) {
        self.set_stroke_color(color);
        // fill_color有值，意味着启用了fill_color
        if self.fill_color().is_some() {
            self.set_fill_color(color);
        }
    }

    pub fn get_color(&self) -> Color32 {
        self.stroke_color()
    }
}

pub type RectangleTool = RectangleBasedTool<RectangleStyle>;
pub type EllipseTool = RectangleBasedTool<EllipseStyle>;

impl RectangleTool {
    pub fn new(annotator_state: Weak<RefCell<AnnotatorState>>) -> RectangleTool {
        Self {
            annotator_state,
            tool_state: Default::default(),
            tool_name: ToolName::Rectangle,
        }
    }
}

impl EllipseTool {
    pub fn new(annotator_state: Weak<RefCell<AnnotatorState>>) -> EllipseTool {
        Self {
            annotator_state,
            tool_state: Default::default(),
            tool_name: ToolName::Ellipse,
        }
    }
}

impl Tool for RectangleTool {
    fn tool_name(&self) -> ToolName {
        ToolName::Rectangle
    }
}

impl Tool for EllipseTool {
    fn tool_name(&self) -> ToolName {
        ToolName::Ellipse
    }
}

impl<S> StrokeWidthSupport for RectangleBasedTool<S>
where
    S: RectangleBasedAnnotationStyle + Default,
{
    fn stroke_width(&self) -> f32 {
        self.tool_state.style.stroke_width()
    }

    fn set_stroke_width(&mut self, stroke_width: f32) {
        self.tool_state.style.set_stroke_width(stroke_width);
    }
}
impl<S> StrokeColorSupport for RectangleBasedTool<S>
where
    S: RectangleBasedAnnotationStyle + Default,
{
    fn stroke_color(&self) -> Color32 {
        self.tool_state.style.stroke_color()
    }

    fn set_stroke_color(&mut self, color: Color32) {
        self.tool_state.style.set_stroke_color(color);
    }
}

impl<S> StrokeTypeSupport for RectangleBasedTool<S>
where
    S: RectangleBasedAnnotationStyle + Default,
{
    fn stroke_type(&self) -> StrokeType {
        self.tool_state.style.stroke_type()
    }

    fn set_stroke_type(&mut self, stroke_type: StrokeType) {
        self.tool_state.style.set_stroke_type(stroke_type);
    }
}

impl<S> FillColorSupport for RectangleBasedTool<S>
where
    S: RectangleBasedAnnotationStyle + Default,
{
    fn fill_color(&self) -> Option<Color32> {
        self.tool_state.style.fill_color()
    }

    fn set_fill_color(&mut self, color: Color32) {
        self.tool_state.style.set_fill_color(color);
    }
}

const MAX_STROKE_WIDTH: f32 = 62.;

impl RectangleTool {
    fn update_stroke_width_for_stack_top_annotation(&mut self, new_width: f32) {
        self.peek_annotation_mut(|mut annotation: Option<&mut RectangleAnnotation>| {
            if let Some(annotation) = annotation.as_mut() {
                if annotation.support_activate() && annotation.is_active() {
                    annotation.set_stroke_width(new_width);
                }
            }
            NOTHING
        });
    }

    fn peek_annotation<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&RectangleAnnotation>) -> Option<R>,
    {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        let annotator_state = annotator_state.borrow();
        annotator_state.peek_annotation(func)
    }

    fn peek_annotation_mut<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&mut RectangleAnnotation>) -> Option<R>,
    {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        let mut annotator_state = annotator_state.borrow_mut();
        annotator_state.peek_annotation_mut(func)
    }

    fn pop_annotation(&self) -> Option<RectangleAnnotation> {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        annotator_state.borrow_mut().pop_annotation()
    }
}

macro_rules! impl_widget_for {
    ($($tool:ty=>$annotation:ty),*) => {
        $(
        impl $tool {
            fn handle_wheel_event(&mut self, ui: &mut Ui) {
                // 滚动鼠标滚轮调整线条大小
                let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta.y);
                if scroll_delta != 0. {
                    ui.memory_mut(|memory| {
                        let step_threshold = 9f32;
                        let value = memory.data.get_temp_mut_or_default::<f32>(Id::from("rectangle-based-tool-scroll-delta"));
                        *value += scroll_delta;

                        while *value >= step_threshold {
                            *value -= step_threshold;
                            let stroke_width = self.stroke_width();
                            if stroke_width > 1. {
                                self.set_stroke_width( stroke_width - 1.);
                            }
                        }

                        while *value <= -step_threshold{
                            *value += step_threshold;
                            let stroke_width = self.stroke_width();
                            if stroke_width < MAX_STROKE_WIDTH {
                                self.set_stroke_width(stroke_width + 1.0);
                            }
                        }
                    });
                }
            }

            fn update_cursor_icon(&self, ui: &mut Ui) {
                let Some(pointer_pos) = ui.ctx().pointer_hover_pos() else {
                    return;
                };
                // 从标注栈的栈顶中获取最近的一个标注
                let hit_target = self.peek_annotation(|annotation_on_stack_top: Option<&$annotation>| {
                    // 判断当前鼠标是否位于此标注上
                    match annotation_on_stack_top {
                        None => None,
                        Some(annotation) => {
                            let stroke_width = annotation.stroke_width();
                            Some(annotation.rect().hit_test(&pointer_pos, stroke_width))
                        }
                    }
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
                            self.stroke_width(),
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
                    let hit_target = self.peek_annotation(|annotation_on_stack_top: Option<&$annotation>| {
                         // 判断当前鼠标是否位于此标注上
                         match annotation_on_stack_top {
                             None => None,
                             Some(annotation) => {
                                 let stroke_width = annotation.stroke_width();
                                 Some(annotation.rect().hit_test(&drag_started_pos, stroke_width))
                             }
                         }
                    });

                    match hit_target {
                        Some(hit_target)
                            if hit_target != HitTarget::Inside && hit_target != HitTarget::Outside =>
                            {
                                let support_activate = self.peek_annotation(|annotation_on_stack_top: Option<&$annotation>| {
                                    Some(annotation_on_stack_top.unwrap().activation.support_activate())
                                }).unwrap();

                                if support_activate {
                                    // 调整现有的标注
                                    let mut annotation = self.pop_annotation().unwrap();
                                    annotation.activate();
                                    self.tool_state.current_annotation = Some(annotation);
                                    self.tool_state.drag_action = hit_target.get_drag_action();
                                }
                            }
                        Some(hit_target)
                            if hit_target == HitTarget::Inside || hit_target == HitTarget::Outside =>
                            {
                                self.peek_annotation_mut(|mut annotation_on_stack_top| {
                                    // 把栈顶的标注设为非激活状态
                                    annotation_on_stack_top.as_mut().unwrap().deactivate();
                                    None::<()>
                                });
                            }
                            _ => {}
                    }
                } else if response.clicked() {
                    self.peek_annotation_mut(|mut annotation_on_stack_top| {
                        // 把栈顶的标注设为非激活状态
                        if let Some(annotation) = annotation_on_stack_top.as_mut() {
                            annotation.deactivate();
                        }

                        None::<()>
                    });
                }

                if response.dragged() {
                    // 拖动中
                    if let Some(annotation) = &mut self.tool_state.current_annotation {
                        match self.tool_state.drag_action {
                            DragAction::AdjustTopEdge => {
                                annotation.rect.min.y = pointer_pos.y;
                            }
                            DragAction::AdjustBottomEdge => {
                                annotation.rect.max.y = pointer_pos.y;
                            }
                            DragAction::AdjustLeftEdge => {
                                annotation.rect.min.x = pointer_pos.x;
                            }
                            DragAction::AdjustRightEdge => {
                                annotation.rect.max.x = pointer_pos.x;
                            }
                            DragAction::AdjustTopLeftCorner => {
                                annotation.rect.min = pointer_pos;
                            }
                            DragAction::AdjustTopRightCorner => {
                                annotation.rect.min.y = pointer_pos.y;
                                annotation.rect.max.x = pointer_pos.x;
                            }
                            DragAction::AdjustBottomRightCorner => {
                                annotation.rect.max = pointer_pos;
                            }
                            DragAction::AdjustBottomLeftCorner => {
                                annotation.rect.min.x = pointer_pos.x;
                                annotation.rect.max.y = pointer_pos.y;
                            }

                            DragAction::None => {
                                let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
                                annotation.rect = Rect::from_two_pos(drag_started_pos, pointer_pos);
                            }
                        }
                        ui.add(annotation);
                    } else {
                        let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
                        let rect = Rect::from_two_pos(drag_started_pos, pointer_pos);
                        let mut annotation = <$annotation>::new(rect, self.tool_state.style, Activation::Activable(true));
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

impl EllipseTool {
    fn update_stroke_width_for_stack_top_annotation(&mut self, new_width: f32) {
        self.peek_annotation_mut(|mut annotation: Option<&mut EllipseAnnotation>| {
            if let Some(annotation) = annotation.as_mut() {
                if annotation.support_activate() && annotation.is_active() {
                    annotation.set_stroke_width(new_width);
                }
            }
            NOTHING
        });
    }

    fn peek_annotation<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&EllipseAnnotation>) -> Option<R>,
    {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        let annotator_state = annotator_state.borrow();
        annotator_state.peek_annotation(func)
    }

    fn peek_annotation_mut<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&mut EllipseAnnotation>) -> Option<R>,
    {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        let mut annotator_state = annotator_state.borrow_mut();
        annotator_state.peek_annotation_mut(func)
    }

    fn pop_annotation(&self) -> Option<EllipseAnnotation> {
        let annotator_state = self.annotator_state.upgrade().unwrap();
        annotator_state.borrow_mut().pop_annotation()
    }
}

impl_widget_for!(RectangleTool => RectangleAnnotation, EllipseTool => EllipseAnnotation);

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

    None,
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
        } else {
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
        assert_eq!(rect.hit_test(&pos2(50.0, 0.0), 1.0), HitTarget::TopEdge);

        // 测试上边 - 偏移3像素内 (tolerance=6, half=3)
        assert_eq!(rect.hit_test(&pos2(50.0, 2.9), 1.0), HitTarget::TopEdge);

        // 测试下边
        assert_eq!(rect.hit_test(&pos2(50.0, 50.0), 1.0), HitTarget::BottomEdge);

        // 测试左边
        assert_eq!(rect.hit_test(&pos2(0.0, 25.0), 1.0), HitTarget::LeftEdge);

        // 测试右边
        assert_eq!(rect.hit_test(&pos2(100.0, 25.0), 1.0), HitTarget::RightEdge);
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
        assert_eq!(rect.hit_test(&pos2(50.0, 25.0), 1.0), HitTarget::Inside);

        // 内部但不是中心
        assert_eq!(rect.hit_test(&pos2(10.0, 10.0), 1.0), HitTarget::Inside);

        // 完全外部
        assert_eq!(rect.hit_test(&pos2(-10.0, 25.0), 1.0), HitTarget::Outside);

        assert_eq!(rect.hit_test(&pos2(50.0, -10.0), 1.0), HitTarget::Outside);

        assert_eq!(rect.hit_test(&pos2(150.0, 25.0), 1.0), HitTarget::Outside);
    }

    // 测试边缘扩展区域
    #[test]
    fn test_hit_test_edge_extended() {
        let rect = test_rect();

        // 上边扩展区域 (y: -3 to 3)
        assert_eq!(rect.hit_test(&pos2(50.0, -2.9), 1.0), HitTarget::TopEdge);

        // 超出扩展区域
        assert_eq!(rect.hit_test(&pos2(50.0, -3.1), 1.0), HitTarget::Outside);

        // 下边扩展区域 (y: 47 to 53)
        assert_eq!(rect.hit_test(&pos2(50.0, 52.9), 1.0), HitTarget::BottomEdge);

        // 左边扩展区域 (x: -3 to 3)
        assert_eq!(rect.hit_test(&pos2(-2.9, 25.0), 1.0), HitTarget::LeftEdge);

        // 右边扩展区域 (x: 97 to 103)
        assert_eq!(rect.hit_test(&pos2(102.9, 25.0), 1.0), HitTarget::RightEdge);
    }

    // 测试不同 stroke_width 值
    #[test]
    fn test_hit_test_different_stroke_width() {
        let rect = test_rect();

        // stroke_width=1.0，tolerance=6
        assert_eq!(rect.hit_test(&pos2(50.0, 2.9), 1.0), HitTarget::TopEdge);

        assert_eq!(
            rect.hit_test(&pos2(50.0, 3.1), 1.0),
            HitTarget::Inside // 在扩展区域外，但在矩形内
        );

        // stroke_width=10.0，tolerance=10，half=5
        // 现在扩展区域更大
        assert_eq!(rect.hit_test(&pos2(50.0, 4.9), 10.0), HitTarget::TopEdge);

        assert_eq!(
            rect.hit_test(&pos2(50.0, 5.1), 10.0),
            HitTarget::Inside // 在扩展区域外，但在矩形内
        );

        // stroke_width=20.0，tolerance=20，half=10
        // 扩展区域非常大
        assert_eq!(rect.hit_test(&pos2(50.0, 9.9), 20.0), HitTarget::TopEdge);

        // 注意：当扩展区域非常大时，甚至可能覆盖到矩形内部
        // 测试一个在矩形内部但在扩展区域内的点
        assert_eq!(rect.hit_test(&pos2(50.0, 8.0), 20.0), HitTarget::TopEdge);
    }

    // 测试边界条件和特殊情况
    #[test]
    fn test_hit_test_edge_cases() {
        let rect = test_rect();

        // 点在边上但x坐标超出矩形范围（但在扩展区域内）
        assert_eq!(
            rect.hit_test(&pos2(-2.9, 0.0), 1.0),
            HitTarget::TopLeftCorner // 同时在上边和左边
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
        assert_eq!(rect2.hit_test(&pos2(0.0, -50.0), 1.0), HitTarget::TopEdge);
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
