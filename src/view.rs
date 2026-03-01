use crate::application::{Application, GlobalState};
use crate::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use crate::gpu::GpuContext;
use crate::window::AppWindow;
use egui::{FullOutput, ImeEvent, PlatformOutput, RawInput};
use sctk::shell::xdg::popup::Popup;
use std::sync::Arc;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols::wp::viewporter::client::wp_viewport::WpViewport;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct ViewId(pub Arc<str>);

impl From<String> for ViewId {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

impl From<&str> for ViewId {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

pub trait View {
    fn id(&self) -> ViewId;
    fn scale_factor(&self) -> f64;
    fn set_scale_factor(&mut self, scale_factor: f64, gpu: &GpuContext);
    fn size(&self) -> LogicalSize<u32>;
    fn viewport_size(&self) -> PhysicalSize<u32>;
    fn viewport(&self) -> &WpViewport;
    fn resize(&mut self, new_size: LogicalSize<u32>, gpu: &GpuContext);
    fn surface(&self) -> &WlSurface;

    fn handle_keyboard_event(
        &mut self,
        event: sctk::seat::keyboard::KeyEvent,
        pressed: bool,
        repeat: bool,
    );
    fn update_modifiers(&mut self, modifiers: sctk::seat::keyboard::Modifiers);
    fn handle_ime_event(&mut self, event: &ImeEvent);
    fn handle_pointer_event(
        &mut self,
        event: &sctk::seat::pointer::PointerEvent,
        global_state: &GlobalState,
    );
    fn visible(&self) -> bool;

    fn set_visible(&mut self, visible: bool);

    fn should_remove(&self) -> bool;

    /// 使用 GPU 上下文进行重绘。
    fn draw(&mut self, app: &mut Application, window: &mut AppWindow) -> Option<PlatformOutput>;
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

pub trait PopupView {
    fn view(&self) -> &dyn View;

    fn view_mut(&mut self) -> &mut dyn View;

    fn is_first_configure(&self) -> bool;

    fn set_is_first_configure(&mut self, is_first_configure: bool);

    fn popup(&self) -> &Popup;
}

pub type BuildViewFn =
    Box<dyn Fn(RawInput, &mut egui::Context, &mut Application, &mut AppWindow, &mut dyn View) -> FullOutput>;

pub enum AppView {
    Root(Box<dyn View>),
    Child(Box<dyn SubView>),
    Pop(Box<dyn PopupView>),
}
