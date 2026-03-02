use crate::dpi::LogicalSize;
use crate::surface_view::SurfaceView;
use crate::view::{BuildViewFn, PopupView, View, ViewId};
use egui_wgpu::wgpu::Surface;
use sctk::shell::xdg::popup::Popup;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols::wp::viewporter::client::wp_viewport::WpViewport;
use wayland_protocols::xdg::shell::client::xdg_positioner::XdgPositioner;

pub struct XdgPopupView<'window> {
    view: SurfaceView<'window>,
    popup: Popup,
    positioner: XdgPositioner,
    /// 是否已完成首次配置（用于避免在窗口未准备好时绘图）
    pub first_configured: bool,
}

impl<'window> XdgPopupView<'window> {
    pub fn new(
        id: ViewId,
        surface: WlSurface,
        popup: Popup,
        positioner: XdgPositioner,
        wgpu_surface: Surface<'window>,
        wgpu_surface_configuration: egui_wgpu::wgpu::SurfaceConfiguration,
        size: LogicalSize<u32>,
        viewport: WpViewport,
        build_view: BuildViewFn,
    ) -> Self {
        let view = SurfaceView::new(
            id,
            surface,
            wgpu_surface,
            wgpu_surface_configuration,
            size,
            viewport,
            build_view,
        );
        Self {
            view,
            positioner,
            popup,
            first_configured: false,
        }
    }
}

impl PopupView for XdgPopupView<'_> {
    fn view(&self) -> &dyn View {
        &self.view
    }

    fn view_mut(&mut self) -> &mut dyn View {
        &mut self.view
    }

    fn first_configured(&self) -> bool {
        self.first_configured
    }

    fn set_first_configured(&mut self) {
        self.first_configured = true;
    }


    fn popup(&self) -> &Popup {
        &self.popup
    }
}