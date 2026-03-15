use image::RgbaImage;
use smithay_clipboard::mime::{AllowedMimeTypes, AsMimeTypes, MimeType};
use std::borrow::Cow;
use std::sync::Arc;

pub struct Image {
    /// png图片
    data: Vec<u8>,
}

impl Image {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}
fn to_png_bytes(image: &RgbaImage) -> Result<Vec<u8>, String> {
    let mut png_bytes: Vec<u8> = Vec::new();
    image
        .write_to(
            &mut std::io::Cursor::new(&mut png_bytes),
            image::ImageFormat::Png,
        )
        .map_err(|err| err.to_string())?;
    Ok(png_bytes)
}

impl From<RgbaImage> for Image {
    fn from(image: RgbaImage) -> Self {
        Self::new(to_png_bytes(&image).unwrap())
    }
}

impl From<Arc<RgbaImage>> for Image {
    fn from(image: Arc<RgbaImage>) -> Self {
        Self::new(to_png_bytes(&*image).unwrap())
    }
}

impl TryFrom<(Vec<u8>, MimeType)> for Image {
    type Error = ();

    fn try_from(value: (Vec<u8>, MimeType)) -> Result<Self, Self::Error> {
        Ok(Self { data: value.0 })
    }
}

impl AllowedMimeTypes for Image {
    fn allowed() -> Cow<'static, [MimeType]> {
        Cow::Borrowed(&[
            MimeType::Other(Cow::Borrowed("image/png")),
        ])
    }
}

impl AsMimeTypes for Image {
    fn available(&self) -> Cow<'static, [MimeType]> {
        Self::allowed()
    }

    fn as_bytes(&self, mime_type: &MimeType) -> Option<Cow<'static, [u8]>> {
        match mime_type {
            MimeType::Other(Cow::Borrowed("image/png")) => Some(Cow::Owned(self.data.to_owned())),
            _ => None,
        }
    }
}