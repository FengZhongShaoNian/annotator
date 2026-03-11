use egui::{Painter, Response, Ui, Widget};

#[derive(Clone)]
pub struct SerialNumberState{

}

impl Widget for &mut SerialNumberState {
    fn ui(self, ui: &mut Ui) -> Response {
        todo!()
    }
}