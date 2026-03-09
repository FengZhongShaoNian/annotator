use egui::{Painter, Response, Ui, Widget};
use crate::annotator::Paint;

#[derive(Clone)]
pub struct BlurState{

}

impl Paint for BlurState {
    fn paint_with(&mut self, painter: &Painter) {
        todo!()
    }
}