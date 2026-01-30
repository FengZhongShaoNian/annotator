use crate::global::Global;
use egui::{widgets, TextureHandle};


#[derive(Default)]
pub struct AnnotatorState {
    /// 背景图片的纹理句柄
    pub background_texture_handle: Option<TextureHandle>,

    pub editing_text: String,
}

impl Global for AnnotatorState {}