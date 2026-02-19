use crate::annotator::rectangle::{RectangleAnnotationTool, RectangleState};
use crate::annotator::{Annotation, AnnotatorState, ToolType};
use crate::context::WindowContext;
use crate::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use crate::global::{ReadGlobal, UpdateGlobal};
use crate::gpu::GpuContext;
use crate::view::{BuildViewFn, SubView};
use crate::window::{AppWindow, WindowConfiguration, WindowId};
use crate::wp_fractional_scaling::FractionalScalingManager;
use crate::wp_viewporter::ViewporterState;
use egui::load::SizedTexture;
use egui::{
    Area, Color32, ColorImage, Id, Image, ImageSource, ImeEvent, Order, Pos2, Rect, RichText,
    Stroke, StrokeKind, TextEdit, Vec2, pos2, vec2,
};
use image::{GenericImageView, RgbaImage};
use log::{info, warn};
use sctk::compositor::{CompositorHandler, CompositorState};
use sctk::globals::GlobalData;
use sctk::output::{OutputHandler, OutputState};
use sctk::reexports::calloop::{EventLoop, LoopHandle};
use sctk::registry::{ProvidesRegistryState, RegistryState};
use sctk::seat::pointer::{CursorIcon, PointerData, PointerDataExt, ThemeSpec, ThemedPointer};
use sctk::seat::{
    Capability, SeatHandler, SeatState,
    keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
    pointer::{PointerEvent, PointerEventKind, PointerHandler},
};
use sctk::shell::xdg::XdgShell;
use sctk::shell::xdg::window::{Window, WindowConfigure, WindowHandler};
use sctk::shm::{Shm, ShmHandler};
use sctk::subcompositor::SubcompositorState;
use sctk::{
    delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm, delegate_subcompositor, delegate_xdg_shell, delegate_xdg_window,
    registry_handlers,
};
use std::cell::{Cell, RefCell};
use std::cmp::PartialEq;
use std::sync::{Arc, Mutex, RwLock};
use wayland_client::globals::{GlobalList, registry_queue_init};
use wayland_client::protocol::wl_keyboard::WlKeyboard;
use wayland_client::protocol::wl_pointer::WlPointer;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::protocol::{wl_output, wl_surface};
use wayland_client::{
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, delegate_dispatch, delegate_noop,
};
use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::WpCursorShapeDeviceV1;
use wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_manager_v1::WpCursorShapeManagerV1;
use wayland_protocols::wp::text_input::zv3::client::zwp_text_input_v3::{
    ContentHint, ContentPurpose,
};
use wayland_protocols::wp::text_input::zv3::client::{
    zwp_text_input_manager_v3, zwp_text_input_v3,
};

/// GlobalState 存储了 Wayland 的全局状态和协议处理器。
pub struct GlobalState {
    pub connection: Connection,
    /// 注册表状态，用于管理 Wayland 全局对象。
    pub registry_state: RegistryState,
    /// 输出设备状态（显示器信息）。
    pub output_state: OutputState,
    /// 合成器状态。
    pub compositor_state: CompositorState,
    /// 子合成器状态，用于管理 SubSurface。
    pub sub_compositor_state: SubcompositorState,
    /// 视口管理器（用于调整 Surface 显示尺寸）。
    pub viewporter_state: Option<ViewporterState>,
    /// 分数缩放管理器。
    pub fractional_scaling_manager: Option<FractionalScalingManager>,
    /// XDG Shell 状态，用于管理窗口。
    pub xdg_shell_state: XdgShell,
    /// 事件队列。
    event_queue: Option<EventQueue<Application>>,
    /// 队列句柄。
    pub queue_handle: QueueHandle<Application>,
    /// GPU 上下文（EGL/Skia）。
    pub gpu: RefCell<Option<GpuContext>>,
    /// 座位状态（管理输入设备）。
    pub seat_state: SeatState,
    /// 最近一次的序列号（用于同步）。
    pub last_serial: u32,
    /// 键盘实例。
    keyboard: Option<WlKeyboard>,

    /// 指针实例。
    pub themed_pointer: Option<ThemedPointer<PointerData>>,
    shm_state: Shm,

    /// 事件循环句柄。
    loop_handle: LoopHandle<'static, Application>,
    pub text_input_manager: Option<zwp_text_input_manager_v3::ZwpTextInputManagerV3>,
    pub text_input: Option<zwp_text_input_v3::ZwpTextInputV3>,
}

/// Application 是应用的核心结构，管理全局状态和窗口列表。
pub struct Application {
    /// 全局状态。
    pub global_state: GlobalState,
    /// 应用 ID。
    pub app_id: &'static str,
    /// 窗口列表。
    windows: Vec<AppWindow>,
}

impl Application {
    /// 初始化 Application，建立 Wayland 连接并准备 GPU 环境。
    pub fn new(app_id: &'static str) -> Application {
        let conn = Connection::connect_to_env().expect("Can't connect to the wayland server");

        let (globals, event_queue) = registry_queue_init(&conn).unwrap();
        let qh = event_queue.handle();

        let compositor_state =
            CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
        let sub_compositor_state =
            SubcompositorState::bind(compositor_state.wl_compositor().clone(), &globals, &qh)
                .expect("wl_subcompositor not available");

        let (viewporter_state, fractional_scaling_manager) =
            if let Ok(fsm) = FractionalScalingManager::new(&globals, &qh) {
                (ViewporterState::new(&globals, &qh).ok(), Some(fsm))
            } else {
                (None, None)
            };
        let event_loop: EventLoop<Application> =
            EventLoop::try_new().expect("Failed to initialize the event loop!");

        let seat_state = SeatState::new(&globals, &qh);
        let shm_state = Shm::bind(&globals, &qh).expect("wl shm not available");
        let text_input_manager = globals.bind(&qh, 1..=1, ()).ok();
        let mut app = Self {
            global_state: GlobalState {
                connection: conn,
                registry_state: RegistryState::new(&globals),
                output_state: OutputState::new(&globals, &qh),
                compositor_state,
                sub_compositor_state,
                viewporter_state,
                fractional_scaling_manager,
                xdg_shell_state: XdgShell::bind(&globals, &qh).expect("xdg shell not available"),
                event_queue: None,
                queue_handle: qh,
                gpu: RefCell::new(None),
                seat_state,
                last_serial: 0,
                keyboard: None,
                themed_pointer: None,
                shm_state,
                loop_handle: event_loop.handle(),
                text_input_manager,
                text_input: None,
            },
            app_id,
            windows: vec![],
        };

        app.global_state.event_queue = Some(event_queue);
        app
    }

    pub fn open_window(
        &mut self,
        window_config: WindowConfiguration,
        build_root_view: BuildViewFn,
    ) -> WindowId {
        let global_state = &self.global_state;
        let window = AppWindow::new(global_state, window_config, build_root_view);
        let window_id = window.window_id();
        self.windows.push(window);
        window_id
    }

    pub fn with_window_mut<F>(&mut self, window_id: WindowId, func: F)
    where
        F: FnOnce(&GlobalState, &mut Option<&mut AppWindow>),
    {
        let mut target_window_idx = None;
        for (idx, w) in self.windows.iter().enumerate() {
            if w.window_id() == window_id {
                target_window_idx = Some(idx);
                break;
            }
        }
        let mut window = if let Some(idx) = target_window_idx {
            Some(&mut self.windows[idx])
        } else {
            None
        };
        let global_state = &self.global_state;
        func(global_state, &mut window);
    }

    pub fn scale_factor_changed(
        &mut self,
        surface: &WlSurface,
        scale_factor: f64,
        is_legacy: bool,
    ) {
        if is_legacy && self.global_state.fractional_scaling_manager.is_some() {
            // 使用分数缩放的情况下忽略整数缩放倍数
            return;
        }
        info!("scale factor changed to {}", scale_factor);
        self.windows.iter_mut().for_each(|w| {
            // 如果窗口的scale_factor不存在，意味着窗口尚未开始绘制
            let old_scale_factor_is_none = w.scale_factor().is_none();
            let gpu_context = self.global_state.gpu.borrow();
            let gpu_context = gpu_context.as_ref().unwrap();
            if w.contains_surface(surface) {
                w.set_scale_factor(scale_factor, gpu_context);
            }
            if old_scale_factor_is_none && w.first_configure == false {
                // 为了能正确绘制，窗口需要等待首次配置完成并且获取到了缩放倍数，再开始首次绘制

                // 如果窗口设置了preferred_size，那么根据这个尺寸调整窗口大小
                if let Some(preferred_size) = w.preferred_size {
                    let new_size = preferred_size.to_logical(scale_factor);
                    w.resize(new_size, gpu_context);
                }
                w.draw(&self.global_state);
            }
        })
    }

    pub fn run(&mut self) {
        let mut event_queue = self.global_state.event_queue.take().unwrap();
        loop {
            event_queue.blocking_dispatch(self).unwrap();
        }
    }
}

delegate_registry!(Application);
delegate_compositor!(Application);
delegate_subcompositor!(Application);
delegate_output!(Application);

delegate_xdg_shell!(Application);
delegate_xdg_window!(Application);

delegate_seat!(Application);
delegate_keyboard!(Application);
delegate_dispatch!(Application: [ WlPointer: PointerData] => SeatState);
delegate_dispatch!(Application: [ WpCursorShapeManagerV1: GlobalData] => SeatState);
delegate_dispatch!(Application: [ WpCursorShapeDeviceV1: GlobalData] => SeatState);
delegate_noop!(Application: ignore zwp_text_input_manager_v3::ZwpTextInputManagerV3);
delegate_shm!(Application);

impl CompositorHandler for Application {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        new_factor: i32,
    ) {
        self.scale_factor_changed(surface, new_factor as f64, true);
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
        // Not needed for this example.
        info!("transform changed");
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        surface: &WlSurface,
        _time: u32,
    ) {
        for window in &mut self.windows {
            // 只在主 Surface (main_view) 的帧回调到达时触发重绘
            // 这样可以保证渲染频率与显示刷新率同步，避免过度提交
            if window.main_view.surface() == surface {
                window.draw(&self.global_state);
                return; // 找到对应的窗口并重绘后退出
            }
        }
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
        info!("Surface entered");
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
        info!("Surface leaved");
    }
}

impl OutputHandler for Application {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.global_state.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }
}

impl ProvidesRegistryState for Application {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.global_state.registry_state
    }

    registry_handlers!(OutputState);
}

impl WindowHandler for Application {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, window: &Window) {
        self.windows.retain(|v| v.xdg_window() != window);
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        window: &Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        info!("Window configured to: {:?}", configure);
        for app_window in &mut self.windows {
            if app_window.xdg_window() != window {
                continue;
            }

            if app_window.first_configure {
                // 为了能正确绘制，窗口需要等待首次配置完成并且获取到了缩放倍数
                // 开始首次绘制
                app_window.first_configure = false;

                if app_window.scale_factor().is_some() {
                    app_window.draw(&self.global_state);
                }
            }
        }
    }
}

impl SeatHandler for Application {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.global_state.seat_state
    }

    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.global_state.keyboard.is_none() {
            println!("Set keyboard capability");
            let keyboard = self
                .global_state
                .seat_state
                .get_keyboard_with_repeat(
                    qh,
                    &seat,
                    None,
                    self.global_state.loop_handle.clone(),
                    Box::new(|_state, _wl_kbd, event| {
                        println!("Repeat: {:?} ", event);
                    }),
                )
                .expect("Failed to create keyboard");

            self.global_state.keyboard = Some(keyboard);

            self.global_state.text_input = self
                .global_state
                .text_input_manager
                .as_ref()
                .map(|text_input_manager| text_input_manager.get_text_input(&seat, qh, ()));
        }

        if capability == Capability::Pointer && self.global_state.themed_pointer.is_none() {
            println!("Set pointer capability");
            let surface = self.global_state.compositor_state.create_surface(qh);
            let pointer_data = PointerData::new(seat.clone());
            let themed_pointer = self
                .global_state
                .seat_state
                .get_pointer_with_theme_and_data(
                    qh,
                    &seat,
                    self.global_state.shm_state.wl_shm(),
                    surface,
                    ThemeSpec::default(),
                    pointer_data,
                )
                .expect("Failed to create pointer");
            self.global_state.themed_pointer.replace(themed_pointer);
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.global_state.keyboard.is_some() {
            println!("Unset keyboard capability");
            self.global_state.keyboard.take().unwrap().release();
        }

        if capability == Capability::Pointer && self.global_state.themed_pointer.is_some() {
            println!("Unset pointer capability");
            self.global_state
                .themed_pointer
                .take()
                .unwrap()
                .pointer()
                .release();
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: WlSeat) {}
}

impl KeyboardHandler for Application {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        surface: &WlSurface,
        _: u32,
        _: &[u32],
        keysyms: &[Keysym],
    ) {
        let window = self
            .windows
            .iter_mut()
            .find(|w| w.contains_surface(surface));

        if let Some(window) = window {
            window.set_keyboard_focus(true);
        }
    }

    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        surface: &wl_surface::WlSurface,
        _: u32,
    ) {
        let window = self
            .windows
            .iter_mut()
            .find(|w| w.contains_surface(surface));

        if let Some(window) = window {
            window.set_keyboard_focus(false);
        }
    }

    fn press_key(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _: &WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        for window in &mut self.windows {
            if window.keyboard_focus() {
                window.handle_keyboard_event(event.clone(), true, false);
            }
        }
    }

    fn repeat_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        for window in &mut self.windows {
            if window.keyboard_focus() {
                window.handle_keyboard_event(event.clone(), true, true);
            }
        }
    }

    fn release_key(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        _: u32,
        event: KeyEvent,
    ) {
        for window in &mut self.windows {
            if window.keyboard_focus() {
                window.handle_keyboard_event(event.clone(), false, false);
            }
        }
    }

    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &WlKeyboard,
        _serial: u32,
        modifiers: Modifiers,
        _raw_modifiers: RawModifiers,
        _layout: u32,
    ) {
        for window in &mut self.windows {
            if window.keyboard_focus() {
                window.update_modifiers(modifiers);
            }
        }
    }
}

impl PointerHandler for Application {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &WlPointer,
        events: &[PointerEvent], // 指针事件使用的是逻辑坐标
    ) {
        for event in events {
            // Ignore events for other window
            let mut target_window_idx = None;
            for (idx, w) in self.windows.iter().enumerate() {
                if w.contains_surface(&event.surface) {
                    target_window_idx = Some(idx);
                    break;
                }
            }

            let enter_event = matches!(event.kind, PointerEventKind::Enter { .. });
            if let Some(themed_cursor) = self.global_state.themed_pointer.as_ref()
                && enter_event
            {
                let connection = &self.global_state.connection;
                if let Err(e) = themed_cursor.set_cursor(connection, CursorIcon::Default) {
                    warn!("Failed tp set cursor: {:?}", e);
                }
            }

            if let Some(idx) = target_window_idx {
                self.windows[idx].handle_pointer_event(event, &self.global_state);
            }
        }
    }
}

impl Dispatch<zwp_text_input_v3::ZwpTextInputV3, ()> for Application {
    fn event(
        this: &mut Self,
        _text_input: &zwp_text_input_v3::ZwpTextInputV3,
        event: <zwp_text_input_v3::ZwpTextInputV3 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwp_text_input_v3::Event::Enter { .. } => {
                let Some(text_input) = this.global_state.text_input.take() else {
                    return;
                };

                text_input.enable();
                text_input.set_content_type(ContentHint::None, ContentPurpose::Normal);

                for window in &mut this.windows {
                    if window.keyboard_focus() {
                        window.handle_ime_event(&ImeEvent::Enabled);
                    }
                }
                text_input.commit();

                this.global_state.text_input = Some(text_input);
            }
            zwp_text_input_v3::Event::Leave { .. } => {
                if let Some(text_input) = &this.global_state.text_input {
                    text_input.disable();
                    text_input.commit();
                }

                for window in &mut this.windows {
                    if window.keyboard_focus() {
                        window.handle_ime_event(&ImeEvent::Disabled);
                    }
                }
            }
            zwp_text_input_v3::Event::CommitString { text } => {
                let Some(text) = text else {
                    return;
                };

                for window in &mut this.windows {
                    if window.keyboard_focus() {
                        window.handle_ime_event(&ImeEvent::Commit(text.clone()));
                    }
                }
            }
            zwp_text_input_v3::Event::PreeditString { text, .. } => {
                let Some(text) = text else {
                    return;
                };

                for window in &mut this.windows {
                    if window.keyboard_focus() {
                        window.handle_ime_event(&ImeEvent::Preedit(text.clone()));
                    }
                }
            }
            zwp_text_input_v3::Event::Done { serial } => {
                this.global_state.last_serial = serial;
            }
            _ => {}
        }
    }
}

impl ShmHandler for Application {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.global_state.shm_state
    }
}

impl Dispatch<WpCursorShapeDeviceV1, GlobalData, Application> for SeatState {
    fn event(
        _: &mut Application,
        _: &WpCursorShapeDeviceV1,
        _: <WpCursorShapeDeviceV1 as Proxy>::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<Application>,
    ) {
        unreachable!("wp_cursor_shape_manager has no events")
    }
}

impl Dispatch<WpCursorShapeManagerV1, GlobalData, Application> for SeatState {
    fn event(
        _: &mut Application,
        _: &WpCursorShapeManagerV1,
        _: <WpCursorShapeManagerV1 as Proxy>::Event,
        _: &GlobalData,
        _: &Connection,
        _: &QueueHandle<Application>,
    ) {
        unreachable!("wp_cursor_device_manager has no events")
    }
}
