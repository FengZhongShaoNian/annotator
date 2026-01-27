use std::sync::Arc;
use egui::{ColorImage, TextureHandle};
use crate::dpi::LogicalSize;

#[derive(Default)]
pub struct AnnotatorContext {
    pub background_image: Option<Arc<ColorImage>>,
    pub background_texture: Option<TextureHandle>,
}

impl AnnotatorContext {

}

pub struct Output {
    pub egui_output: egui::FullOutput,
    pub preferred_size: Option<LogicalSize<u32>>,
}

impl Output {
    pub fn new(egui_output: egui::FullOutput) -> Self {
        Self {
            egui_output,
            preferred_size: None,
        }
    }
    
    pub fn set_preferred_size(&mut self, size: Option<LogicalSize<u32>>) {
        self.preferred_size = size;
    }
}