use egui::{ColorImage, TextureHandle};
use std::sync::Arc;

#[derive(Default)]
pub struct SurfaceViewContext {
    pub background_image: Option<Arc<ColorImage>>,
    pub background_texture: Option<TextureHandle>,
}
