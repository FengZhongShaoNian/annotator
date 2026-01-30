use crate::global::Global;
use egui::TextureHandle;


/// 背景图片的纹理
pub struct BackgroundTexture(pub TextureHandle);

impl Global for BackgroundTexture {}
