use crate::annotator::{ActivationSupport, Annotation, AnnotationCommon, AnnotatorState, FillColorSupport, Paint, StrokeColorSupport, StrokeType, StrokeTypeSupport, StrokeWidthSupport};
use crate::dpi::{LogicalBounds, LogicalSize, PhysicalBounds};
use egui::{pos2, Color32, ColorImage, CursorIcon, Frame, Image, ImageSource, Painter, Pos2, Rect, Response, Sense, TextureHandle, Ui, Widget};
use image::GenericImageView;
use image::{Rgba, RgbaImage};
use std::cell::RefCell;
use std::cmp::{max, min};
use std::default::Default;
use std::rc::{Rc, Weak};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use egui::load::SizedTexture;
use crate::egui_off_screen_render::render_egui_to_image;
use crate::gpu::GpuContext;

fn mosaic(image: &RgbaImage, bounds: PhysicalBounds<u32>, block_size: u32) -> Result<RgbaImage, image::ImageError> {
    // 转换为 RgbImage 以便直接修改像素
    let x = max(bounds.origin.x, 0);
    let y = max(bounds.origin.y, 0);
    let width = min(bounds.size.width, image.width());
    let height = min(bounds.size.height, image.height());

    let sub_image = image.view(x, y, width, height);
    let mut result = RgbaImage::new(width, height);

    // 遍历每个块
    for y in (0..height).step_by(block_size as usize) {
        for x in (0..width).step_by(block_size as usize) {
            // 确定当前块的实际边界（防止超出图像边缘）
            let block_width = block_size.min(width - x);
            let block_height = block_size.min(height - y);

            // 计算块内所有像素的 RGB 总和
            let (mut r_sum, mut g_sum, mut b_sum, mut a_sum) = (0u64, 0u64, 0u64, 0u64);
            let mut count = 0;

            for dy in 0..block_height {
                for dx in 0..block_width {
                    let pixel = sub_image.get_pixel(x + dx, y + dy);
                    r_sum += pixel[0] as u64;
                    g_sum += pixel[1] as u64;
                    b_sum += pixel[2] as u64;
                    a_sum += pixel[3] as u64;
                    count += 1;
                }
            }

            // 计算平均颜色
            let avg_r = (r_sum / count) as u8;
            let avg_g = (g_sum / count) as u8;
            let avg_b = (b_sum / count) as u8;
            let avg_a = (a_sum / count) as u8;
            let avg_color = Rgba([avg_r, avg_g, avg_b, avg_a]);

            // 用平均颜色填充整个块
            for dy in 0..block_height {
                for dx in 0..block_width {
                    result.put_pixel(x + dx, y + dy, avg_color);
                }
            }
        }
    }

    Ok(result)
}

#[derive(Clone)]
pub struct ImageBasedAnnotation<T: Default + Clone> {
    _style: T,
    rect: Rect,
    background_image: Arc<RgbaImage>,
    mosaic_texture_handle: Option<TextureHandle>,
    activation: ActivationSupport
}

impl<T: Clone + Default> ImageBasedAnnotation<T> {
    pub fn new(rect: Rect, background_image: Arc<RgbaImage>) -> Self {
        Self {
            _style: Default::default(),
            rect,
            background_image,
            mosaic_texture_handle: None,
            activation: ActivationSupport::NotSupported,
        }
    }
}

static TEXTURE_COUNTER: AtomicUsize = AtomicUsize::new(0);

impl<T: Clone + Default> Paint for ImageBasedAnnotation<T> {
    fn paint_with(&mut self, painter: &Painter) {
        let scale_factor = painter.ctx().pixels_per_point();
        let logical_bounds = LogicalBounds::new(self.rect.min.x, self.rect.min.y, self.rect.width(), self.rect.height());
        let physical_bounds: PhysicalBounds<u32> = logical_bounds.to_physical(scale_factor as f64);

        let block_size = 10;

        if physical_bounds.size.width < block_size || physical_bounds.size.height < block_size {
            return;
        }

        if let Some(texture_handle) = self.mosaic_texture_handle.clone() {
            let texture_id = texture_handle.id();
            painter.image(texture_id, self.rect, Rect::from_two_pos(pos2(0., 0.), pos2(1., 1.)), Color32::WHITE);
        }else {
            if let Ok(mosaic_image) = mosaic(&*self.background_image, physical_bounds, 10) {
                let color_image = Arc::new(ColorImage::from_rgba_premultiplied(
                    [mosaic_image.width() as usize, mosaic_image.height() as usize],
                    mosaic_image.as_raw(),
                ));
                let texture_handle = painter.ctx().load_texture(
                    format!("mosaic-{}", TEXTURE_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)),
                    egui::ImageData::Color(color_image),
                    Default::default(),
                );
                self.mosaic_texture_handle = Some(texture_handle.clone());
                let texture_id = texture_handle.id();
                painter.image(texture_id, self.rect, Rect::from_two_pos(pos2(0., 0.), pos2(1., 1.)), Color32::WHITE);
            }
        }
    }
}

impl<T: Clone + Default> StrokeWidthSupport for ImageBasedAnnotation<T> {
    fn supports_get_stroke_width(&self) -> bool {
        false
    }

    fn stroke_width(&self) -> f32 {
        unimplemented!()
    }

    fn supports_set_stroke_width(&self) -> bool {
        false
    }

    fn set_stroke_width(&mut self, _stroke_width: f32) {
        unimplemented!()
    }
}

impl<T: Clone + Default> StrokeColorSupport for ImageBasedAnnotation<T> {
    fn supports_get_stroke_color(&self) -> bool {
        false
    }

    fn stroke_color(&self) -> Color32 {
        unimplemented!()
    }

    fn supports_set_stroke_color(&self) -> bool {
        false
    }

    fn set_stroke_color(&mut self, _color: Color32) {
        unimplemented!()
    }
}

impl<T: Clone + Default> StrokeTypeSupport for ImageBasedAnnotation<T> {
    fn supports_get_stroke_type(&self) -> bool {
        false
    }

    fn stroke_type(&self) -> StrokeType {
        unimplemented!()
    }

    fn supports_set_stroke_type(&self) -> bool {
        false
    }

    fn set_stroke_type(&mut self, _stroke_type: StrokeType) {
        unimplemented!()
    }
}

impl<T: Clone + Default> FillColorSupport for ImageBasedAnnotation<T> {
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

impl<T: Clone + Default> AnnotationCommon for ImageBasedAnnotation<T> {
    fn activation(&self) -> &ActivationSupport {
        &self.activation
    }

    fn activation_mut(&mut self) -> &mut ActivationSupport {
        &mut self.activation
    }
}

#[derive(Default, Clone)]
pub struct MosaicStyle {}

#[derive(Default, Clone)]
pub struct BlurStyle {}

pub type MosaicAnnotation = ImageBasedAnnotation<MosaicStyle>;
pub type BlurAnnotation = ImageBasedAnnotation<BlurStyle>;

#[derive(Default)]
pub struct ImageBasedToolState<S>
where
    S: Default + Clone,
{
    /// 样式
    style: S,
    /// 拖动的起点
    drag_start_pos: Option<Pos2>
}

pub struct ImageBasedTool<S: Default + Clone> {
    annotator_state: Weak<RefCell<AnnotatorState>>,
    tool_state: ImageBasedToolState<S>,
    gpu_context: GpuContext
}

impl<S: Default + Clone> ImageBasedTool<S> {
    pub fn new(annotator_state: Weak<RefCell<AnnotatorState>>, gpu_context: GpuContext) -> Self {
        Self {
            annotator_state,
            tool_state: Default::default(),
            gpu_context
        }
    }
}

pub type MosaicTool = ImageBasedTool<MosaicStyle>;

impl Into<Annotation> for MosaicAnnotation {
    fn into(self) -> Annotation {
        Annotation::Mosaic(self)
    }
}

impl Widget for &mut MosaicTool {
    fn ui(self, ui: &mut Ui) -> Response {
        let sense_area = Rect::from_min_size(Pos2::ZERO, ui.available_size());
        let response = ui.allocate_rect(sense_area, Sense::click_and_drag());

        let Some(pointer_pos) = ui.ctx().pointer_hover_pos() else {
            return response;
        };

        ui.ctx().set_cursor_icon(CursorIcon::Crosshair);

        if response.drag_started() {
            let drag_started_pos = ui.ctx().input(|i| i.pointer.press_origin()).unwrap();
            self.tool_state.drag_start_pos = Some(drag_started_pos);

            let virtual_screen_size = LogicalSize::new(sense_area.width(), sense_area.height()).cast();
            let annotator_state = self.annotator_state.upgrade().unwrap();
            let texture_handle = Rc::new(annotator_state.borrow().background_texture_handle.clone().unwrap());
            let annotations = annotator_state.borrow().annotations_stack.clone();
            let image = render_egui_to_image(&self.gpu_context, virtual_screen_size, ui.ctx().pixels_per_point(), Box::new(move |input,context|{
                let mut annotaions = annotations;
                context.run(input, move |ctx| {
                    egui::CentralPanel::default()
                        .frame(Frame::new())
                        .show(ctx, |ui| {
                            // let bg_image = Image::new(ImageSource::Texture(SizedTexture::from_handle(
                            //     &texture_handle.clone(),
                            // )));
                            // bg_image.paint_at(ui, Rect::from_min_size(pos2(0., 0.), ui.available_size()));

                            annotaions
                                .iter_mut()
                                .for_each(|annotation| {
                                    annotation.paint_with(ui.painter());
                                });
                        });
                })
            }));
            // image.save("/home/one/Pictures/saved-image.png").unwrap();
        }

        if response.dragged() {
            // 拖动中
            let drag_started_pos = self.tool_state.drag_start_pos.unwrap();
            let rect = Rect::from_two_pos(drag_started_pos, pointer_pos);
            let annotator_state = self.annotator_state.upgrade().unwrap();
            let background_image = annotator_state.borrow().background_image.clone();
            let mut annotation = MosaicAnnotation::new(rect, background_image);
            annotation.paint_with(ui.painter());
        }

        if response.drag_stopped() {
            // 拖动结束
            let drag_started_pos = self.tool_state.drag_start_pos.unwrap();
            let rect = Rect::from_two_pos(drag_started_pos, pointer_pos);
            let annotator_state = self.annotator_state.upgrade().unwrap();
            let background_image = annotator_state.borrow().background_image.clone();
            let annotation = MosaicAnnotation::new(rect, background_image);
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