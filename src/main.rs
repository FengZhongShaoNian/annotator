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

use std::any::{type_name, TypeId};
use crate::annotator::rectangle::{RectangleAnnotationTool, RectangleState};
use crate::annotator::ellipse::{EllipseAnnotationTool, EllipseState};
use crate::annotator::svg_button::SvgButton;
use crate::annotator::{Annotation, AnnotatorState, ToolType};
use crate::application::Application;
use crate::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use crate::icon::Icons;
use crate::window::WindowConfiguration;
use egui::load::SizedTexture;
use egui::{Color32, ColorImage, Frame, Image, ImageSource, Rect, Shadow, pos2, vec2};
use log::error;
use std::env;
use std::sync::Arc;
use anyhow::Context;
use crate::context::Command;

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
                                    if let Some(rectangle_annotation) = annotation.downcast_ref::<RectangleState>() {
                                        rectangle_annotation.show(ui);
                                    }else if let Some(ellipse_annotation) = annotation.downcast_ref::<EllipseState>() {
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
                        });
                })
            }),
        );

        app.with_window_mut(window_id.clone(), |global_state, window| {
            let window = window.as_mut().unwrap();

            let position_calculator = Arc::new(
                |parent_surface_size: &PhysicalSize<u32>, subview_size: &PhysicalSize<u32>| {
                    let subview_width = &subview_size.width;
                    PhysicalPosition::new(
                        parent_surface_size.width - subview_width,
                        parent_surface_size.height + 10,
                    )
                },
            );

            // 创建工具条
            window.create_sub_surface_view(
                "primary-toolbar".into(),
                global_state,
                LogicalSize::new(600, 32),
                LogicalPosition::new(0i32, 0i32),
                Box::new(|input, egui_ctx, window_ctx| {
                    // 构建 UI 的具体内容
                    egui_ctx.run(input, move |ctx| {
                        egui::CentralPanel::default()
                            .frame(Frame::new().fill(Color32::from_hex("#393b40").unwrap())
                                .shadow(Shadow {
                                offset: [2, 3],
                                blur: 10,
                                spread: 0,
                                color: Color32::from_rgba_premultiplied(0, 0, 0, 80),
                            }))
                            .show(ctx, |ui| {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                                ui.spacing_mut().item_spacing = vec2(1.0, 0.0);

                                let current_view_id = window_ctx.current_view_id.clone().unwrap();
                                let annotator_state = window_ctx.globals_by_type
                                    .get_mut(&TypeId::of::<AnnotatorState>())
                                    .map(|any_state| any_state.downcast_mut::<AnnotatorState>().unwrap())
                                    .with_context(|| format!("no state of type {} exists", type_name::<AnnotatorState>()))
                                    .unwrap();
                                let active_tool = annotator_state.current_annotation_tool;

                                ui.horizontal(|ui| {
                                    if ui
                                        .add(SvgButton::new(
                                            "rectangle-tool".into(),
                                            Icons::DrawRectangle.get_image(),
                                            LogicalSize::new(32., 32.),
                                            true,
                                            matches!(active_tool, Some(ToolType::Rectangle)),
                                        ))
                                        .clicked()
                                    {
                                        if matches!(active_tool, Some(ToolType::Rectangle)) {
                                            annotator_state.current_annotation_tool = None;
                                        } else {
                                            annotator_state.current_annotation_tool =
                                                Some(ToolType::Rectangle);
                                        }
                                    }
                                    if ui
                                        .add(SvgButton::new(
                                            "ellipse-tool".into(),
                                            Icons::DrawEllipse.get_image(),
                                            LogicalSize::new(32., 32.),
                                            true,
                                            matches!(active_tool, Some(ToolType::Ellipse)),
                                        ))
                                        .clicked()
                                    {
                                        if matches!(active_tool, Some(ToolType::Ellipse)) {
                                            annotator_state.current_annotation_tool = None;
                                        } else {
                                            annotator_state.current_annotation_tool =
                                                Some(ToolType::Ellipse);
                                        }
                                    }

                                    if ui
                                        .add(SvgButton::new(
                                            "straight-line-tool".into(),
                                            Icons::DrawLine.get_image(),
                                            LogicalSize::new(32., 32.),
                                            true,
                                            matches!(active_tool, Some(ToolType::StraightLine)),
                                        ))
                                        .clicked()
                                    {
                                        if matches!(active_tool, Some(ToolType::StraightLine)) {
                                            annotator_state.current_annotation_tool = None;
                                        } else {
                                            annotator_state.current_annotation_tool =
                                                Some(ToolType::StraightLine);
                                        }
                                    }
                                    if ui
                                        .add(SvgButton::new(
                                            "arrow-tool".into(),
                                            Icons::DrawArrow.get_image(),
                                            LogicalSize::new(32., 32.),
                                            true,
                                            matches!(active_tool, Some(ToolType::Arrow)),
                                        ))
                                        .clicked()
                                    {
                                        if matches!(active_tool, Some(ToolType::Arrow)) {
                                            annotator_state.current_annotation_tool = None;
                                        } else {
                                            annotator_state.current_annotation_tool =
                                                Some(ToolType::Arrow);
                                        }
                                    }

                                    if ui
                                        .add(SvgButton::new(
                                            "pencil-tool".into(),
                                            Icons::DrawFreehand.get_image(),
                                            LogicalSize::new(32., 32.),
                                            true,
                                            matches!(active_tool, Some(ToolType::Pencil)),
                                        ))
                                        .clicked()
                                    {
                                        if matches!(active_tool, Some(ToolType::Pencil)) {
                                            annotator_state.current_annotation_tool = None;
                                        } else {
                                            annotator_state.current_annotation_tool =
                                                Some(ToolType::Pencil);
                                        }
                                    }

                                    if ui
                                        .add(SvgButton::new(
                                            "marker-pen-tool".into(),
                                            Icons::DrawHighlight.get_image(),
                                            LogicalSize::new(32., 32.),
                                            true,
                                            matches!(active_tool, Some(ToolType::MarkerPen)),
                                        ))
                                        .clicked()
                                    {
                                        if matches!(active_tool, Some(ToolType::MarkerPen)) {
                                            annotator_state.current_annotation_tool = None;
                                        } else {
                                            annotator_state.current_annotation_tool =
                                                Some(ToolType::MarkerPen);
                                        }
                                    }

                                    if ui
                                        .add(SvgButton::new(
                                            "mosaic-tool".into(),
                                            Icons::PixelArtTrace.get_image(),
                                            LogicalSize::new(32., 32.),
                                            true,
                                            matches!(active_tool, Some(ToolType::Mosaic)),
                                        ))
                                        .clicked()
                                    {
                                        if matches!(active_tool, Some(ToolType::Mosaic)) {
                                            annotator_state.current_annotation_tool = None;
                                        } else {
                                            annotator_state.current_annotation_tool =
                                                Some(ToolType::Mosaic);
                                        }
                                    }

                                    if ui
                                        .add(SvgButton::new(
                                            "blur-tool".into(),
                                            Icons::BlurFx.get_image(),
                                            LogicalSize::new(32., 32.),
                                            true,
                                            matches!(active_tool, Some(ToolType::Blur)),
                                        ))
                                        .clicked()
                                    {
                                        if matches!(active_tool, Some(ToolType::Blur)) {
                                            annotator_state.current_annotation_tool = None;
                                        } else {
                                            annotator_state.current_annotation_tool =
                                                Some(ToolType::Blur);
                                        }
                                    }

                                    if ui
                                        .add(SvgButton::new(
                                            "text-tool".into(),
                                            Icons::DrawText.get_image(),
                                            LogicalSize::new(32., 32.),
                                            true,
                                            matches!(active_tool, Some(ToolType::Text)),
                                        ))
                                        .clicked()
                                    {
                                        if matches!(active_tool, Some(ToolType::Text)) {
                                            annotator_state.current_annotation_tool = None;
                                        } else {
                                            annotator_state.current_annotation_tool =
                                                Some(ToolType::Text);
                                        }
                                    }

                                    if ui
                                        .add(SvgButton::new(
                                            "serial-number-tool".into(),
                                            Icons::DrawNumber.get_image(),
                                            LogicalSize::new(32., 32.),
                                            true,
                                            matches!(active_tool, Some(ToolType::SerialNumber)),
                                        ))
                                        .clicked()
                                    {
                                        if matches!(active_tool, Some(ToolType::SerialNumber)) {
                                            annotator_state.current_annotation_tool = None;
                                        } else {
                                            annotator_state.current_annotation_tool =
                                                Some(ToolType::SerialNumber);
                                        }
                                    }

                                    if ui
                                        .add(SvgButton::new(
                                            "eraser-tool".into(),
                                            Icons::DrawEraser.get_image(),
                                            LogicalSize::new(32., 32.),
                                            false,
                                            matches!(active_tool, Some(ToolType::Eraser)),
                                        ))
                                        .clicked()
                                    {
                                        if matches!(active_tool, Some(ToolType::Eraser)) {
                                            annotator_state.current_annotation_tool = None;
                                        } else {
                                            annotator_state.current_annotation_tool =
                                                Some(ToolType::Eraser);
                                        }
                                    }

                                    if ui
                                        .add(SvgButton::new(
                                            "undo-tool".into(),
                                            Icons::EditUndo.get_image(),
                                            LogicalSize::new(32., 32.),
                                            false,
                                            false,
                                        ))
                                        .clicked()
                                    {}

                                    if ui
                                        .add(SvgButton::new(
                                            "redo-tool".into(),
                                            Icons::EditRedo.get_image(),
                                            LogicalSize::new(32., 32.),
                                            false,
                                            false,
                                        ))
                                        .clicked()
                                    {}

                                    if ui
                                        .add(SvgButton::new(
                                            "save-tool".into(),
                                            Icons::DocumentSave.get_image(),
                                            LogicalSize::new(32., 32.),
                                            false,
                                            false,
                                        ))
                                        .clicked()
                                    {}

                                    if ui
                                        .add(SvgButton::new(
                                            "copy-tool".into(),
                                            Icons::EditCopy.get_image(),
                                            LogicalSize::new(32., 32.),
                                            false,
                                            false,
                                        ))
                                        .clicked()
                                    {}
                                    if ui
                                        .add(SvgButton::new(
                                            "ok-tool".into(),
                                            Icons::DialogOk.get_image(),
                                            LogicalSize::new(32., 32.),
                                            false,
                                            false,
                                        ))
                                        .clicked()
                                    {
                                        annotator_state.current_annotation_tool = None;
                                        window_ctx.commands.push_back(Command::HideView(current_view_id));
                                    }

                                    if active_tool != annotator_state.current_annotation_tool {
                                        println!("标注工具由{:?}切换为{:?}", active_tool, annotator_state.current_annotation_tool);

                                        if let Some(tool) = active_tool {
                                            match tool {
                                                ToolType::Rectangle => {
                                                    let rectangle_state = annotator_state.annotations_stack
                                                        .last_mut()
                                                        .map(|annotation|annotation.downcast_mut::<RectangleState>())
                                                        .flatten();
                                                    if let Some(rectangle_state) = rectangle_state {
                                                        rectangle_state.deactivate();
                                                    }
                                                }
                                                ToolType::Ellipse => {
                                                    let ellipse_state = annotator_state.annotations_stack
                                                        .last_mut()
                                                        .map(|annotation|annotation.downcast_mut::<EllipseState>())
                                                        .flatten();
                                                    if let Some(ellipse_state) = ellipse_state {
                                                        ellipse_state.deactivate();
                                                    }
                                                }
                                                ToolType::StraightLine => {}
                                                ToolType::Arrow => {}
                                                ToolType::Pencil => {}
                                                ToolType::MarkerPen => {}
                                                ToolType::Mosaic => {}
                                                ToolType::Blur => {}
                                                ToolType::Text => {}
                                                ToolType::SerialNumber => {}
                                                ToolType::Watermark => {}
                                                ToolType::Eraser => {}
                                            }
                                        }

                                    }
                                });
                            });
                    })
                }),
                Some(position_calculator),
            );
        });

        app.with_window_mut(window_id, |global_state, window| {
            let window = window.as_mut().unwrap();

            let position_calculator = Arc::new(
                |parent_surface_size: &PhysicalSize<u32>, subview_size: &PhysicalSize<u32>| {
                    let subview_width = &subview_size.width;
                    PhysicalPosition::new(
                        parent_surface_size.width - subview_width,
                        parent_surface_size.height + 54,
                    )
                },
            );

            // 创建工具条
            window.create_sub_surface_view(
                "secondly-toolbar".into(),
                global_state,
                LogicalSize::new(600, 32),
                LogicalPosition::new(0i32, 0i32),
                Box::new(|input, egui_ctx, window_ctx| {
                    // 构建 UI 的具体内容
                    egui_ctx.run(input, move |ctx| {
                        egui::CentralPanel::default()
                            .frame(Frame::new().fill(Color32::from_hex("#393b40").unwrap())
                                .shadow(Shadow {
                                    offset: [2, 3],
                                    blur: 10,
                                    spread: 0,
                                    color: Color32::from_rgba_premultiplied(0, 0, 0, 80),
                                }))
                            .show(ctx, |ui| {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::Default);
                                ui.spacing_mut().item_spacing = vec2(1.0, 0.0);

                                let current_view_id = window_ctx.current_view_id.clone().unwrap();
                                let annotator_state = window_ctx.globals_by_type
                                    .get_mut(&TypeId::of::<AnnotatorState>())
                                    .map(|any_state| any_state.downcast_mut::<AnnotatorState>().unwrap())
                                    .with_context(|| format!("no state of type {} exists", type_name::<AnnotatorState>()))
                                    .unwrap();
                                let active_tool = annotator_state.current_annotation_tool;

                                if active_tool.is_none() {
                                    window_ctx.commands.push_back(Command::HideView(current_view_id));
                                }
                            });
                    })
                }), Some(position_calculator));

        });
    }

    app.run();
}
