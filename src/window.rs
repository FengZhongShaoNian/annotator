use crate::application::{Application, GlobalState};
use crate::context::{Command, WindowContext};
use crate::dpi::{LogicalPosition, LogicalSize, PhysicalSize, Position};
use crate::gpu::GpuContext;
use crate::serial::{SerialKind, SerialTracker};
use crate::view::AppView::{Child, Pop, Root};
use crate::view::sub_surface_view::SubSurfaceView;
use crate::view::surface_view::SurfaceView;
use crate::view::xdg_popup_view::{TriggerType, XdgPopupView};
use crate::view::{AppView, BuildViewFn, PopupView, SubView, View, ViewId};
use egui::ahash::{HashMap, HashMapExt};
use egui::{CursorIcon, ImeEvent, PlatformOutput};
use egui_wgpu::wgpu;
use egui_wgpu::wgpu::{CompositeAlphaMode, Surface as WgpuSurface, SurfaceCapabilities};
use indexmap::IndexMap;
use log::{error, info, warn};
use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle,
};
use sctk::compositor::Surface;
use sctk::seat::pointer::PointerEventKind;
use sctk::shell::WaylandSurface;
use sctk::shell::xdg::XdgSurface;
use sctk::shell::xdg::popup::{Popup, PopupConfigure};
use sctk::shell::xdg::window::{Window as XdgWindow, WindowDecorations};
use std::cmp::PartialEq;
use std::convert::Into;
use std::ptr::NonNull;
use std::sync::{oneshot, Arc};
use wayland_backend::client::ObjectId;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::{Proxy, QueueHandle};
use wayland_protocols::wp::fractional_scale::v1::client::wp_fractional_scale_v1::WpFractionalScaleV1;
use wayland_protocols::xdg::shell::client::xdg_positioner::XdgPositioner;
use wgpu::SurfaceConfiguration;
use crate::clipboard::Image;

const ROOT_VIEW_ID_STR: &str = "root-view";

/// AppWindow 管理应用的主窗口，包括主视图和动态管理的子视图。
pub struct AppWindow {
    pub views: IndexMap<ViewId, Option<AppView>>,
    surface_id_to_view_id: HashMap<ObjectId, ViewId>,

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
    pub window_context: WindowContext,
    /// 当前鼠标指针所在的表面
    surface_under_mouse: Option<ObjectId>,
    /// 当前窗口是否需要移除
    pub should_remove: bool,

    pub serial_tracker: SerialTracker,
}

/// 窗口的Id
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct WindowId(ObjectId);

pub struct SurfaceId(ObjectId);

pub struct WindowConfiguration {
    app_id: String,
    title: String,
    size: LogicalSize<u32>,
    preferred_size: Option<PhysicalSize<u32>>,
}

impl WindowConfiguration {
    pub fn new(
        app_id: String,
        title: String,
        size: LogicalSize<u32>,
        preferred_size: Option<PhysicalSize<u32>>,
    ) -> Self {
        WindowConfiguration {
            app_id,
            title,
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
        global_state: &GlobalState,
        window_config: WindowConfiguration,
        build_root_view: BuildViewFn,
    ) -> AppWindow {
        // 创建主表面
        let main_surface = global_state
            .compositor_state
            .create_surface(&global_state.queue_handle);

        // 创建 XDG 窗口并设置属性
        let xdg_window = global_state.xdg_shell_state.create_window(
            main_surface.clone(),
            WindowDecorations::None,
            &global_state.queue_handle,
        );
        xdg_window.set_title(window_config.title);
        xdg_window.set_app_id(window_config.app_id);
        xdg_window.commit();

        // 此时尚未拿到缩放系数，后面拿到之后会重新调整大小
        let physical_size = window_config.size.to_physical(1.);

        // 初始化 wgpu
        let (wgpu_surface, surface_config) =
            Self::create_wgpu_surface(global_state, &main_surface, physical_size);

        // 初始化分数缩放和视口
        let fractional_scale = global_state
            .fractional_scaling_manager
            .as_ref()
            .map(|ref m| m.fractional_scaling(&main_surface, &global_state.queue_handle));
        let main_viewport = global_state
            .viewporter_state
            .as_ref()
            .map(|ref viewporter_state| {
                viewporter_state.get_viewport(&main_surface, &global_state.queue_handle)
            })
            .expect("Failed to retrieve viewport");

        // 初始化主视图 (SurfaceView)
        let main_size = LogicalSize::new(window_config.size.width, window_config.size.height);
        let root_view = SurfaceView::new(
            Self::root_view_id(),
            main_surface.clone(),
            wgpu_surface,
            surface_config,
            main_size,
            1., // 此时还拿不到scale_factor
            None,
            main_viewport,
            build_root_view,
        );

        let qh = global_state.queue_handle.clone();

        let mut surface_id_to_view_id = HashMap::new();
        surface_id_to_view_id.insert(root_view.surface().id(), root_view.id());

        let mut views = IndexMap::new();
        views.insert(root_view.id(), Some(Root(Box::new(root_view))));

        let window = Self {
            views,
            surface_id_to_view_id,
            queue_handle: qh,
            xdg_window,
            first_configure: true,
            fractional_scale,
            scale_factor: None,
            keyboard_focus: false,
            preferred_size: window_config.preferred_size,
            window_context: Default::default(),
            surface_under_mouse: None,
            should_remove: false,
            serial_tracker: SerialTracker::new(),
        };

        window
    }

    pub fn root_view_id() -> ViewId {
        ROOT_VIEW_ID_STR.into()
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
        id: ViewId,
        global_state: &GlobalState,
        size: LogicalSize<u32>,
        position: LogicalPosition<i32>,
        build_view: BuildViewFn,
        position_calculator: Option<Arc<crate::view::RelativePositionCalculator>>,
    ) {
        let parent_surface = self.xdg_window.wl_surface().clone();
        let (subsurface, surface) = global_state
            .sub_compositor_state
            .create_subsurface(parent_surface, &global_state.queue_handle);

        let viewport = global_state
            .viewporter_state
            .as_ref()
            .map(|v| v.get_viewport(&surface, &global_state.queue_handle))
            .expect("Failed to retrieve viewport");

        let scale_factor = self.scale_factor().unwrap();
        let physical_size = size.to_physical(scale_factor);

        let (wgpu_surface, surface_config) =
            Self::create_wgpu_surface(global_state, &surface, physical_size);

        let subview = SubSurfaceView::new(
            id,
            surface.clone(),
            wgpu_surface,
            surface_config,
            subsurface,
            size,
            scale_factor,
            Some(position),
            viewport,
            build_view,
            position_calculator,
        );

        self.surface_id_to_view_id
            .insert(subview.surface_id(), subview.view().id());
        self.views
            .insert(subview.view_id(), Some(Child(Box::new(subview))));
    }

    fn select_alpha_mode(surface_caps: &SurfaceCapabilities) -> CompositeAlphaMode {
        let desired_alpha_mode = wgpu::CompositeAlphaMode::PreMultiplied;
        let selected_alpha_mode = if surface_caps.alpha_modes.contains(&desired_alpha_mode) {
            desired_alpha_mode
        } else {
            // 如果不支持，则回退到第一个可用的模式，通常是 Opaque
            surface_caps
                .alpha_modes
                .first()
                .copied()
                .unwrap_or(wgpu::CompositeAlphaMode::Auto)
        };
        selected_alpha_mode
    }

    pub fn create_positioner(&self, global_state: &GlobalState) -> XdgPositioner {
        let qh = &global_state.queue_handle;
        let xdg_shell_state = &global_state.xdg_shell_state;
        xdg_shell_state.xdg_wm_base().create_positioner(qh, ())
    }

    /// 创建xdg-popup
    /// 需要注意：positioner的anchor_rect必须是root-view上的有效区域，
    /// 也就是弹窗必须和xdg-toplevel挨着，不然弹窗不显示
    pub fn create_xdg_popup_view(
        &mut self,
        id: ViewId,
        global_state: &GlobalState,
        trigger_type: TriggerType,
        positioner: XdgPositioner,
        build_view: BuildViewFn,
    ) -> SurfaceId {
        let parent_xdg_surface = self.xdg_window.xdg_surface();
        let qh = &global_state.queue_handle;
        let xdg_shell_state = &global_state.xdg_shell_state;
        let compositor = &global_state.compositor_state;

        let surface = Surface::new(compositor, qh).unwrap();
        let popup = Popup::from_surface(
            Some(parent_xdg_surface),
            &positioner,
            qh,
            surface,
            xdg_shell_state,
        )
        .expect("Failed to create popup");

        let mut serial = None;
        let mut seat = global_state.seat.clone();
        match trigger_type {
            TriggerType::MousePress => {
                serial = Some(self.serial_tracker.get(SerialKind::MousePress));
            }
            TriggerType::KeyPress => {
                serial = Some(self.serial_tracker.get(SerialKind::KeyPress));
            }
            TriggerType::Touch => {
                todo!()
            }
        }
        popup.xdg_popup().grab(&seat.unwrap(), serial.unwrap());

        let surface = popup.wl_surface();
        surface.commit();

        let viewport = global_state
            .viewporter_state
            .as_ref()
            .map(|v| v.get_viewport(surface, &global_state.queue_handle))
            .expect("Failed to retrieve viewport");

        // 暂时先随便设置一个尺寸，真正的尺寸需要等xdg_popup的configure事件通知
        let default_logical_size = LogicalSize::new(120, 48);
        let default_physical_size =
            default_logical_size.to_physical(self.scale_factor.unwrap_or(1.));

        let (wgpu_surface, surface_config) =
            Self::create_wgpu_surface(&global_state, surface, default_physical_size);
        let scale_factor = self.scale_factor.unwrap();
        let mut popup_view = XdgPopupView::new(
            id,
            surface.clone(),
            popup,
            positioner,
            wgpu_surface,
            surface_config,
            default_logical_size,
            scale_factor,
            viewport,
            build_view,
        );
        let subview_id = SurfaceId(popup_view.view().surface().id());

        popup_view.view_mut().surface().commit(); // Initial commit

        self.surface_id_to_view_id
            .insert(popup_view.view().surface().id(), popup_view.view().id());
        self.views
            .insert(popup_view.view().id(), Some(Pop(Box::new(popup_view))));

        subview_id
    }

    fn create_wgpu_surface(
        global_state: &GlobalState,
        surface: &WlSurface,
        surface_size: PhysicalSize<u32>,
    ) -> (WgpuSurface<'static>, SurfaceConfiguration) {
        // Create the raw window handle for the surface.
        let raw_display_handle = RawDisplayHandle::Wayland(WaylandDisplayHandle::new(
            NonNull::new(global_state.connection.backend().display_ptr() as *mut _).unwrap(),
        ));
        let raw_window_handle = RawWindowHandle::Wayland(WaylandWindowHandle::new(
            NonNull::new(surface.id().as_ptr() as *mut _).unwrap(),
        ));

        // 初始化 wgpu
        let wgpu_initialized = global_state.gpu.borrow().as_ref().is_some();
        let wgpu_surface = if !wgpu_initialized {
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
            let gpu_context = GpuContext::new(instance, &surface).unwrap();
            global_state.gpu.replace(Some(gpu_context));

            surface
        } else {
            let gpu_context = global_state.gpu.borrow();
            let gpu_context = gpu_context.as_ref().unwrap();
            let instance = &gpu_context.instance;
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

        let gpu_context = global_state.gpu.borrow();
        let gpu_context = gpu_context.as_ref().unwrap();

        let surface_caps = wgpu_surface.get_capabilities(&gpu_context.adapter);
        let surface_format = surface_caps.formats[0];
        let selected_alpha_mode = Self::select_alpha_mode(&surface_caps);
        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,

            // width 和 height 指定 SurfaceTexture 的宽度和高度（物理像素，等于逻辑像素乘以屏幕缩放因子）
            width: surface_size.width,
            height: surface_size.height,

            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: selected_alpha_mode,
            view_formats: vec![],
        };

        wgpu_surface.configure(&gpu_context.device, &config);
        (wgpu_surface, config)
    }

    pub fn handle_pointer_event(
        &mut self,
        event: &sctk::seat::pointer::PointerEvent,
        globals: &GlobalState,
    ) {
        let event_surface = &event.surface;
        match event.kind {
            PointerEventKind::Enter { serial } => {
                self.surface_under_mouse = Some(event_surface.id());
                self.serial_tracker.update(SerialKind::MouseEnter, serial);
            }
            PointerEventKind::Leave { .. } => {
                self.surface_under_mouse = None;
            }
            PointerEventKind::Press { serial, .. } => {
                self.serial_tracker.update(SerialKind::MousePress, serial);
            }
            _ => (),
        }

        let view_id = self.surface_id_to_view_id.get(&event.surface.id());
        let Some(view_id) = view_id else {
            return;
        };
        let app_view = self.views.get_mut(view_id).unwrap().as_mut().unwrap();
        let view = app_view.get_view_ref_mut();
        view.handle_pointer_event(event, globals);
    }

    pub fn handle_keyboard_event(
        &mut self,
        event: sctk::seat::keyboard::KeyEvent,
        serial: u32,
        pressed: bool,
        repeat: bool,
    ) {
        self.serial_tracker.update(SerialKind::KeyPress, serial);
        self.views.values_mut().for_each(|app_view| {
            let app_view = app_view.as_mut().unwrap();
            let view = app_view.get_view_ref_mut();
            view.handle_keyboard_event(event.clone(), pressed, repeat);
        });
    }

    pub fn update_modifiers(&mut self, modifiers: sctk::seat::keyboard::Modifiers) {
        self.views.values_mut().for_each(|app_view| {
            let app_view = app_view.as_mut().unwrap();
            let view = app_view.get_view_ref_mut();
            view.update_modifiers(modifiers.clone());
        });
    }

    pub fn handle_ime_event(&mut self, event: &ImeEvent) {
        self.views.values_mut().for_each(|app_view| {
            let app_view = app_view.as_mut().unwrap();
            let view = app_view.get_view_ref_mut();
            view.handle_ime_event(event);
        });
    }

    pub fn contains_surface(&self, surface: &WlSurface) -> bool {
        self.surface_id_to_view_id.contains_key(&surface.id())
    }

    pub fn xdg_window(&self) -> &XdgWindow {
        &self.xdg_window
    }

    pub fn scale_factor(&self) -> &Option<f64> {
        &self.scale_factor
    }

    pub fn set_scale_factor(&mut self, new_scale_factor: f64, gpu: &GpuContext) {
        let first_time_set_scale_factor = self.scale_factor.is_none();
        self.scale_factor = Some(new_scale_factor);

        if first_time_set_scale_factor {
            // 如果窗口设置了preferred_size，那么根据这个尺寸调整窗口大小
            if let Some(preferred_size) = self.preferred_size {
                let new_size = preferred_size.to_logical(new_scale_factor);
                self.resize_root_view(new_size, gpu);
            }
        }

        self.views.values_mut().for_each(|app_view| {
            let app_view = app_view.as_mut().unwrap();
            let view = app_view.get_view_ref_mut();
            view.set_scale_factor(new_scale_factor, gpu);
        });

        let root_view_id = Self::root_view_id();
        let root_view = self.views.get(&root_view_id);
        let root_view = root_view.unwrap();
        let view = root_view.as_ref().unwrap().get_view_ref();
        let parent_surface_size = view.viewport_size();
        self.views
            .values_mut()
            .filter(|app_view| matches!(app_view, Some(Child(..))))
            .for_each(|app_view| {
                let mut sub_view = match app_view.as_mut().unwrap() {
                    Child(sub_view) => Some(sub_view),
                    _ => None,
                };
                let sub_view = sub_view.as_mut().unwrap();
                let subview_size = sub_view.view().viewport_size();
                if let Some(position_calculator) = sub_view.position_calculator() {
                    let new_position = position_calculator(&parent_surface_size, &subview_size);
                    sub_view.set_position(new_position.to_logical(new_scale_factor));
                }
            });
    }

    pub fn resize_root_view(&mut self, new_size: LogicalSize<u32>, gpu: &GpuContext) {
        let root_view_id = Self::root_view_id();
        let mut root_view = self.views.get_mut(&root_view_id);
        let root_view = root_view.as_mut().unwrap();
        let root_view = root_view.as_mut().unwrap();
        let view = root_view.get_view_ref_mut();

        view.resize(new_size, gpu);

        let parent_surface_size = view.viewport_size();
        self.views
            .values_mut()
            .filter(|app_view| matches!(app_view, Some(Child(..))))
            .for_each(|app_view| {
                let mut sub_view = match app_view.as_mut().unwrap() {
                    Child(sub_view) => Some(sub_view),
                    _ => None,
                };
                let sub_view = sub_view.as_mut().unwrap();
                let subview_size = sub_view.view().viewport_size();
                if let Some(position_calculator) = sub_view.position_calculator() {
                    let new_position = position_calculator(&parent_surface_size, &subview_size);
                    sub_view.set_position(new_position.to_logical(self.scale_factor.unwrap()));
                }
            });
    }

    pub fn set_keyboard_focus(&mut self, focus: bool) {
        self.keyboard_focus = focus;
    }

    pub fn keyboard_focus(&self) -> bool {
        self.keyboard_focus
    }

    /// 执行窗口重绘逻辑。
    /// 遍历所有视图并调用其独立的渲染方法。
    pub fn draw(&mut self, app: &mut Application) {
        if self.first_configure || self.scale_factor.is_none() {
            return;
        }

        let mut view_ids = vec![];
        {
            for (view_id, _app_view) in &self.views {
                view_ids.push(view_id.clone());
            }
        }

        for view_id in view_ids {
            let mut app_view = self.views.get_mut(&view_id);
            let app_view = app_view.as_mut().unwrap();

            // 注意！这里会将self.app_views中view_id对应的value置为None，
            // 所以如果尝试在BuildViewFn中访问当前self.app_views[view_id]，
            // 将会获取到None
            let mut app_view = app_view.take().unwrap();

            match app_view {
                Pop(ref popup_view) => {
                    // 如果PopupView尚未完成首次配置，那么不进行绘制
                    if !popup_view.first_configure_done() {
                        self.views.insert(view_id, Some(app_view));
                        continue;
                    }
                }
                _ => (),
            }

            let view = app_view.get_view_ref_mut();
            let output = view.draw(app, self);

            let surface_id = view.surface().id().clone();
            if view.should_remove() {
                let surface_id = view.surface().id();
                self.views.shift_remove(&view_id);
                self.surface_id_to_view_id.remove(&surface_id);
                info!("Removing view {:?}", view_id);
            } else {
                self.views.insert(view_id, Some(app_view));
            }
            if let Some(platform_output) = output {
                let global_state = &app.global_state;
                Self::update_ime_position_if_necessary(&platform_output, global_state);
                if self.surface_under_mouse == Some(surface_id) {
                    Self::update_cursor_icon_if_necessary(&platform_output, global_state);
                }
            }
        }

        while let Some(command) = self.window_context.commands.pop_front() {
            match command {
                Command::HideView(id) => {
                    let view = self.views.get_mut(&id);
                    if let Some(view) = view {
                        if let Some(view) = view.as_mut() {
                            let view = view.get_view_ref_mut();
                            view.set_visible(false);
                        }
                    }
                }
                Command::ResizeView(id, new_size) => {
                    let mut gpu_context = app.global_state.gpu.borrow_mut();
                    let gpu_context = gpu_context.as_mut().expect("GPU context not initialized!");
                    let view = self.views.get_mut(&id);
                    if let Some(view) = view {
                        if let Some(view) = view.as_mut() {
                            match view {
                                Root(view) => {
                                    view.resize(new_size, gpu_context);
                                }
                                Child(sub_view) => {
                                    sub_view.view_mut().resize(new_size, gpu_context);
                                }
                                Pop(popup_view) => {
                                    popup_view.view_mut().resize(new_size, gpu_context);
                                }
                            }
                        }
                    }
                }
                Command::DropView(id) => {
                    let app_view = self.views.remove(&id);
                    if let Some(Some(app_view)) = app_view {
                        self.surface_id_to_view_id.remove(&app_view.surface_id());
                    }
                }
                Command::RepositionSubView(id, position) => {
                    let view = self.views.get_mut(&id);
                    if let Some(Some(app_view)) = view {
                        match app_view {
                            Child(sub_view) => {
                                sub_view.set_position(position);
                            }
                            _  => {
                                warn!("AppView with id {:?} is not type of SubView", id);
                            }
                        }
                    }else {
                        warn!("AppView with id {:?} not found", id);
                    }
                }
                Command::CopyImage(receiver) => {
                    let clipboard = &mut app.global_state.clipboard;

                    match receiver.recv() {
                        Ok(rgba_image) => {
                            let image = Image::from(rgba_image);
                            clipboard.store(image);
                        }
                        Err(err) => {
                            warn!("Failed to send data to clipboard: {:?}", err);
                        }
                    }
                }
                Command::StartMovingWindow => {
                    self.xdg_window.move_(app.global_state.seat.as_ref().unwrap(), self.serial_tracker.get(SerialKind::MousePress));
                }
            }
        }
    }

    fn update_ime_position_if_necessary(
        platform_output: &PlatformOutput,
        global_state: &GlobalState,
    ) {
        if let Some(text_input) = global_state.text_input.as_ref() {
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

    fn update_cursor_icon_if_necessary(
        platform_output: &PlatformOutput,
        global_state: &GlobalState,
    ) {
        if let Some(themed_pointer) = global_state.themed_pointer.as_ref() {
            use sctk::seat::pointer::CursorIcon as SctkCursorIcon;
            let cursor_icon = match platform_output.cursor_icon {
                CursorIcon::Default => Some(SctkCursorIcon::Default),
                CursorIcon::None => None,
                CursorIcon::ContextMenu => Some(SctkCursorIcon::ContextMenu),
                CursorIcon::Help => Some(SctkCursorIcon::Help),
                CursorIcon::PointingHand => Some(SctkCursorIcon::Pointer),
                CursorIcon::Progress => Some(SctkCursorIcon::Progress),
                CursorIcon::Wait => Some(SctkCursorIcon::Wait),
                CursorIcon::Cell => Some(SctkCursorIcon::Cell),
                CursorIcon::Crosshair => Some(SctkCursorIcon::Crosshair),
                CursorIcon::Text => Some(SctkCursorIcon::Text),
                CursorIcon::VerticalText => Some(SctkCursorIcon::VerticalText),
                CursorIcon::Alias => Some(SctkCursorIcon::Alias),
                CursorIcon::Copy => Some(SctkCursorIcon::Copy),
                CursorIcon::Move => Some(SctkCursorIcon::Move),
                CursorIcon::NoDrop => Some(SctkCursorIcon::NoDrop),
                CursorIcon::NotAllowed => Some(SctkCursorIcon::NotAllowed),
                CursorIcon::Grab => Some(SctkCursorIcon::Grab),
                CursorIcon::Grabbing => Some(SctkCursorIcon::Grabbing),
                CursorIcon::AllScroll => Some(SctkCursorIcon::AllScroll),
                CursorIcon::ResizeHorizontal => Some(SctkCursorIcon::EwResize),
                CursorIcon::ResizeNeSw => Some(SctkCursorIcon::SwResize),
                CursorIcon::ResizeNwSe => Some(SctkCursorIcon::NwResize),
                CursorIcon::ResizeVertical => Some(SctkCursorIcon::NsResize),
                CursorIcon::ResizeEast => Some(SctkCursorIcon::EResize),
                CursorIcon::ResizeSouthEast => Some(SctkCursorIcon::SeResize),
                CursorIcon::ResizeSouth => Some(SctkCursorIcon::SResize),
                CursorIcon::ResizeSouthWest => Some(SctkCursorIcon::SwResize),
                CursorIcon::ResizeWest => Some(SctkCursorIcon::WResize),
                CursorIcon::ResizeNorthWest => Some(SctkCursorIcon::NwResize),
                CursorIcon::ResizeNorth => Some(SctkCursorIcon::NResize),
                CursorIcon::ResizeNorthEast => Some(SctkCursorIcon::NeResize),
                CursorIcon::ResizeColumn => Some(SctkCursorIcon::ColResize),
                CursorIcon::ResizeRow => Some(SctkCursorIcon::RowResize),
                CursorIcon::ZoomIn => Some(SctkCursorIcon::ZoomIn),
                CursorIcon::ZoomOut => Some(SctkCursorIcon::ZoomOut),
            };

            if let Some(cursor_icon) = cursor_icon {
                let connection = &global_state.connection;
                if let Err(e) = themed_pointer.set_cursor(connection, cursor_icon) {
                    warn!("Failed to set cursor icon: {:?}", e);
                }
            } else {
                if let Err(e) = themed_pointer.hide_cursor() {
                    warn!("Failed to hide cursor icon: {:?}", e);
                }
            }
        }
    }

    pub fn root_surface(&self) -> &WlSurface {
        self.xdg_window.wl_surface()
    }

    pub fn window_id(&self) -> WindowId {
        WindowId(self.root_surface().id())
    }

    pub fn configure_popup(&mut self, popup: &Popup, config: &PopupConfigure, gpu: &GpuContext) {
        for app_view in &mut self.views.values_mut() {
            match app_view.as_mut().unwrap() {
                Pop(popup_view) => {
                    if popup_view.popup() == popup {
                        popup_view.record_position(LogicalPosition::new(
                            config.position.0,
                            config.position.1,
                        ));
                        popup_view.view_mut().resize(
                            LogicalSize::new(config.width as u32, config.height as u32),
                            gpu,
                        );
                        if !popup_view.first_configure_done() {
                            popup_view.set_first_configure_done();
                        }
                    }
                }
                _ => (),
            }
        }
    }

    pub fn remove_popup(&mut self, popup: &Popup) {
        let mut view_id = None;
        self.views.iter().for_each(|(id, app_view)| match app_view {
            Some(Pop(popup_view)) => {
                if popup_view.popup() == popup {
                    view_id = Some(id.clone());
                }
            }
            _ => (),
        });
        if let Some(view_id) = view_id {
            info!("Removing popup: {:?}", view_id);
            let view = self.views.shift_remove(&view_id);
            if let Some(Some(app_view)) = view {
                match app_view {
                    Pop(popup_view) => {
                        self.surface_id_to_view_id
                            .remove(&popup_view.view().surface_id());
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn set_view_visible(&mut self, view_id: &ViewId, visible: bool) {
        let app_view = self.views.get_mut(view_id);
        if let Some(app_view) = app_view {
            if let Some(app_view) = app_view {
                let view = app_view.get_view_ref_mut();
                view.set_visible(visible);
            }
        }
    }
}
