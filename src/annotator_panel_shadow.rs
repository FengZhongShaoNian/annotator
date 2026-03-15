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
    AnnotationTool, AnnotatorState, ApplyExtraZoomFactor, ExtraZoomFactorSupport,
    SharedAnnotatorState, SharedAnnotatorStateUtil, ToolName, WheelHandler,
};
use crate::application::Application;
use crate::context::Command;
use crate::context_menu::create_context_menu;
use crate::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use crate::egui_off_screen_render::EguiOffScreenRender;
use crate::global::{ReadGlobalMut, ReadOrInsertGlobal};
use crate::view::ViewId;
use crate::window::AppWindow;
use egui::load::SizedTexture;
use egui::{
    Area, Color32, ColorImage, Frame, Image, ImageSource, PointerButton, Rect, Shadow, pos2, vec2,
};
use image::RgbaImage;
use log::info;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub fn create_annotator_panel_shadow(
    view_id: ViewId,
    app: &mut Application,
    window: &mut AppWindow,
    logical_size: LogicalSize<u32>,
    logical_position: LogicalPosition<i32>,
) {
    let global_state = &app.global_state;
    window.create_sub_surface_view(
        view_id,
        global_state,
        logical_size,
        logical_position,
        Box::new(move |input, egui_ctx, app, window, current_view| {
            // 构建 UI 的具体内容
            egui_ctx.run(input, move |ctx| {
                egui::CentralPanel::default()
                    .frame(Frame::new().shadow(Shadow {
                        offset: [1, 2],
                        blur: 255, // 模糊半径（值越大越模糊）
                        spread: 2,  // 扩散半径
                        color: Color32::from_black_alpha(150), // 半透明黑色
                    }))
                    .show(ctx, |_ui| {});
            })
        }),
        None,
    );
}
