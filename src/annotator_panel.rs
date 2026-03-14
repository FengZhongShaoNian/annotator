use crate::annotator::free_line_based::{MarkerPenTool, PencilTool};
use crate::annotator::image_based::{
    BackgroundImageWithAnnotationsProvider, BlurHandler, BlurTool, EraserTool, ExtractHandler,
    MosaicHandler, MosaicTool, OriginalBackgroundImageProvider,
};
use crate::annotator::rectangle_based::{EllipseTool, RectangleTool};
use crate::annotator::serial_number::SerialNumberTool;
use crate::annotator::straight_line_based::{ArrowTool, StraightLineTool};
use crate::annotator::text::TextTool;
use crate::annotator::{
    AnnotationTool, AnnotatorState, ExtraZoomFactorSupport, SharedAnnotatorState,
    SharedAnnotatorStateUtil, ToolName, WheelHandler, ApplyExtraZoomFactor
};
use crate::application::Application;
use crate::context::Command;
use crate::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use crate::egui_off_screen_render::EguiOffScreenRender;
use crate::global::{ReadGlobalMut, ReadOrInsertGlobal};
use crate::view::ViewId;
use crate::window::AppWindow;
use egui::load::SizedTexture;
use egui::{Area, Color32, ColorImage, Frame, Image, ImageSource, Rect, pos2, vec2};
use image::RgbaImage;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use log::info;

pub fn create_annotator_panel(
    view_id: ViewId,
    app: &mut Application,
    window: &mut AppWindow,
    image: Arc<RgbaImage>,
    logical_position: LogicalPosition<i32>,
) {
    let global_state = &app.global_state;

    let scale_factor = window.scale_factor().unwrap();
    let panel_size = PhysicalSize::new(image.width(), image.height()).to_logical(scale_factor);

    window.create_sub_surface_view(
        view_id,
        global_state,
        panel_size,
        logical_position,
        Box::new(move |input, egui_ctx, app, window, current_view| {
            let image_width = image.width();
            let image_height = image.height();
            // 将图像数据上传到 GPU 并获取纹理句柄
            let annotator_state: &SharedAnnotatorState = window
                .window_context
                .globals_by_type
                .get_global_or_insert_with(|| {
                    // 创建 ColorImage
                    // 注意：RgbaImage 的 bytes 应该是连续的 RGBA 数据
                    let background_image = Arc::new(ColorImage::from_rgba_premultiplied(
                        [image_width as usize, image_height as usize],
                        image.as_raw(),
                    ));
                    // Load the texture only once.
                    let texture_handle = egui_ctx.load_texture(
                        "background-image",
                        egui::ImageData::Color(background_image),
                        Default::default(),
                    );

                    let renderer = Arc::new(EguiOffScreenRender::new(
                        app.global_state.gpu.borrow().as_ref().unwrap(),
                    ));

                    let annotator_state = AnnotatorState {
                        extra_zoom_factor: 1.0,
                        hide_primary_toolbar: false,
                        background_image: image.clone(),
                        background_texture_handle: Some(texture_handle),
                        renderer: renderer.clone(),
                        annotation_tools: Default::default(),
                        annotations_stack: vec![],
                        redo_stack: vec![],
                        current_annotation_tool: None,
                        candidate_colors: vec![
                            Color32::RED,
                            Color32::YELLOW,
                            Color32::GREEN,
                            Color32::BLUE,
                            Color32::BLACK,
                            Color32::WHITE,
                            Color32::GRAY,
                            Color32::DARK_RED,
                            Color32::LIGHT_RED,
                            Color32::CYAN,
                            Color32::MAGENTA,
                            Color32::DARK_GREEN,
                            Color32::LIGHT_GREEN,
                            Color32::DARK_BLUE,
                            Color32::LIGHT_BLUE,
                        ],
                    };

                    let annotator_state_rc = Rc::new(RefCell::new(annotator_state));
                    let rectangle_tool = RectangleTool::new(Rc::downgrade(&annotator_state_rc));
                    let ellipse_tool = EllipseTool::new(Rc::downgrade(&annotator_state_rc));
                    let straight_line_tool =
                        StraightLineTool::new(Rc::downgrade(&annotator_state_rc));
                    let arrow_tool = ArrowTool::new(Rc::downgrade(&annotator_state_rc));
                    let pencil_tool = PencilTool::new(Rc::downgrade(&annotator_state_rc));
                    let marker_pen_tool = MarkerPenTool::new(Rc::downgrade(&annotator_state_rc));
                    let mosaic_tool = MosaicTool::new(
                        Rc::downgrade(&annotator_state_rc),
                        Box::new(BackgroundImageWithAnnotationsProvider::new(
                            renderer.clone(),
                        )),
                        Rc::new(MosaicHandler::new(10)),
                    );
                    let blur_tool = BlurTool::new(
                        Rc::downgrade(&annotator_state_rc),
                        Box::new(BackgroundImageWithAnnotationsProvider::new(renderer)),
                        Rc::new(BlurHandler::default()),
                    );
                    let eraser_tool = EraserTool::new(
                        Rc::downgrade(&annotator_state_rc),
                        Box::new(OriginalBackgroundImageProvider::new()),
                        Rc::new(ExtractHandler::new()),
                    );
                    let text_tool = TextTool::new(Rc::downgrade(&annotator_state_rc));
                    let serial_number_tool =
                        SerialNumberTool::new(Rc::downgrade(&annotator_state_rc));

                    annotator_state_rc.borrow_mut().annotation_tools.insert(
                        ToolName::Rectangle,
                        AnnotationTool::Rectangle(rectangle_tool),
                    );
                    annotator_state_rc
                        .borrow_mut()
                        .annotation_tools
                        .insert(ToolName::Ellipse, AnnotationTool::Ellipse(ellipse_tool));
                    annotator_state_rc.borrow_mut().annotation_tools.insert(
                        ToolName::StraightLine,
                        AnnotationTool::StraightLine(straight_line_tool),
                    );
                    annotator_state_rc
                        .borrow_mut()
                        .annotation_tools
                        .insert(ToolName::Arrow, AnnotationTool::Arrow(arrow_tool));
                    annotator_state_rc
                        .borrow_mut()
                        .annotation_tools
                        .insert(ToolName::Pencil, AnnotationTool::Pencil(pencil_tool));
                    annotator_state_rc.borrow_mut().annotation_tools.insert(
                        ToolName::MarkerPen,
                        AnnotationTool::MarkerPen(marker_pen_tool),
                    );
                    annotator_state_rc
                        .borrow_mut()
                        .annotation_tools
                        .insert(ToolName::Mosaic, AnnotationTool::Mosaic(mosaic_tool));
                    annotator_state_rc
                        .borrow_mut()
                        .annotation_tools
                        .insert(ToolName::Blur, AnnotationTool::Blur(blur_tool));
                    annotator_state_rc
                        .borrow_mut()
                        .annotation_tools
                        .insert(ToolName::Eraser, AnnotationTool::Eraser(eraser_tool));
                    annotator_state_rc
                        .borrow_mut()
                        .annotation_tools
                        .insert(ToolName::Text, AnnotationTool::Text(text_tool));
                    annotator_state_rc.borrow_mut().annotation_tools.insert(
                        ToolName::SerialNumber,
                        AnnotationTool::SerialNumber(serial_number_tool),
                    );

                    annotator_state_rc
                });

            // 将图像数据上传到 GPU 并获取纹理句柄
            let annotator_state = annotator_state.borrow_mut();
            let texture_handle = annotator_state.background_texture_handle.as_ref().unwrap();
            let texture_handle = texture_handle.clone();
            drop(annotator_state);

            // 构建 UI 的具体内容
            egui_ctx.run(input, move |ctx| {
                egui::CentralPanel::default()
                    .frame(Frame::new())
                    .show(ctx, |ui| {
                        let bg_image = Image::new(ImageSource::Texture(SizedTexture::from_handle(
                            &texture_handle,
                        )));

                        let annotator_state = window
                            .window_context
                            .globals_by_type
                            .require_ref_mut::<SharedAnnotatorState>()
                            .clone();

                        let scale_factor = ui.ctx().pixels_per_point();
                        let extra_zoom_factor = annotator_state.borrow().extra_zoom_factor;
                        ui.ctx().set_extra_zoom_factor(extra_zoom_factor);

                        // 标注面板的逻辑大小=背景图片的逻辑大小=(背景图片的物理大小/scale_factor) * extra_zoom_factor)
                        let logical_size = PhysicalSize::new(image_width, image_height)
                            .to_logical(scale_factor as f64)
                            .apply_extra_zoom_factor_with_ctx(ui.ctx());

                        let panel_size = vec2(logical_size.width, logical_size.height);

                        bg_image.paint_at(ui, Rect::from_min_size(pos2(0., 0.), panel_size));

                        annotator_state
                            .borrow_mut()
                            .annotations_stack
                            .iter_mut()
                            .for_each(|annotation| {
                                ui.add(annotation);
                            });

                        if annotator_state.borrow().current_annotation_tool.is_some() {
                            Area::new("annotation_tool_area".into())
                                .fixed_pos(pos2(0., 0.))
                                .show(ctx, |ui| {
                                    ui.set_width(panel_size.x);
                                    ui.set_height(panel_size.y);
                                    annotator_state.with_current_annotation_tool(|tool| {
                                        ui.add(tool);
                                    })
                                });
                        } else {
                            // 缩放窗口
                            let zoom_before = annotator_state.borrow().extra_zoom_factor;
                            annotator_state.borrow_mut().handle_wheel_event(ui);
                            let zoom_after = annotator_state.borrow().extra_zoom_factor;

                            if zoom_before != zoom_after {
                                info!("extra_zoom_factor changed from {} to {}", zoom_before, zoom_after);

                                // 标注面板的逻辑大小=背景图片的逻辑大小=(背景图片的物理大小/scale_factor) * extra_zoom_factor)
                                let new_size = PhysicalSize::new(image_width, image_height)
                                    .to_logical(ctx.pixels_per_point() as f64)
                                    .apply_extra_zoom_factor(zoom_after);

                                info!("标注面板尺寸更改from {:?} to {:?}", logical_size, new_size);

                                window
                                    .window_context
                                    .commands
                                    .push_back(Command::ResizeView(current_view.id(), new_size));
                            }
                        }
                    });
            })
        }),
        None,
    );
}

impl WheelHandler for AnnotatorState {
    fn on_scroll_delta_changed(&mut self, value: f32) {
        if value > 0. {
            let zoom = self.extra_zoom_factor - 0.1;
            if zoom >= 0.1 {
                self.extra_zoom_factor = zoom;
            }
        } else {
            let zoom = self.extra_zoom_factor + 0.1;
            self.extra_zoom_factor = zoom;
        }
    }
}
