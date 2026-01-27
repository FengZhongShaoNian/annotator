use log::info;
use crate::application::Application;
use crate::dpi::{LogicalPosition, LogicalSize};
use wayland_client::QueueHandle;
use wayland_client::protocol::wl_surface;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols::wp::viewporter::client::wp_viewport::WpViewport;
use crate::window::AppWindow;

pub trait View {
    fn scale_factor(&self) -> f64;
    fn set_scale_factor(&mut self, scale_factor: f64);
    fn size(&self) -> LogicalSize<u32>;
    fn viewport(&self) -> &WpViewport;
    fn resize(&mut self, new_size: LogicalSize<i32>) {
        info!("Resize viewport to: {:?}", new_size);
        self.viewport().set_destination(new_size.width.into(), new_size.height.into());
    }

    fn surface(&self) -> &WlSurface;

    fn handle_keyboard_event(&mut self, event: sctk::seat::keyboard::KeyEvent, pressed: bool);
    fn update_modifiers(&mut self, modifiers: sctk::seat::keyboard::Modifiers);
    fn handle_pointer_event(&mut self, event: &sctk::seat::pointer::PointerEvent, globals: &crate::application::Globals);

    /// 使用 GPU 上下文进行重绘。
    fn draw(&mut self, queue_handle: &QueueHandle<Application>, gpu: &mut crate::gpu::GpuContext);
}

pub trait SubView {
    fn view(&self) -> &dyn View;
    
    fn view_mut(&mut self) -> &mut dyn View;
    
    fn set_position(&mut self, pos: LogicalPosition<i32>);
}
