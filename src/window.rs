use crate::application::{Application, GlobalState};
use crate::context::WindowContext;
use crate::dpi::{LogicalPosition, LogicalSize, PhysicalBounds, PhysicalSize};
use crate::gpu;
use crate::gpu::GpuContext;
use crate::sub_surface_view::SubSurfaceView;
use crate::surface_view::SurfaceView;
use crate::view::{SubView, View};
use egui::{FullOutput, ImeEvent, PlatformOutput};
use egui_wgpu::wgpu;
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use rustc_hash::FxHashMap;
use sctk::shell::WaylandSurface;
use sctk::shell::xdg::window::{Window as XdgWindow, WindowDecorations};
use std::any::{Any, TypeId};
use std::ptr::NonNull;
use std::sync::Arc;
use wayland_client::protocol::wl_surface;
use wayland_client::{Proxy, QueueHandle};
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_v1::WpFractionalScaleV1;

/// AppWindow 管理应用的主窗口，包括主视图和动态管理的子视图。
pub struct AppWindow {
    /// 主视图（Surface）
    pub main_view: SurfaceView<'static>,
    /// 子视图列表（SubSurface）
    pub sub_views: Vec<Box<dyn SubView>>,
    /// 用于发送 Wayland 请求的消息队列句柄
    queue_handle: QueueHandle<Application>,
    /// XDG Shell 窗口句柄
    xdg_window: XdgWindow,
    /// 是否为首次配置（用于避免在窗口未准备好时绘图）
    pub first_configure: bool,
    /// 分数缩放管理（Wayland 协议扩展）
    fractional_scale: Option<WpFractionalScaleV1>,
    /// 当前缩放倍数
    scale_factor: Option<f64>,
    /// 窗口是否持有键盘焦点
    keyboard_focus: bool,
    /// 一个物理尺寸，用于在首次获取到缩放倍数后调整窗口的大小
    pub preferred_size: Option<PhysicalSize<u32>>,
    window_context: WindowContext,
}

pub(crate) struct WindowConfiguration {
    size: LogicalSize<u32>,
    preferred_size: Option<PhysicalSize<u32>>,
}

impl WindowConfiguration {
    pub fn new(size: LogicalSize<u32>, preferred_size: Option<PhysicalSize<u32>>) -> Self {
        WindowConfiguration {
            size,
            preferred_size,
        }
    }
}

impl AppWindow {
    /// 创建一个新的 AppWindow。
    /// 该方法会初始化 Wayland Surface、SubSurface、XDG Window，并为每个视图创建 GPU 渲染表面。
    ///
    /// # 参数
    /// - `app`: 应用实例
    /// - `build_root_view`: 构建根视图 UI 的回调函数，接收窗口实例和 egui Context，返回 FullOutput
    pub fn new(
        app: &mut Application,
        window_config: WindowConfiguration,
        build_root_view: Box<
            dyn Fn(egui::RawInput, &mut egui::Context, &mut WindowContext) -> FullOutput,
        >,
    ) -> AppWindow {
        // 创建主表面
        let main_surface = app
            .global_state
            .compositor_state
            .create_surface(&app.global_state.queue_handle);

        // 创建 XDG 窗口并设置属性
        let xdg_window = app.global_state.xdg_shell_state.create_window(
            main_surface.clone(),
            WindowDecorations::None,
            &app.global_state.queue_handle,
        );
        xdg_window.set_title("Image Annotator");
        xdg_window.set_app_id(app.app_id);
        xdg_window.commit();

        // Create the raw window handle for the surface.
        let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(app.global_state.connection.backend().display_ptr() as *mut _).unwrap(),
        ));
        let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(xdg_window.wl_surface().id().as_ptr() as *mut _).unwrap(),
        ));

        // 初始化 wgpu
        let wgpu_surface = if app.global_state.gpu.is_none() {
            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                ..Default::default()
            });

            let surface = unsafe {
                instance
                    .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle,
                        raw_window_handle,
                    })
                    .unwrap()
            };
            let gpu_context = gpu::GpuContext::new(instance, &surface).unwrap();
            app.global_state.gpu = Some(gpu_context);

            surface
        } else {
            let instance = &app.global_state.gpu.as_mut().unwrap().instance;
            let surface = unsafe {
                instance
                    .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                        raw_display_handle,
                        raw_window_handle,
                    })
                    .unwrap()
            };
            surface
        };

        let surface_caps =
            wgpu_surface.get_capabilities(&app.global_state.gpu.as_mut().unwrap().adapter);

        let surface_format = surface_caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,

            // width 和 height 指定 SurfaceTexture 的宽度和高度（物理像素，等于逻辑像素乘以屏幕缩放因子）
            // 现在还无法获取到缩放因子，暂时设置为和逻辑尺寸相同大小
            width: window_config.size.width,
            height: window_config.size.height,

            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        wgpu_surface.configure(&app.global_state.gpu.as_mut().unwrap().device, &config);

        // 初始化分数缩放和视口
        let fractional_scale = app
            .global_state
            .fractional_scaling_manager
            .as_ref()
            .map(|ref m| m.fractional_scaling(&main_surface, &app.global_state.queue_handle));
        let main_viewport = app
            .global_state
            .viewporter_state
            .as_ref()
            .map(|ref viewporter_state| {
                viewporter_state.get_viewport(&main_surface, &app.global_state.queue_handle)
            })
            .expect("Failed to retrieve viewport");

        // 初始化主视图 (SurfaceView)
        let main_size = LogicalSize::new(window_config.size.width, window_config.size.height);
        let main_view = SurfaceView::new(
            main_surface.clone(),
            wgpu_surface,
            config,
            main_size,
            main_viewport,
            build_root_view,
        );

        let qh = app.global_state.queue_handle.clone();

        let window = Self {
            main_view,
            sub_views: Vec::new(),
            queue_handle: qh,
            xdg_window,
            first_configure: true,
            fractional_scale,
            scale_factor: None,
            keyboard_focus: false,
            preferred_size: window_config.preferred_size,
            window_context: Default::default(),
        };

        window
    }

    /// 动态创建一个 SubSurfaceView 并添加到窗口中。
    ///
    /// # 参数
    /// - `app`: 应用实例
    /// - `size`: 子视图的逻辑大小
    /// - `position`: 子视图的位置
    /// - `build_view`: 构建子视图 UI 的回调函数，接收窗口实例和 egui Context，返回 FullOutput
    pub fn create_sub_surface_view(
        &mut self,
        app: &mut Application,
        size: LogicalSize<u32>,
        position: LogicalPosition<i32>,
        build_view: Box<
            dyn Fn(egui::RawInput, &mut egui::Context, &mut WindowContext) -> FullOutput,
        >,
        position_calculator: Option<Arc<crate::view::RelativePositionCalculator>>,
    ) -> &mut SubSurfaceView {
        let (sub_surface_handle, surface) =
            app.global_state.sub_compositor_state.create_subsurface(
                self.main_view.surface().clone(),
                &app.global_state.queue_handle,
            );

        let viewport = app
            .global_state
            .viewporter_state
            .as_ref()
            .map(|v| v.get_viewport(&surface, &app.global_state.queue_handle))
            .expect("Failed to retrieve viewport");

        // Create the raw window handle for the surface.
        let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(app.global_state.connection.backend().display_ptr() as *mut _).unwrap(),
        ));
        let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(surface.id().as_ptr() as *mut _).unwrap(),
        ));

        // 初始化 wgpu
        let instance = &mut app.global_state.gpu.as_mut().unwrap().instance;
        let wgpu_surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle,
                    raw_window_handle,
                })
                .unwrap()
        };

        let surface_caps =
            wgpu_surface.get_capabilities(&app.global_state.gpu.as_mut().unwrap().adapter);

        let surface_format = surface_caps.formats[0];
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,

            // width 和 height 指定 SurfaceTexture 的宽度和高度（物理像素，等于逻辑像素乘以屏幕缩放因子）
            // 现在还无法获取到缩放因子，暂时设置为和逻辑尺寸相同大小
            width: size.width,
            height: size.height,

            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        wgpu_surface.configure(&app.global_state.gpu.as_mut().unwrap().device, &config);

        let mut subview = SubSurfaceView::new(
            surface.clone(),
            wgpu_surface,
            config,
            sub_surface_handle,
            size,
            viewport,
            build_view,
            position_calculator,
        );
        subview.set_position(position);

        subview.view_mut().surface().commit(); // Initial commit

        self.sub_views.push(Box::new(subview));

        // 返回刚添加的视图的引用。由于使用 Box<dyn SubView>，我们需要进行 downcast。
        // 为了简单起见，这里假设我们知道它是 SubSurfaceView 类型。
        let last_idx = self.sub_views.len() - 1;
        let boxed_view = &mut self.sub_views[last_idx];
        // SAFETY: 我们刚刚存入的就是 SubSurfaceView
        unsafe {
            let ptr = boxed_view.as_mut() as *mut dyn SubView as *mut SubSurfaceView;
            &mut *ptr
        }
    }

    pub fn handle_pointer_event(
        &mut self,
        event: &sctk::seat::pointer::PointerEvent,
        globals: &crate::application::GlobalState,
    ) {
        let event_surface = &event.surface;
        if event_surface == self.main_view.surface() {
            self.main_view.handle_pointer_event(event, globals);
        } else {
            let sub_view = self
                .sub_views
                .iter_mut()
                .find(|sub_view| sub_view.view().surface() == event_surface);
            if let Some(sub_view) = sub_view {
                sub_view.view_mut().handle_pointer_event(event, globals);
            }
        }
    }

    pub fn handle_keyboard_event(&mut self, event: sctk::seat::keyboard::KeyEvent, pressed: bool, repeat: bool) {
        self.main_view.handle_keyboard_event(event.clone(), pressed, repeat);
        self.sub_views.iter_mut().for_each(|sub_view| {
            sub_view
                .view_mut()
                .handle_keyboard_event(event.clone(), pressed, repeat);
        });
    }

    pub fn update_modifiers(&mut self, modifiers: sctk::seat::keyboard::Modifiers) {
        self.main_view.update_modifiers(modifiers.clone());
        self.sub_views.iter_mut().for_each(|sub_view| {
            sub_view.view_mut().update_modifiers(modifiers.clone());
        });
    }

    pub fn handle_ime_event(&mut self, event: &ImeEvent) {
        self.main_view.handle_ime_event(event);
        self.sub_views.iter_mut().for_each(|sub_view| {
            sub_view.view_mut().handle_ime_event(event);
        });
    }

    pub fn contains_surface(&self, surface: &wl_surface::WlSurface) -> bool {
        if surface == self.main_view.surface() {
            return true;
        }
        for sub_view in &self.sub_views {
            if surface == sub_view.view().surface() {
                return true;
            }
        }
        false
    }

    pub fn xdg_window(&self) -> &XdgWindow {
        &self.xdg_window
    }

    pub fn scale_factor(&self) -> &Option<f64> {
        &self.scale_factor
    }

    pub fn set_scale_factor(&mut self, new_scale_factor: f64, gpu: &mut GpuContext) {
        self.scale_factor = Some(new_scale_factor);
        self.main_view.set_scale_factor(new_scale_factor, gpu);
        for sub_view in &mut self.sub_views {
            sub_view.view_mut().set_scale_factor(new_scale_factor, gpu);
            let position_calculate_fn = sub_view.position_calculator();
            if let Some(position_calculate_fn) = position_calculate_fn {
                let subview_size = sub_view.view().viewport_size();
                let new_position =
                    position_calculate_fn(&self.main_view.viewport_size(), &subview_size);
                sub_view.set_position(new_position.to_logical(new_scale_factor));
            }
        }
    }

    pub fn resize(&mut self, new_size: LogicalSize<u32>, gpu: &mut GpuContext) {
        self.main_view.resize(new_size, gpu);
        for sub_view in &mut self.sub_views {
            let position_calculate_fn = sub_view.position_calculator();
            if let Some(position_calculate_fn) = position_calculate_fn {
                let subview_size = sub_view.view().viewport_size();
                let new_position =
                    position_calculate_fn(&self.main_view.viewport_size(), &subview_size);
                sub_view.set_position(new_position.to_logical(self.scale_factor.unwrap()));
            }
        }
    }

    pub fn set_keyboard_focus(&mut self, focus: bool) {
        self.keyboard_focus = focus;
    }

    pub fn keyboard_focus(&self) -> bool {
        self.keyboard_focus
    }

    /// 执行窗口重绘逻辑。
    /// 遍历所有视图并调用其独立的渲染方法。
    pub fn draw(&mut self, global_state: &GlobalState) {
        if self.first_configure || self.scale_factor.is_none() {
            return;
        }

        let window_context = &mut self.window_context;

        // 1. 渲染主视图
        {
            let output = self.main_view.draw(global_state, window_context);
            Self::update_ime_position_if_necessary(&output, global_state);
        }

        // 2. 渲染子视图
        for i in 0..self.sub_views.len() {
            let output = self.sub_views[i].view_mut().draw(global_state, window_context);
            Self::update_ime_position_if_necessary(&output, global_state);
        }
    }

    fn update_ime_position_if_necessary(output: &Option<PlatformOutput>, global_state: &GlobalState) {
        if let (Some(platform_output), Some(text_input)) =
            (output, global_state.text_input.as_ref())
        {
            if let Some(ime) = platform_output.ime {
                let cursor_rect = ime.cursor_rect;
                text_input.set_cursor_rectangle(
                    cursor_rect.min.x.round() as i32,
                    cursor_rect.min.y.round() as i32,
                    ((cursor_rect.max.x - ime.cursor_rect.min.x) as f64).round() as i32,
                    ((cursor_rect.max.y - ime.cursor_rect.min.y) as f64).round() as i32,
                );
                text_input.commit();
            }
        }
    }
}
