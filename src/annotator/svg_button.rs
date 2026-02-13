use egui::{pos2, vec2, Color32, Image, Rect, Response, Sense, Stroke, StrokeKind, Ui, Widget};
use egui_extras::Size;
use crate::dpi::LogicalSize;

pub struct SvgButton {
    /// svg图片
    image: Image<'static>,
    /// 图片显示的大小
    size: LogicalSize<f32>,
    /// 是否可勾选
    checkable: bool,
    /// 是否已勾选
    checked: bool,
}

impl SvgButton {
    pub fn new(image: Image<'static>, size: LogicalSize<f32>, checkable: bool, checked: bool) -> Self {
        Self {
            image,
            size,
            checkable,
            checked,
        }
    }
}

impl Widget for SvgButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = vec2(self.size.width, self.size.height);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let bg_color_checked = Color32::from_hex("#535bf2").unwrap();
        let bg_color_active = Color32::from_hex("#535bf2").unwrap();
        let bg_color_hover = Color32::from_hex("#2980b9").unwrap();
        if self.checkable && self.checked {
            ui.painter().rect(rect, 0., bg_color_checked, Stroke::new(1., Color32::TRANSPARENT), StrokeKind::Middle);
        }
        if response.hovered() {
            ui.painter().rect(rect, 0., bg_color_hover, Stroke::new(1., Color32::TRANSPARENT), StrokeKind::Middle);
        }
        if response.clicked() {
            ui.painter().rect(rect, 0., bg_color_active, Stroke::new(1., Color32::TRANSPARENT), StrokeKind::Middle);
        }
        self.image.paint_at(ui, rect.scale_from_center(0.75));
        response
    }
}