use crate::annotator::rectangle_based::{EllipseTool, RectangleTool};
use crate::annotator::{AnnotationTool, AnnotatorState, SharedAnnotatorState, SharedAnnotatorStateUtil, ToolName};
use crate::application::Application;
use crate::dpi::{LogicalPosition, PhysicalSize};
use crate::global::{ReadGlobalMut, ReadOrInsertGlobal};
use crate::view::ViewId;
use crate::window::AppWindow;
use egui::load::SizedTexture;
use egui::{pos2, vec2, ColorImage, Frame, Image, ImageSource, Rect};
use image::RgbaImage;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use crate::annotator::free_line_based::{MarkerPenTool, PencilTool};
use crate::annotator::image_based::MosaicTool;
use crate::annotator::straight_line_based::{ArrowTool, StraightLineTool};

pub fn create_annotator_panel(
    view_id: ViewId,
    app: &mut Application,
    window: &mut AppWindow,
    image: Arc<RgbaImage>,
) {
    let global_state = &app.global_state;

    let scale_factor = window.scale_factor().unwrap();
    let panel_size = PhysicalSize::new(image.width(), image.height())
        .to_logical(scale_factor);

    let logical_position = LogicalPosition::new(0, 0);

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
                    let mut annotator_state = AnnotatorState::default();
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
                    annotator_state.background_texture_handle = Some(texture_handle);
                    annotator_state.background_image = image.clone();

                    let annotator_state_rc = Rc::new(RefCell::new(annotator_state));
                    let rectangle_tool = RectangleTool::new(Rc::downgrade(&annotator_state_rc));
                    let ellipse_tool = EllipseTool::new(Rc::downgrade(&annotator_state_rc));
                    let straight_line_tool = StraightLineTool::new(Rc::downgrade(&annotator_state_rc));
                    let arrow_tool = ArrowTool::new(Rc::downgrade(&annotator_state_rc));
                    let pencil_tool = PencilTool::new(Rc::downgrade(&annotator_state_rc));
                    let marker_pen_tool = MarkerPenTool::new(Rc::downgrade(&annotator_state_rc));
                    let mosaic_tool = MosaicTool::new(Rc::downgrade(&annotator_state_rc));

                    annotator_state_rc.borrow_mut().annotation_tools.insert(ToolName::Rectangle, AnnotationTool::Rectangle(rectangle_tool));
                    annotator_state_rc.borrow_mut().annotation_tools.insert(ToolName::Ellipse, AnnotationTool::Ellipse(ellipse_tool));
                    annotator_state_rc.borrow_mut().annotation_tools.insert(ToolName::StraightLine, AnnotationTool::StraightLine(straight_line_tool));
                    annotator_state_rc.borrow_mut().annotation_tools.insert(ToolName::Arrow, AnnotationTool::Arrow(arrow_tool));
                    annotator_state_rc.borrow_mut().annotation_tools.insert(ToolName::Pencil, AnnotationTool::Pencil(pencil_tool));
                    annotator_state_rc.borrow_mut().annotation_tools.insert(ToolName::MarkerPen, AnnotationTool::MarkerPen(marker_pen_tool));
                    annotator_state_rc.borrow_mut().annotation_tools.insert(ToolName::Mosaic, AnnotationTool::Mosaic(mosaic_tool));

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

                        let frame_size = PhysicalSize::new(image_width, image_height)
                            .to_logical(ctx.pixels_per_point() as f64);

                        bg_image.paint_at(
                            ui,
                            Rect::from_min_size(
                                pos2(0., 0.),
                                vec2(frame_size.width, frame_size.height),
                            ),
                        );

                        let annotator_state = window
                            .window_context
                            .globals_by_type
                            .require_ref_mut::<SharedAnnotatorState>()
                            .clone();

                        if annotator_state.borrow().current_annotation_tool.is_some() {
                            annotator_state.with_current_annotation_tool(|tool| {
                                ui.add(tool);
                            })
                        }

                        annotator_state.borrow_mut()
                            .annotations_stack
                            .iter_mut()
                            .for_each(|annotation| {
                                ui.add(annotation);
                            });

                        // Area::new(Id::from("text_edit")).movable(true).current_pos(annotator_state.pos).show(ctx, |ui| {
                        //     let response = ui.add(TextEdit::multiline(&mut annotator_state.editing_text)
                        //         .background_color(Color32::TRANSPARENT));
                        //     let vec = response.drag_motion();
                        //     annotator_state.pos.x += vec.x;
                        //     annotator_state.pos.y += vec.y;
                        // });
                        //
                        // ui.painter().clip_rect()


                    });
            })
        }),
        None,
    );
}
