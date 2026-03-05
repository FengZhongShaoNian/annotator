use egui::{Response, Ui, Widget};
use crate::application::Application;
use crate::window::AppWindow;


struct DropDownBox<'a, 'w> {
    app: &'a mut Application,
    window: &'w mut AppWindow,
}

impl Widget for DropDownBox<'_, '_> {
    fn ui(self, ui: &mut Ui) -> Response {
        todo!()
    }
}