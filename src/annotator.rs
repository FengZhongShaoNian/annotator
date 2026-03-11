mod cursor;
pub mod drop_down_box;
pub mod free_line_based;
pub mod image_based;
pub mod rectangle_based;
pub mod serial_number;
pub mod straight_line_based;
pub(crate) mod svg_button;
pub mod text;
pub mod watermark;

use crate::annotator::free_line_based::{
    MarkerPenAnnotation, MarkerPenTool, PencilAnnotation, PencilTool,
};
use crate::annotator::image_based::{
    BlurAnnotation, BlurTool, EraserAnnotation, EraserTool, MosaicAnnotation, MosaicTool,
};
use crate::annotator::rectangle_based::{
    EllipseAnnotation, EllipseTool, RectangleAnnotation, RectangleTool,
};
use crate::annotator::straight_line_based::{
    ArrowAnnotation, ArrowTool, StraightLineAnnotation, StraightLineTool,
};
use crate::egui_off_screen_render::EguiOffScreenRender;
use crate::global::Global;
use crate::view::ViewId;
use egui::ahash::HashMap;
use egui::{
    Color32, Id, Painter, Pos2, Rect, Response, Shape, Stroke, StrokeKind, TextureHandle, Ui, Vec2,
    Widget, pos2,
};
use image::RgbaImage;
use spire_enum::prelude::{delegate_impl, delegated_enum};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use delegate::delegate;

/// 线条类型（实线、虚线、点线）
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum StrokeType {
    /// 实线
    SolidLine,

    /// 虚线
    DashedLine,

    /// 点线
    DottedLine,
}

/// 线条宽度
pub trait StrokeWidthSupport {
    /// 是否支持获取线条宽度
    fn supports_get_stroke_width(&self) -> bool;
    /// 获取线条宽度
    fn stroke_width(&self) -> f32;
    /// 是否支持设置线条宽度
    fn supports_set_stroke_width(&self) -> bool;
    /// 设置线条宽度
    fn set_stroke_width(&mut self, stroke_width: f32);
}

/// 线条颜色
pub trait StrokeColorSupport {
    /// 是否支持获取线条颜色
    fn supports_get_stroke_color(&self) -> bool;
    /// 获取线条颜色
    fn stroke_color(&self) -> Color32;
    /// 是否支持设置线条颜色
    fn supports_set_stroke_color(&self) -> bool;
    /// 设置线条颜色
    fn set_stroke_color(&mut self, color: Color32);
}

/// 线条类型
pub trait StrokeTypeSupport {
    /// 是否支持获取线条类型
    fn supports_get_stroke_type(&self) -> bool;
    /// 线条类型
    fn stroke_type(&self) -> StrokeType;
    /// 是否支持设置线条类型
    fn supports_set_stroke_type(&self) -> bool;
    /// 设置线条类型
    fn set_stroke_type(&mut self, stroke_type: StrokeType);
}

/// 填充颜色
pub trait FillColorSupport {
    /// 是否支持获取填充颜色
    fn supports_get_fill_color(&self) -> bool;
    /// 填充颜色
    fn fill_color(&self) -> Option<Color32>;
    /// 是否支持设置填充颜色
    fn supports_set_fill_color(&self) -> bool;
    /// 设置填充颜色
    fn set_fill_color(&mut self, color: Color32);
}

#[macro_export]
macro_rules! declare_not_support_stroke_width {
    ($($type_name:ty),*) => {
        $(

            impl StrokeWidthSupport for $type_name {
                /// 是否支持获取线条宽度
                fn supports_get_stroke_width(&self) -> bool {
                    false
                }

                /// 获取线条宽度
                fn stroke_width(&self) -> f32 {
                    unimplemented!()
                }

                /// 是否支持设置线条宽度
                fn supports_set_stroke_width(&self) -> bool {
                    false
                }
                /// 设置线条宽度
                fn set_stroke_width(&mut self, _stroke_width: f32) {
                    unimplemented!()
                }
            }

        )*
    }
}

#[macro_export]
macro_rules! declare_not_support_stroke_color {
    ($($type_name:ty),*) => {
        $(

            impl StrokeColorSupport for $type_name {
                /// 是否支持获取线条颜色
                fn supports_get_stroke_color(&self) -> bool{
                    false
                }
                /// 获取线条颜色
                fn stroke_color(&self) -> Color32 {
                    unimplemented!()
                }
                /// 是否支持设置线条颜色
                fn supports_set_stroke_color(&self) -> bool{
                    false
                }
                /// 设置线条颜色
                fn set_stroke_color(&mut self, _color: Color32){
                    unimplemented!()
                }
            }

        )*
    }
}

#[macro_export]
macro_rules! declare_not_support_stroke_type {
    ($($type_name:ty),*) => {
        $(

            impl StrokeTypeSupport for $type_name {
                /// 是否支持获取线条类型
                fn supports_get_stroke_type(&self) -> bool{
                    false
                }
                /// 线条类型
                fn stroke_type(&self) -> StrokeType{
                    unimplemented!()
                }
                /// 是否支持设置线条类型
                fn supports_set_stroke_type(&self) -> bool{
                    false
                }
                /// 设置线条类型
                fn set_stroke_type(&mut self, _stroke_type: StrokeType){
                    unimplemented!()
                }
            }

        )*
    }
}

#[macro_export]
macro_rules! declare_not_support_fill_color {
    ($($type_name:ty),*) => {
        $(

            impl FillColorSupport for $type_name {
                /// 是否支持获取填充颜色
                fn supports_get_fill_color(&self) -> bool{
                    false
                }
                /// 填充颜色
                fn fill_color(&self) -> Option<Color32>{
                    unimplemented!()
                }
                /// 是否支持设置填充颜色
                fn supports_set_fill_color(&self) -> bool{
                    false
                }
                /// 设置填充颜色
                fn set_fill_color(&mut self, _color: Color32){
                    unimplemented!()
                }
            }

        )*
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ActivationState {
    pub active: bool,
}

impl ActivationState {
    pub fn new(active: bool) -> Self {
        ActivationState { active }
    }
}

#[derive(Clone, Debug)]
pub enum ActivationSupport {
    NotSupported,
    Supported(ActivationState),
}

impl ActivationSupport {
    /// 当前的标注是否支持激活
    /// 某些类型的标注可能不支持激活，那么和激活相关的逻辑、依赖激活状态的逻辑将与此标注无关(例如：一个标注被添加到栈顶后将不再可编辑)
    pub fn supports_activate(&self) -> bool {
        match self {
            ActivationSupport::Supported(_) => true,
            _ => false,
        }
    }

    /// 激活此标注
    pub fn activate(&mut self) {
        match self {
            ActivationSupport::Supported(state) => state.active = true,
            _ => unimplemented!(),
        }
    }

    /// 取消激活此标注
    pub fn deactivate(&mut self) {
        match self {
            ActivationSupport::Supported(state) => {
                state.active = false;
            }
            _ => (),
        }
    }

    /// 此标注是否处于激活状态
    pub fn is_active(&self) -> bool {
        match self {
            ActivationSupport::Supported(state) => state.active,
            _ => false,
        }
    }
}

pub trait AnnotationStyle:
    StrokeWidthSupport + StrokeColorSupport + StrokeTypeSupport + FillColorSupport + Default
{
}

pub trait Paint {
    fn paint_with(&mut self, painter: &Painter);
}

pub trait AnnotationActivationSupport {
    fn activation(&self) -> &ActivationSupport;
    fn activation_mut(&mut self) -> &mut ActivationSupport;
}

pub trait AnnotationToolCommon:
    StrokeWidthSupport + StrokeColorSupport + StrokeTypeSupport + FillColorSupport
{
    fn annotator_state(&self) -> SharedAnnotatorState;
}

pub trait WheelHandler {
    fn handle_wheel_event(&mut self, ui: &mut Ui) {
        // 滚动鼠标滚轮调整线条大小
        let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta.y);
        if scroll_delta != 0. {
            ui.memory_mut(|memory| {
                let step_threshold = 9f32;
                let value = memory
                    .data
                    .get_temp_mut_or_default::<f32>(Id::from("wheel-scroll-value-accumulate"));
                *value += scroll_delta;

                while *value >= step_threshold {
                    *value -= step_threshold;
                    self.on_scroll_delta_changed(step_threshold);
                }

                while *value <= -step_threshold {
                    *value += step_threshold;
                    self.on_scroll_delta_changed(-step_threshold);
                }
            });
        }
    }

    fn on_scroll_delta_changed(&mut self, value: f32);
}

#[macro_export]
macro_rules! impl_stroke_width_handler_for {
    ($($tool:ty=>$max_stroke_width:expr),*) => {
        $(

        impl WheelHandler for $tool {
            fn on_scroll_delta_changed(&mut self, value: f32) {
                if !self.supports_set_stroke_width() {
                    return;
                }
                let mut stroke_width = self.stroke_width();
                if value > 0. {
                    if stroke_width > 1. {
                        stroke_width -= 1.;
                    }
                }else {
                    if stroke_width < $max_stroke_width {
                        stroke_width += 1.0;
                    }
                }
                self.set_stroke_width(stroke_width);
                self.peek_annotation_mut(|option| {
                    match option {
                        Some(annotation) => {
                            if annotation.activation.is_active() && annotation.supports_get_stroke_width() {
                                annotation.set_stroke_width(stroke_width);
                            }
                        }
                        _ => ()
                    }
                    NOTHING
                });
            }
        }

        )*
    }
}

#[derive(Clone)]
#[delegated_enum]
pub enum Annotation {
    /// 矩形
    Rectangle(RectangleAnnotation),

    /// 椭圆
    Ellipse(EllipseAnnotation),

    /// 直线
    StraightLine(StraightLineAnnotation),

    /// 箭头
    Arrow(ArrowAnnotation),

    /// 铅笔
    Pencil(PencilAnnotation),

    /// 记号笔
    MarkerPen(MarkerPenAnnotation),

    /// 马赛克
    Mosaic(MosaicAnnotation),

    /// 模糊
    Blur(BlurAnnotation),

    /// 文本
    // Text(TextState),

    /// 序号
    // SerialNumber(SerialNumberState),

    /// 水印
    // Watermark(WaterMarkState),

    /// 橡皮擦
    Eraser(EraserAnnotation),
}

impl Annotation {
    /// 检测当前标注是否是由指定工具创建的
    pub fn was_created_by(&mut self, tool: &AnnotationTool) -> bool {
        let tool_name = tool.tool_name();
        match self {
            Annotation::Rectangle(_) => tool_name == ToolName::Rectangle,
            Annotation::Ellipse(_) => tool_name == ToolName::Ellipse,
            Annotation::StraightLine(_) => tool_name == ToolName::StraightLine,
            Annotation::Arrow(_) => tool_name == ToolName::Arrow,
            Annotation::Pencil(_) => tool_name == ToolName::Pencil,
            Annotation::MarkerPen(_) => tool_name == ToolName::MarkerPen,
            Annotation::Mosaic(_) => tool_name == ToolName::Mosaic,
            Annotation::Blur(_) => tool_name == ToolName::Blur,
            // Annotation::Text(_) => tool_name == ToolName::Text,
            // Annotation::SerialNumber(_) => tool_name == ToolName::SerialNumber,
            // Annotation::Watermark(_) => tool_name == ToolName::Watermark,
            Annotation::Eraser(_) => tool_name == ToolName::Eraser,
        }
    }

    delegate! {
        to match self {
            Annotation::Rectangle(inner) => inner,
            Annotation::Ellipse(inner) => inner,
            Annotation::StraightLine(inner) => inner,
            Annotation::Arrow(inner) => inner,
            Annotation::Pencil(inner) => inner,
            Annotation::MarkerPen(inner) => inner,
            Annotation::Mosaic(inner) => inner,
            Annotation::Blur(inner) => inner,
            // Annotation::Text(inner) => inner,
            // Annotation::SerialNumber(inner) => inner,
            // Annotation::Watermark(inner) => inner,
            Annotation::Eraser(inner) => inner,
        } {
            pub fn activation(&self) -> &ActivationSupport;
            pub fn activation_mut(&mut self) -> &mut ActivationSupport;
        }
    }
}

#[delegate_impl]
impl StrokeWidthSupport for Annotation {
    fn supports_get_stroke_width(&self) -> bool;
    fn stroke_width(&self) -> f32;
    fn supports_set_stroke_width(&self) -> bool;
    fn set_stroke_width(&mut self, stroke_width: f32);
}

#[delegate_impl]
impl StrokeColorSupport for Annotation {
    fn supports_get_stroke_color(&self) -> bool;
    fn stroke_color(&self) -> Color32;
    fn supports_set_stroke_color(&self) -> bool;
    fn set_stroke_color(&mut self, color: Color32);
}
#[delegate_impl]
impl StrokeTypeSupport for Annotation {
    fn supports_get_stroke_type(&self) -> bool;
    fn stroke_type(&self) -> StrokeType;
    fn supports_set_stroke_type(&self) -> bool;
    fn set_stroke_type(&mut self, stroke_type: StrokeType);
}

#[delegate_impl]
impl FillColorSupport for Annotation {
    fn supports_get_fill_color(&self) -> bool;
    fn fill_color(&self) -> Option<Color32>;
    fn supports_set_fill_color(&self) -> bool;
    fn set_fill_color(&mut self, color: Color32);
}

#[delegate_impl]
impl Paint for Annotation {
    fn paint_with(&mut self, painter: &Painter);
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum ToolName {
    /// 矩形
    Rectangle,

    /// 椭圆
    Ellipse,

    /// 直线
    StraightLine,

    /// 箭头
    Arrow,

    /// 铅笔
    Pencil,

    /// 记号笔
    MarkerPen,

    /// 马赛克
    Mosaic,

    /// 模糊
    Blur,

    /// 文本
    Text,

    /// 序号
    SerialNumber,

    /// 水印
    Watermark,

    /// 橡皮擦
    Eraser,
}

/// 标注工具的类型
#[delegated_enum]
pub enum AnnotationTool {
    /// 矩形
    Rectangle(RectangleTool),

    /// 椭圆
    Ellipse(EllipseTool),

    /// 直线
    StraightLine(StraightLineTool),

    /// 箭头
    Arrow(ArrowTool),

    /// 铅笔
    Pencil(PencilTool),

    /// 记号笔
    MarkerPen(MarkerPenTool),

    /// 马赛克
    Mosaic(MosaicTool),

    /// 模糊
    Blur(BlurTool),

    // /// 文本
    // Text,
    //
    // /// 序号
    // SerialNumber,
    //
    // /// 水印
    // Watermark,
    /// 橡皮擦
    Eraser(EraserTool),
}

impl AnnotationTool {
    pub fn tool_name(&self) -> ToolName {
        match self {
            AnnotationTool::Rectangle(_) => ToolName::Rectangle,
            AnnotationTool::Ellipse(_) => ToolName::Ellipse,
            AnnotationTool::StraightLine(_) => ToolName::StraightLine,
            AnnotationTool::Arrow(_) => ToolName::Arrow,
            AnnotationTool::Pencil(_) => ToolName::Pencil,
            AnnotationTool::MarkerPen(_) => ToolName::MarkerPen,
            AnnotationTool::Mosaic(_) => ToolName::Mosaic,
            AnnotationTool::Blur(_) => ToolName::Blur,
            // AnnotationTool::Text => ToolName::Text,
            // AnnotationTool::SerialNumber => ToolName::SerialNumber,
            // AnnotationTool::Watermark => ToolName::Watermark,
            AnnotationTool::Eraser(_) => ToolName::Eraser,
        }
    }
}

#[delegate_impl]
impl StrokeWidthSupport for AnnotationTool {
    fn supports_get_stroke_width(&self) -> bool;
    fn stroke_width(&self) -> f32;
    fn supports_set_stroke_width(&self) -> bool;
    fn set_stroke_width(&mut self, stroke_width: f32);
}

#[delegate_impl]
impl StrokeColorSupport for AnnotationTool {
    fn supports_get_stroke_color(&self) -> bool;
    fn stroke_color(&self) -> Color32;
    fn supports_set_stroke_color(&self) -> bool;
    fn set_stroke_color(&mut self, color: Color32);
}

#[delegate_impl]
impl StrokeTypeSupport for AnnotationTool {
    fn supports_get_stroke_type(&self) -> bool;
    fn stroke_type(&self) -> StrokeType;
    fn supports_set_stroke_type(&self) -> bool;
    fn set_stroke_type(&mut self, stroke_type: StrokeType);
}

#[delegate_impl]
impl FillColorSupport for AnnotationTool {
    fn supports_get_fill_color(&self) -> bool;

    fn fill_color(&self) -> Option<Color32>;

    fn supports_set_fill_color(&self) -> bool;

    fn set_fill_color(&mut self, color: Color32);
}

impl AnnotationToolCommon for AnnotationTool {
    fn annotator_state(&self) -> SharedAnnotatorState {
        todo!()
    }
}

impl Widget for &mut AnnotationTool {
    fn ui(self, ui: &mut Ui) -> Response {
        match self {
            AnnotationTool::Rectangle(rectangle_tool) => rectangle_tool.ui(ui),
            AnnotationTool::Ellipse(ellipse_tool) => ellipse_tool.ui(ui),
            AnnotationTool::StraightLine(straight_line_tool) => straight_line_tool.ui(ui),
            AnnotationTool::Arrow(arrow_tool) => arrow_tool.ui(ui),
            AnnotationTool::Pencil(tool) => tool.ui(ui),
            AnnotationTool::MarkerPen(tool) => tool.ui(ui),
            AnnotationTool::Mosaic(tool) => tool.ui(ui),
            AnnotationTool::Blur(tool) => tool.ui(ui),
            // AnnotationTool::Text => {
            //     todo!("Text")
            // }
            // AnnotationTool::SerialNumber => {
            //     todo!("Serial Number")
            // }
            // AnnotationTool::Watermark => {
            //     todo!("Watermark")
            // }
            AnnotationTool::Eraser(tool) => tool.ui(ui),
        }
    }
}

/// 当前标注状态
pub struct AnnotatorState {
    /// 是否隐藏主工具条
    pub hide_primary_toolbar: bool,

    /// 背景图片
    pub background_image: Arc<RgbaImage>,

    /// 背景图片的纹理句柄
    pub background_texture_handle: Option<TextureHandle>,

    /// 用于离线渲染
    pub renderer: Arc<EguiOffScreenRender>,

    /// 标注工具
    pub annotation_tools: HashMap<ToolName, AnnotationTool>,

    /// 界面上显示的标注内容
    pub annotations_stack: Vec<Annotation>,

    /// "重做"栈：因"撤销"操作而从annotations_stack中弹出的内容会被放入这里，以支持重做
    pub redo_stack: Vec<Annotation>,

    /// 当前激活的标注工具
    pub current_annotation_tool: Option<AnnotationTool>,
}

pub type SharedAnnotatorState = Rc<RefCell<AnnotatorState>>;

impl Global for SharedAnnotatorState {}

/// 从栈顶访问T类型的标注
pub trait StackTopAccessor<T> {
    fn peek_annotation<F, R>(&self, func: F) -> Option<R>
    where
        F: Fn(Option<&T>) -> Option<R>;

    fn peek_annotation_mut<F, R>(&mut self, func: F) -> Option<R>
    where
        F: Fn(Option<&mut T>) -> Option<R>;

    fn pop_annotation(&mut self) -> Option<T>;
}

#[macro_export]
macro_rules! impl_stack_top_access_for {
    ($($tool:ty=>$annotation:ty),*) => {
        $(
            impl $tool {
                fn peek_annotation<F, R>(&self, func: F) -> Option<R>
                where
                    F: Fn(Option<&$annotation>) -> Option<R>,
                {
                    let annotator_state = self.annotator_state();
                    let annotator_state = annotator_state.borrow();
                    annotator_state.peek_annotation(func)
                }

                fn peek_annotation_mut<F, R>(&self, func: F) -> Option<R>
                where
                    F: Fn(Option<&mut $annotation>) -> Option<R>,
                {
                    let annotator_state = self.annotator_state();
                    let mut annotator_state = annotator_state.borrow_mut();
                    annotator_state.peek_annotation_mut(func)
                }

                fn pop_annotation(&self) -> Option<$annotation> {
                    let annotator_state = self.annotator_state();
                    annotator_state.borrow_mut().pop_annotation()
                }
            }
        )*
    };
}

pub trait SharedAnnotatorStateUtil {
    fn with_current_annotation_tool<F>(&self, func: F)
    where
        F: FnOnce(&mut AnnotationTool);
}

impl SharedAnnotatorStateUtil for SharedAnnotatorState {
    fn with_current_annotation_tool<F>(&self, func: F)
    where
        F: FnOnce(&mut AnnotationTool),
    {
        let mut annotator_state_mut_ref = self.borrow_mut();
        let mut current_annotation_tool = annotator_state_mut_ref
            .current_annotation_tool
            .take()
            .unwrap();
        drop(annotator_state_mut_ref);

        func(&mut current_annotation_tool);

        let mut annotator_state_mut_ref = self.borrow_mut();
        annotator_state_mut_ref
            .current_annotation_tool
            .replace(current_annotation_tool);
    }
}

impl AnnotatorState {
    pub fn annotator_panel_id() -> ViewId {
        "annotator-panel".into()
    }
    pub fn primary_toolbar_id() -> ViewId {
        "primary-toolbar".into()
    }
    pub fn secondly_toolbar_id() -> ViewId {
        "secondly-toolbar".into()
    }

    pub fn activate_annotation_tool(&mut self, tool_name: ToolName) {
        if let Some(active_tool) = &self.current_annotation_tool {
            if active_tool.tool_name() == tool_name {
                return;
            }
        }
        let tool = self
            .annotation_tools
            .remove(&tool_name)
            .expect(&format!("{:?}Tool does not exist", tool_name));
        if let Some(previous_tool) = self.current_annotation_tool.replace(tool) {
            self.annotation_tools
                .insert(previous_tool.tool_name(), previous_tool);
        }
    }

    pub fn deactivate_annotation_tool(&mut self) {
        if let Some(previous_tool) = self.current_annotation_tool.take() {
            self.annotation_tools
                .insert(previous_tool.tool_name(), previous_tool);
        }
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

/// 小矩形的默认宽度和高度
pub const DEFAULT_SIZE_FOR_SMALL_RECT: (f32, f32) = (6., 6.);

pub trait PainterExt {
    /// 将一个的扩展成一个小矩形并绘制它
    fn small_rect(&self, pos: &Pos2);

    /// 为一个矩形绘制各个角以及边上的小矩形
    fn small_rects(&self, rect: &Rect);

    /// 绘制矩形，支持填充和不同风格的边框
    ///
    /// # 参数
    /// - `painter`: egui 绘制器
    /// - `rect`: 矩形区域（填充区域）
    /// - `fill_color`: 填充颜色
    /// - `stroke`: 边框样式（颜色、宽度）
    /// - `stroke_kind`: 边框对齐方式（Inside / Outside / Middle）
    /// - `stroke_type`: 线条类型（实线 / 虚线 / 点线）
    fn rectangle(
        &self,
        rect: &Rect,
        fill_color: impl Into<Color32>,
        stroke: impl Into<Stroke>,
        stroke_kind: StrokeKind,
        stroke_type: StrokeType,
    );

    fn simple_arrow(&self, origin: Pos2, vec: Vec2, stroke: impl Into<Stroke>);
}

impl PainterExt for Painter {
    fn small_rect(&self, pos: &Pos2) {
        let painter = self;
        let width = DEFAULT_SIZE_FOR_SMALL_RECT.0;
        let height = DEFAULT_SIZE_FOR_SMALL_RECT.1;
        painter.rect(
            pos.rect(width, height),
            0,
            Color32::TRANSPARENT,
            Stroke::new(1f32, Color32::WHITE),
            StrokeKind::Middle,
        );
    }

    fn small_rects(&self, rect: &Rect) {
        let painter = self;

        let top_left_pos = rect.left_top();
        let top_right_pos = rect.right_top();
        let bottom_right_pos = rect.right_bottom();
        let bottom_left_pos = rect.left_bottom();

        let center_left_edge = pos2(top_left_pos.x, top_left_pos.y + rect.height() / 2f32);
        let center_right_edge = pos2(top_right_pos.x, top_right_pos.y + rect.height() / 2f32);
        let center_top_edge = pos2(top_left_pos.x + rect.width() / 2f32, top_left_pos.y);
        let center_bottom_edge = pos2(bottom_left_pos.x + rect.width() / 2f32, bottom_left_pos.y);

        painter.small_rect(&top_left_pos);
        painter.small_rect(&top_right_pos);
        painter.small_rect(&bottom_right_pos);
        painter.small_rect(&bottom_left_pos);

        painter.small_rect(&center_left_edge);
        painter.small_rect(&center_right_edge);
        painter.small_rect(&center_top_edge);
        painter.small_rect(&center_bottom_edge);
    }

    fn rectangle(
        &self,
        rect: &Rect,
        fill_color: impl Into<Color32>,
        stroke: impl Into<Stroke>,
        stroke_kind: StrokeKind,
        stroke_type: StrokeType,
    ) {
        let painter = self;
        let fill_color = fill_color.into();
        let stroke = stroke.into();

        // 1. 绘制填充矩形
        painter.rect_filled(*rect, 0.0, fill_color);

        // 2. 根据对齐方式计算边框路径所在的矩形
        let half_width = stroke.width / 2.0;
        let path_rect = match stroke_kind {
            StrokeKind::Inside => rect.shrink(half_width),
            StrokeKind::Outside => rect.expand(half_width),
            StrokeKind::Middle => *rect,
        };

        // 3. 绘制边框
        match stroke_type {
            StrokeType::SolidLine => {
                // 实线直接用 rect_stroke
                painter.rect_stroke(path_rect, 0.0, stroke, stroke_kind);
            }
            StrokeType::DashedLine => {
                // 虚线：使用 dashed_line，自定义 dash 和 gap 长度
                let dash_len = dash_len_for_dashed_line(stroke.width);
                let gap_len = gap_len_for_dashed_line(stroke.width);
                draw_dashed_rect(painter, path_rect, stroke, dash_len, gap_len);
            }
            StrokeType::DottedLine => {
                // 点线：使用 dotted_line，根据线宽计算点间距和半径
                let spacing = spacing_for_dotted_line(stroke.width); // 点间距
                let radius = radius_for_dotted_line(stroke.width); // 点半径
                draw_dotted_rect(painter, path_rect, stroke.color, spacing, radius);
            }
        }
    }

    fn simple_arrow(&self, origin: Pos2, vec: Vec2, stroke: impl Into<Stroke>) {
        use egui::emath::Rot2;
        let rot = Rot2::from_angle(std::f32::consts::TAU / 10.0);
        let tip_length = 12.;
        let tip = origin + vec;
        let dir = vec.normalized();
        let stroke = stroke.into();
        self.line_segment([origin, tip], stroke);
        self.line_segment([tip, tip - tip_length * (rot * dir)], stroke);
        self.line_segment([tip, tip - tip_length * (rot.inverse() * dir)], stroke);
    }
}

pub fn dash_len_for_dashed_line(stroke_width: f32) -> f32 {
    let dash_len = if stroke_width * 3. < 6. {
        6.
    } else {
        stroke_width * 3.
    };
    dash_len
}

pub fn gap_len_for_dashed_line(stroke_width: f32) -> f32 {
    let gap_len = if stroke_width * 3. < 6. {
        6.
    } else {
        stroke_width * 3.
    };
    gap_len
}

pub fn spacing_for_dotted_line(stroke_width: f32) -> f32 {
    let spacing = stroke_width * 2.0; // 点间距
    if spacing < 6. { 6. } else { spacing }
}

pub fn radius_for_dotted_line(stroke_width: f32) -> f32 {
    let radius = stroke_width / 2.0;
    radius
}

/// 绘制矩形的虚线边框
fn draw_dashed_rect(painter: &Painter, rect: Rect, stroke: Stroke, dash_len: f32, gap_len: f32) {
    let [left, right, top, bottom] = [rect.left(), rect.right(), rect.top(), rect.bottom()];

    let edges = [
        (Pos2::new(left, top), Pos2::new(right, top)), // 上边
        (Pos2::new(right, top), Pos2::new(right, bottom)), // 右边
        (Pos2::new(right, bottom), Pos2::new(left, bottom)), // 下边
        (Pos2::new(left, bottom), Pos2::new(left, top)), // 左边
    ];

    for (start, end) in edges {
        let shape = Shape::dashed_line(&[start, end], stroke, dash_len, gap_len);
        painter.add(shape);
    }
}

/// 绘制矩形的点线边框
fn draw_dotted_rect(painter: &Painter, rect: Rect, color: Color32, spacing: f32, radius: f32) {
    let [left, right, top, bottom] = [rect.left(), rect.right(), rect.top(), rect.bottom()];

    let edges = [
        (Pos2::new(left, top), Pos2::new(right, top)), // 上边
        (Pos2::new(right, top), Pos2::new(right, bottom)), // 右边
        (Pos2::new(right, bottom), Pos2::new(left, bottom)), // 下边
        (Pos2::new(left, bottom), Pos2::new(left, top)), // 左边
    ];

    for (start, end) in edges {
        // dotted_line 返回 Vec<Shape>，需要逐个添加
        let shapes = Shape::dotted_line(&[start, end], color, spacing, radius);
        for shape in shapes {
            painter.add(shape);
        }
    }
}
