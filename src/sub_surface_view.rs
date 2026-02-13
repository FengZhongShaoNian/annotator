use crate::context::WindowContext;
use crate::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use crate::surface_view::SurfaceView;
use crate::view::{BuildViewFn, SubView, View};
use egui::FullOutput;
use egui_wgpu::wgpu::Surface;
use std::sync::Arc;
use wayland_client::protocol::wl_subsurface::WlSubsurface;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols::wp::viewporter::client::wp_viewport::WpViewport;

pub struct SubSurfaceView<'window> {
    view: SurfaceView<'window>,
    subsurface: WlSubsurface,
    position_calculator: Option<Arc<crate::view::RelativePositionCalculator>>,
}

impl<'window> SubSurfaceView<'window> {
    pub fn new(
        surface: WlSurface,
        wgpu_surface: Surface<'window>,
        wgpu_surface_configuration: egui_wgpu::wgpu::SurfaceConfiguration,
        subsurface: WlSubsurface,
        size: LogicalSize<u32>,
        viewport: WpViewport,
        build_view: BuildViewFn,
        position_calculator: Option<Arc<crate::view::RelativePositionCalculator>>,
    ) -> Self {
        let view = SurfaceView::new(
            surface,
            wgpu_surface,
            wgpu_surface_configuration,
            size,
            viewport,
            build_view,
        );
        Self {
            view,
            subsurface,
            position_calculator,
        }
    }
}

impl<'window> SubView for SubSurfaceView<'window> {
    fn view(&self) -> &dyn View {
        &self.view
    }

    fn view_mut(&mut self) -> &mut dyn View {
        &mut self.view
    }

    fn set_position(&mut self, pos: LogicalPosition<i32>) {
        self.subsurface.set_position(pos.x, pos.y);
    }

    fn position_calculator(&mut self) -> Option<Arc<crate::view::RelativePositionCalculator>> {
        if let Some(relocation_fn) = &self.position_calculator {
            Some(relocation_fn.clone())
        } else {
            None
        }
    }
}
