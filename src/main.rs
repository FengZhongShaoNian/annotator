mod application;
mod egui_input;
mod gpu;
mod window;

mod wp_fractional_scaling;

mod annotator;
mod context;
mod dpi;
mod font;
mod global;
mod icon;
mod sub_surface_view;
mod surface_view;
mod view;
mod wp_viewporter;
mod xdg_popup_view;
mod primary_toolbar;
mod secondly_toolbar;

use crate::annotator::ellipse::{EllipseAnnotationTool, EllipseState};
use crate::annotator::rectangle::{
    RectangleAnnotationTool, RectangleAnnotationToolState, RectangleState,
};
use crate::annotator::svg_button::SvgButton;
use crate::annotator::{Annotation, AnnotatorState, StrokeType, ToolType};
use crate::application::Application;
use crate::context::Command;
use crate::dpi::{LogicalBounds, LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use crate::global::{ReadGlobalMut, ReadOrInsertGlobal};
use crate::icon::Icons;
use crate::window::WindowConfiguration;
use anyhow::Context;
use egui::load::SizedTexture;
use egui::{Color32, ColorImage, Frame, Image, ImageSource, Rect, Shadow, pos2, vec2};
use log::error;
use std::any::{TypeId, type_name};
use std::env;
use std::ops::Not;
use std::sync::Arc;
use crate::primary_toolbar::create_primary_toolbar;
use crate::secondly_toolbar::create_secondly_toolbar;
use crate::view::ViewId;

fn main() {
    env_logger::init();

    let mut app = Application::new("site.nullable.annotator");

    for path in env::args_os().skip(1) {
        let image = match image::open(&path) {
            Ok(i) => i,
            Err(e) => {
                error!("Failed to open image {}.", path.to_string_lossy());
                error!("Error was: {e:?}");
                return;
            }
        };

        // We'll need the image in RGBA for drawing it
        let image = image.to_rgba8();
        let window_config = WindowConfiguration::new(
            app.app_id.to_string(),
            "".to_string(),
            LogicalSize::new(800, 600),
            Some(PhysicalSize::new(image.width(), image.height())),
        );
        let window_id = app.open_window(
            window_config,
            Box::new(move |input, egui_ctx, app, window, current_view| {
                let image_width = image.width();
                let image_height = image.height();
                // 将图像数据上传到 GPU 并获取纹理句柄
                let annotator_state: &AnnotatorState = window
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

                        annotator_state.current_annotation_tool = Some(ToolType::Rectangle);

                        annotator_state
                    });

                // 将图像数据上传到 GPU 并获取纹理句柄
                let texture_handle = annotator_state.background_texture_handle.as_ref().unwrap();
                let texture_handle = texture_handle.clone();

                // 构建 UI 的具体内容
                egui_ctx.run(input, move |ctx| {
                    egui::CentralPanel::default()
                        .frame(Frame::new())
                        .show(ctx, |ui| {
                            let bg_image = Image::new(ImageSource::Texture(
                                SizedTexture::from_handle(&texture_handle),
                            ));

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
                                .require_ref_mut::<AnnotatorState>();

                            match annotator_state.current_annotation_tool {
                                Some(ToolType::Rectangle) => {
                                    ui.add(RectangleAnnotationTool::new(annotator_state));
                                }

                                Some(ToolType::Ellipse) => {
                                    ui.add(EllipseAnnotationTool::new(annotator_state));
                                }

                                _ => {}
                            }

                            annotator_state
                                .annotations_stack
                                .iter()
                                .for_each(|annotation| {
                                    if let Some(rectangle_annotation) =
                                        annotation.downcast_ref::<RectangleState>()
                                    {
                                        rectangle_annotation.show(ui);
                                    } else if let Some(ellipse_annotation) =
                                        annotation.downcast_ref::<EllipseState>()
                                    {
                                        ellipse_annotation.show(ui);
                                    }
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

                            let primary_toolbar_id = AnnotatorState::primary_toolbar_id();
                            if !annotator_state.hide_primary_toolbar && !window.views.contains_key(&primary_toolbar_id) {
                                create_primary_toolbar(primary_toolbar_id, app, window, current_view.size());
                                let secondly_toolbar = AnnotatorState::secondly_toolbar_id();
                                if  !window.views.contains_key(&secondly_toolbar){
                                    create_secondly_toolbar(secondly_toolbar, app, window, current_view.size());
                                }
                            }
                        });
                })
            }),
        );




        //
        // app.with_window_mut(window_id, |global_state, window| {
        //     let window = window.as_mut().unwrap();
        //     window.create_xdg_popup_view(
        //         "popup".into(),
        //         global_state,
        //         LogicalSize::new(200, 48),
        //         LogicalPosition::new(0, 100),
        //         Box::new(|input, egui_ctx, app, window| {
        //             // 构建 UI 的具体内容
        //             egui_ctx.run(input, move |ctx| {
        //                 egui::CentralPanel::default()
        //                     .frame(Frame::new().fill(Color32::from_hex("#393b40").unwrap()))
        //                     .show(ctx, |ui| {
        //                         ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
        //                         ui.spacing_mut().item_spacing = vec2(1.0, 0.0);
        //
        //                         let current_view_id =
        //                             window.window_context.current_view_id.clone().unwrap();
        //                         let annotator_state = window
        //                             .window_context
        //                             .globals_by_type
        //                             .require_ref_mut::<AnnotatorState>();
        //                         let active_tool = annotator_state.current_annotation_tool;
        //
        //                         if active_tool.is_none() {
        //                             window
        //                                 .window_context
        //                                 .commands
        //                                 .push_back(Command::HideView(current_view_id));
        //                             return;
        //                         }
        //
        //                         if matches!(active_tool, Some(ToolType::Rectangle)) {
        //                             let tool_state =
        //                                 &mut annotator_state.rectangle_annotation_tool_state;
        //
        //                             let label = match tool_state.style.stroke_type {
        //                                 StrokeType::SolidLine => "实线",
        //                                 StrokeType::DashedLine => "虚线",
        //                                 StrokeType::DottedLine => "点线",
        //                             };
        //                             let stroke_type = egui::ComboBox::from_label("");
        //                             stroke_type.selected_text(label).show_ui(ui, |ui| {
        //                                 if ui.label("实线").clicked() {
        //                                     tool_state.style.stroke_type = StrokeType::SolidLine;
        //                                 }
        //                                 if ui.label("虚线").clicked() {
        //                                     tool_state.style.stroke_type = StrokeType::DashedLine;
        //                                 }
        //                                 if ui.label("点线").clicked() {
        //                                     tool_state.style.stroke_type = StrokeType::DottedLine;
        //                                 }
        //                             });
        //                         }
        //                     });
        //             })
        //         }),
        //     );
        // });
    }

    app.run();
}
