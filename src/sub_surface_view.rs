use wayland_client::protocol::wl_subsurface::WlSubsurface;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols::wp::viewporter::client::wp_viewport::WpViewport;
use crate::context::{AnnotatorContext, Output};
use crate::dpi::{LogicalPosition, LogicalSize};
use crate::surface_view::SurfaceView;
use crate::view::{SubView, View};

pub struct SubSurfaceView {
    view: SurfaceView,
    subsurface: WlSubsurface
}

impl SubSurfaceView {
    pub fn new(surface: WlSurface,
               subsurface: WlSubsurface, 
               size: LogicalSize<u32>,
               viewport: WpViewport,
               build_view: Box<dyn Fn(egui::RawInput, &mut egui::Context, &mut AnnotatorContext) -> Output>) -> Self {
        let view = SurfaceView::new(surface, size, viewport, build_view);
        Self {
            view,
            subsurface
        }
    }
}

impl SubView for SubSurfaceView {
    fn view(&self) -> &dyn View {
        &self.view
    }
    
    fn view_mut(&mut self) -> &mut dyn View {
        &mut self.view
    }

    fn set_position(&mut self, pos: LogicalPosition<i32>) {
        self.subsurface.set_position(pos.x, pos.y);
    }
}
