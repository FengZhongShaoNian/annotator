use std::cell::RefCell;
use std::rc::{Rc, Weak};
use crate::annotator::cursor::{CustomCursor, SerialNumber, SerialNumberStyle};
use egui::{Color32, CursorIcon, Pos2, Rect, Response, Sense, Ui, Widget};
use crate::{declare_not_support_stroke_color, declare_not_support_stroke_type, declare_not_support_stroke_width};
use crate::annotator::{StrokeWidthSupport, StrokeColorSupport, StrokeTypeSupport, FillColorSupport, StrokeType, AnnotatorState, Annotation, ActivationSupport, AnnotationActivationSupport, DeactivatedAware, WheelHandler};

#[derive(Clone)]
pub struct SerialNumberAnnotation {
    serial_number: SerialNumber,
    activation: ActivationSupport
}

impl SerialNumberAnnotation {
    pub fn new(center_pos: Pos2, number: u32, style: SerialNumberStyle) -> SerialNumberAnnotation {
        Self {
            serial_number: SerialNumber::new(center_pos, number, style),
            activation: ActivationSupport::NotSupported,
        }
    }
}

impl AnnotationActivationSupport for SerialNumberAnnotation {
    fn activation(&self) -> &ActivationSupport {
        &self.activation
    }

    fn activation_mut(&mut self) -> &mut ActivationSupport {
        &mut self.activation
    }
}

impl Widget for &mut SerialNumberAnnotation {
    fn ui(self, ui: &mut Ui) -> Response {
        let response = ui.allocate_rect(self.serial_number.rect(), Sense::hover());
        self.serial_number.paint_with(ui.painter());
        response
    }
}

declare_not_support_stroke_width!(SerialNumberAnnotation);
declare_not_support_stroke_color!(SerialNumberAnnotation);
declare_not_support_stroke_type!(SerialNumberAnnotation);

impl FillColorSupport for SerialNumberAnnotation {
    fn supports_get_fill_color(&self) -> bool {
        true
    }

    fn fill_color(&self) -> Option<Color32> {
        Some(self.serial_number.style().fill_color)
    }

    fn supports_set_fill_color(&self) -> bool {
        true
    }

    fn set_fill_color(&mut self, color: Color32) {
        self.serial_number.style_mut().fill_color = color;
    }
}

impl Into<Annotation> for SerialNumberAnnotation {
    fn into(self) -> Annotation {
        Annotation::SerialNumber(self)
    }
}


struct SerialNumberToolState {
    style: SerialNumberStyle,
    next_number: u32,
}
impl SerialNumberToolState {
    fn new(style: SerialNumberStyle) -> Self {
        Self {
            style,
            next_number: 1,
        }
    }
}

impl Default for SerialNumberToolState {
    fn default() -> Self {
        let mut style = SerialNumberStyle::default();
        style.draw_rect_stroke = false;
        Self::new(style)
    }
}

pub struct SerialNumberTool {
    annotator_state: Weak<RefCell<AnnotatorState>>,
    tool_state: SerialNumberToolState,
}

impl SerialNumberTool {
    pub fn new(annotator_state: Weak<RefCell<AnnotatorState>>) -> Self {
        Self {
            annotator_state,
            tool_state: Default::default(),
        }
    }

    fn update_cursor(&mut self, ui: &mut Ui) {
        let pointer_pos = ui.ctx().input(|i|i.pointer.hover_pos());
        let Some(pointer_pos) = pointer_pos else {
            return;
        };

        ui.ctx().set_cursor_icon(CursorIcon::None);
        let mut style = self.tool_state.style.clone();
        style.draw_rect_stroke = true;
        SerialNumber::new(pointer_pos, self.tool_state.next_number, style).paint_with(ui.painter());
    }
}

declare_not_support_stroke_width!(SerialNumberTool);
declare_not_support_stroke_color!(SerialNumberTool);
declare_not_support_stroke_type!(SerialNumberTool);

impl FillColorSupport for SerialNumberTool {
    fn supports_get_fill_color(&self) -> bool {
        true
    }
    fn fill_color(&self) -> Option<Color32> {
        Some(self.tool_state.style.fill_color)
    }
    fn supports_set_fill_color(&self) -> bool {
        true
    }
    fn set_fill_color(&mut self, color: Color32) {
        self.tool_state.style.fill_color = color;
    }
}

impl DeactivatedAware for SerialNumberTool {

}

impl Widget for &mut SerialNumberTool {
    fn ui(self, ui: &mut Ui) -> Response {
        let sense_area = Rect::from_min_size(Pos2::ZERO, ui.available_size());
        let response = ui.allocate_rect(sense_area, Sense::click());

        self.update_cursor(ui);
        self.handle_wheel_event(ui);

        if response.clicked() {
            let pointer_pos = ui.ctx().input(|i|i.pointer.hover_pos());
            let number = self.tool_state.next_number;
            let annotation = SerialNumberAnnotation::new(pointer_pos.unwrap(), number, self.tool_state.style.clone());
            self.tool_state.next_number += 1;

            let  annotator_state= self.annotator_state.upgrade().unwrap();
            annotator_state.borrow_mut().annotations_stack.push(annotation.into());
        }

        response
    }
}

impl WheelHandler for SerialNumberTool {
    fn on_scroll_delta_changed(&mut self, value: f32) {
        if value < 0. {
            if self.tool_state.next_number + 1 <= 99 {
                self.tool_state.next_number += 1;
            }
        }else {
            if self.tool_state.next_number - 1 > 0 {
                self.tool_state.next_number -= 1;
            }

        }

    }
}