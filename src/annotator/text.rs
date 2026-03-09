use egui::{Painter, Response, Ui, Widget};
use crate::annotator::Paint;

#[derive(Clone)]
pub struct TextState{

}

impl Paint for TextState {
    fn paint_with(&mut self, painter: &Painter) {
        todo!()
    }
}