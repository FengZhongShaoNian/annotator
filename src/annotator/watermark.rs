use egui::{Painter, Response, Ui, Widget};
use crate::annotator::Paint;

#[derive(Clone)]
pub struct WaterMarkState{

}

impl Paint for WaterMarkState {
    fn paint_with(&mut self, painter: &Painter) {
        todo!()
    }
}