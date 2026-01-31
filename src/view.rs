use crate::application::Application;
use crate::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use crate::gpu::GpuContext;
use crate::surface_view::SurfaceView;
use crate::window::AppWindow;
use log::info;
use std::sync::Arc;
use egui::ImeEvent;
use wayland_client::QueueHandle;
use wayland_client::protocol::wl_surface;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols::wp::viewporter::client::wp_viewport::WpViewport;
use crate::context::WindowContext;

pub trait View {
    fn scale_factor(&self) -> f64;
    fn set_scale_factor(&mut self, scale_factor: f64, gpu: &mut GpuContext);
    fn size(&self) -> LogicalSize<u32>;
    fn viewport_size(&self) -> PhysicalSize<u32>;
    fn viewport(&self) -> &WpViewport;
    fn resize(&mut self, new_size: LogicalSize<u32>, gpu: &mut GpuContext);
    fn surface(&self) -> &WlSurface;

    fn handle_keyboard_event(&mut self, event: sctk::seat::keyboard::KeyEvent, pressed: bool);
    fn update_modifiers(&mut self, modifiers: sctk::seat::keyboard::Modifiers);
    fn handle_ime_event(&mut self, event: &ImeEvent);
    fn handle_pointer_event(
        &mut self,
        event: &sctk::seat::pointer::PointerEvent,
        global_state: &crate::application::GlobalState,
    );

    /// 使用 GPU 上下文进行重绘。
    fn draw(&mut self, queue_handle: &QueueHandle<Application>, gpu: &GpuContext, window_context: &mut WindowContext);
}

/// 一个函数，可以根据父表面的尺寸和子表面自身的尺寸重新计算子表面的位置
pub(crate) type RelativePositionCalculator =
    dyn Fn(&PhysicalSize<u32>, &PhysicalSize<u32>) -> PhysicalPosition<u32>;

pub trait SubView {
    fn view(&self) -> &dyn View;

    fn view_mut(&mut self) -> &mut dyn View;

    fn set_position(&mut self, pos: LogicalPosition<i32>);

    fn position_calculator(&mut self) -> Option<Arc<RelativePositionCalculator>>;
}
