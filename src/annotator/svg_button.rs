use crate::dpi::LogicalSize;
use egui::{
    Color32, Id, Image, Rect, Response, Rgba, Sense, Stroke, StrokeKind, Ui, Widget, pos2, vec2,
};
use egui_extras::Size;
use std::time::{Duration, Instant};

pub struct SvgButton {
    /// 一个id，用于跨帧追踪一些状态
    id: Id,
    /// svg图片
    image: Image<'static>,
    /// 图片显示的大小
    size: LogicalSize<f32>,
    /// 是否已禁用
    disabled: bool,
    /// 是否可勾选
    checkable: bool,
    /// 是否已勾选
    checked: bool,
}

impl SvgButton {
    pub fn new(
        id: Id,
        image: Image<'static>,
        size: LogicalSize<f32>,
        disabled: bool,
        checkable: bool,
        checked: bool,
    ) -> Self {
        Self {
            id,
            image,
            size,
            disabled,
            checkable,
            checked,
        }
    }
}

impl Widget for SvgButton {
    fn ui(self, ui: &mut Ui) -> Response {
        let size = vec2(self.size.width, self.size.height);
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let bg_color_hover = Color32::from_hex("#2980b9").unwrap();
        let bg_color_active = Color32::from_hex("#2980b9").unwrap();
        let bg_color_checked = Color32::from_hex("#535bf2").unwrap();

        if !self.disabled {
            if response.clicked() {
                ui.ctx().memory_mut(|memory| {
                    memory.data.insert_temp(self.id, Instant::now());
                });
            }

            let instant_clicked = ui
                .ctx()
                .memory(|memory| memory.data.get_temp::<Instant>(self.id));

            if let Some(instant_clicked) = instant_clicked {
                let duration = Instant::now().checked_duration_since(instant_clicked);
                if let Some(duration) = duration {
                    if duration < Duration::from_millis(500) {
                        ui.painter().rect(
                            rect.scale_from_center(0.9),
                            0.,
                            bg_color_active,
                            Stroke::new(1., Color32::TRANSPARENT),
                            StrokeKind::Middle,
                        );
                    } else {
                        ui.ctx().memory_mut(|memory| {
                            memory.data.remove_by_type::<Instant>();
                        });
                    }
                }
            } else if response.hovered() {
                ui.painter().rect(
                    rect,
                    0.,
                    bg_color_hover,
                    Stroke::new(1., Color32::TRANSPARENT),
                    StrokeKind::Middle,
                );
            }

            if self.checkable && self.checked {
                ui.painter().rect(
                    rect,
                    0.,
                    bg_color_checked,
                    Stroke::new(1., Color32::TRANSPARENT),
                    StrokeKind::Middle,
                );
            }
        }
        
        self.image.paint_at(ui, rect.scale_from_center(0.75));
        response
    }
}
