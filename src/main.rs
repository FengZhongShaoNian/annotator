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

use crate::annotator::rectangle::{RectangleAnnotationTool, RectangleState};
use crate::annotator::{Annotation, AnnotatorState, ToolType};
use crate::application::Application;
use crate::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use crate::window::WindowConfiguration;
use egui::load::SizedTexture;
use egui::{ColorImage, Image, ImageSource, Rect, pos2, vec2};
use log::error;
use std::env;
use std::sync::Arc;

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
            LogicalSize::new(800, 600),
            Some(PhysicalSize::new(image.width(), image.height())),
        );
        let window_id = app.open_window(
            window_config,
            Box::new(move |input, egui_ctx, window_ctx| {
                let image_width = image.width();
                let image_height = image.height();
                // 将图像数据上传到 GPU 并获取纹理句柄
                let annotator_state: &AnnotatorState = window_ctx.get_global_or_insert_with(|| {
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

                let annotator_state = window_ctx.global_mut::<AnnotatorState>();

                // 构建 UI 的具体内容
                egui_ctx.run(input, move |ctx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::new())
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

                            match annotator_state.current_annotation_tool {
                                Some(ToolType::Rectangle) => {
                                    ui.add(RectangleAnnotationTool::new(annotator_state));
                                }

                                _ => {}
                            }

                            annotator_state
                                .annotations_stack
                                .iter()
                                .for_each(|annotation| {
                                    annotation
                                        .downcast_ref::<RectangleState>()
                                        .unwrap()
                                        .show(ui);
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
        );

        let position_calculator = Arc::new(
            |parent_surface_size: &PhysicalSize<u32>, subview_size: &PhysicalSize<u32>| {
                let subview_width = &subview_size.width;
                PhysicalPosition::new(
                    parent_surface_size.width - subview_width,
                    parent_surface_size.height + 10,
                )
            },
        );

        app.with_window_mut(window_id, |global_state, window| {
            let window = window.as_mut().unwrap();
            // 创建工具条
            window.create_sub_surface_view(
                global_state,
                LogicalSize::new(600, 38),
                LogicalPosition::new(0i32, 0i32),
                Box::new(|input, egui_ctx, annotator_ctx| {
                    // 构建 UI 的具体内容
                    egui_ctx.run(input, |ctx| {
                        egui::CentralPanel::default()
                            .frame(egui::Frame::new())
                            .show(ctx, |ui| {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                                if ui.button("Line").clicked() {
                                    println!("点击了直线工具 ");
                                }
                            });
                    })
                }),
                Some(position_calculator),
            );
        });
    }

    app.run();
}
