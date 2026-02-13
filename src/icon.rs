pub enum Icons {
    DrawRectangle,
    DrawEllipse,
    DrawLine,
    DrawArrow,
    DrawFreehand,
    DrawHighlight,
    PixelArtTrace,
    BlurFx,
    DrawText,
    DrawNumber,
    DrawEraser,
    EditUndo,
    EditRedo,
    DocumentSave,
    EditCopy,
    DialogOk,
    Sticky,
}

impl Icons {
    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            Icons::DrawRectangle => {
                include_bytes!("../assets/icons/draw-rectangle.svg")
            }
            Icons::DrawEllipse => {
                include_bytes!("../assets/icons/draw-ellipse.svg")
            }
            Icons::DrawLine => {
                include_bytes!("../assets/icons/draw-line.svg")
            }
            Icons::DrawArrow => {
                include_bytes!("../assets/icons/draw-arrow.svg")
            }
            Icons::DrawFreehand => {
                include_bytes!("../assets/icons/draw-freehand.svg")
            }
            Icons::DrawHighlight => {
                include_bytes!("../assets/icons/draw-highlight.svg")
            }
            Icons::PixelArtTrace => {
                include_bytes!("../assets/icons/pixelart-trace.svg")
            }
            Icons::BlurFx => {
                include_bytes!("../assets/icons/blurfx.svg")
            }
            Icons::DrawText => {
                include_bytes!("../assets/icons/draw-text.svg")
            }
            Icons::DrawNumber => {
                include_bytes!("../assets/icons/draw-number.svg")
            }
            Icons::DrawEraser => {
                include_bytes!("../assets/icons/draw-eraser.svg")
            }
            Icons::EditUndo => {
                include_bytes!("../assets/icons/edit-undo.svg")
            }
            Icons::EditRedo => {
                include_bytes!("../assets/icons/edit-redo.svg")
            }
            Icons::DocumentSave => {
                include_bytes!("../assets/icons/document-save.svg")
            }
            Icons::EditCopy => {
                include_bytes!("../assets/icons/edit-copy.svg")
            }
            Icons::DialogOk => {
                include_bytes!("../assets/icons/dialog-ok.svg")
            }
            Icons::Sticky =>{
                include_bytes!("../assets/icons/sticky.svg")
            }
        }
    }
}
