use crate::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use crate::gpu::GpuContext;
use crate::view::SubView;
use crate::window::AppWindow;
use crate::wp_fractional_scaling::FractionalScalingManager;
use crate::wp_viewporter::ViewporterState;
use egui::load::SizedTexture;
use egui::{Color32, ColorImage, Id, Image, ImageSource, Order, Pos2, Rect, RichText};
use image::{GenericImageView, RgbaImage};
use log::info;
use sctk::compositor::{CompositorHandler, CompositorState};
use sctk::output::{OutputHandler, OutputState};
use sctk::reexports::calloop::{EventLoop, LoopHandle};
use sctk::registry::{ProvidesRegistryState, RegistryState};
use sctk::seat::{
    Capability, SeatHandler, SeatState,
    keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers, RawModifiers},
    pointer::{PointerEvent, PointerEventKind, PointerHandler},
};
use sctk::shell::xdg::XdgShell;
use sctk::shell::xdg::window::{Window, WindowConfigure, WindowHandler};
use sctk::subcompositor::SubcompositorState;
use sctk::{
    delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_subcompositor, delegate_xdg_shell, delegate_xdg_window,
    registry_handlers,
};
use std::sync::Arc;
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::wl_keyboard::WlKeyboard;
use wayland_client::protocol::wl_pointer::WlPointer;
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::protocol::{wl_output, wl_surface};
use wayland_client::{Connection, EventQueue, QueueHandle};
use crate::annotator::{BackgroundTexture};
use crate::global::{ReadGlobal, UpdateGlobal};

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
    pub gpu: Option<GpuContext>,
    /// 座位状态（管理输入设备）。
    pub seat_state: SeatState,
    /// 最近一次的序列号（用于同步）。
    pub last_serial: u32,
    /// 键盘实例。
    keyboard: Option<WlKeyboard>,
    /// 指针实例。
    pointer: Option<WlPointer>,
    /// 事件循环句柄。
    loop_handle: LoopHandle<'static, Application>,
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
                gpu: None,
                seat_state,
                last_serial: 0,
                keyboard: None,
                pointer: None,
                loop_handle: event_loop.handle(),
            },
            app_id,
            windows: vec![],
        };

        app.global_state.event_queue = Some(event_queue);
        app
    }

    pub fn open_image(&mut self, image: RgbaImage) {
        let image_width = image.width();
        let image_height = image.height();
        let window_config = crate::window::WindowConfiguration::new(
            LogicalSize::new(image_width, image_height),
            Some(PhysicalSize::new(image_width, image_height)),
        );
        let mut window = AppWindow::new(
            self,
            window_config,
            Box::new(move |input, egui_ctx, window_ctx| {

                // 将图像数据上传到 GPU 并获取纹理句柄
                let texture_handle: &BackgroundTexture =
                    window_ctx.get_global_or_insert_with(|| {
                        // 创建 ColorImage
                        // 注意：RgbaImage 的 bytes 应该是连续的 RGBA 数据
                        let background_image = Arc::new(ColorImage::from_rgba_premultiplied(
                            [image_width as usize, image_height as usize],
                            image.as_raw(),
                        ));
                        // Load the texture only once.
                        let texture_handle = egui_ctx.load_texture(
                            "background-image",
                            egui::ImageData::Color(background_image),
                            Default::default(),
                        );
                        BackgroundTexture(texture_handle)
                    });

                // 构建 UI 的具体内容
                egui_ctx.run(input, |ctx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::new())
                        .show(ctx, |ui| {
                            ui.add(
                                Image::new(ImageSource::Texture(SizedTexture::from_handle(
                                    &texture_handle.0,
                                )))
                                .shrink_to_fit(),
                            );

                            // ui.text_edit_multiline()

                            let mut shapes = Vec::new();

                            // 添加多个形状
                            shapes.push(egui::Shape::rect_filled(
                                egui::Rect::from_min_size(
                                    egui::pos2(10.0, 10.0),
                                    egui::vec2(100.0, 50.0),
                                ),
                                0.0,
                                egui::Color32::BLUE,
                            ));

                            shapes.push(egui::Shape::line_segment(
                                [egui::pos2(150.0, 30.0), egui::pos2(300.0, 80.0)],
                                egui::Stroke::new(2.0, egui::Color32::RED),
                            ));

                            // 一次性绘制所有形状
                            ui.painter().extend(shapes);
                        });
                })
            }),
        );

        let position_calculator = Arc::new(
            |parent_surface_size: &PhysicalSize<u32>, subview_size: &PhysicalSize<u32>| {
                let subview_width = &subview_size.width;
                PhysicalPosition::new(
                    parent_surface_size.width - subview_width,
                    parent_surface_size.height + 10,
                )
            },
        );

        // 创建工具条
        window.create_sub_surface_view(
            self,
            LogicalSize::new(600, 38),
            LogicalPosition::new(image_width as i32 - 600, (image_height + 10) as i32),
            Box::new(|input, egui_ctx, annotator_ctx| {
                // 构建 UI 的具体内容
                egui_ctx.run(input, |ctx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::new())
                        .show(ctx, |ui| {
                            if ui.button("Line").clicked() {
                                println!("点击了直线工具 ");
                            }
                        });
                })
            }),
            Some(position_calculator),
        );

        self.windows.push(window);
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
            if w.contains_surface(surface) {
                w.set_scale_factor(scale_factor, self.global_state.gpu.as_mut().unwrap());
            }
            if old_scale_factor_is_none && w.first_configure == false {
                // 为了能正确绘制，窗口需要等待首次配置完成并且获取到了缩放倍数，再开始首次绘制

                // 如果窗口设置了preferred_size，那么根据这个尺寸调整窗口大小
                if let Some(preferred_size) = w.preferred_size {
                    let new_size = preferred_size.to_logical(scale_factor);
                    w.resize(new_size, &mut self.global_state.gpu.as_mut().unwrap());
                }
                w.draw(
                    &self.global_state.queue_handle,
                    &mut self.global_state.gpu.as_mut().unwrap(),
                );
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
delegate_pointer!(Application);

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
        qh: &QueueHandle<Self>,
        surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        let gpu = &mut self.global_state.gpu;
        for window in &mut self.windows {
            // 只在主 Surface (main_view) 的帧回调到达时触发重绘
            // 这样可以保证渲染频率与显示刷新率同步，避免过度提交
            if window.main_view.surface() == surface {
                window.draw(qh, gpu.as_mut().unwrap());
                return; // 找到对应的窗口并重绘后退出
            }
        }
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
        // Not needed for this example.
        info!("Surface entered");
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
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
        qh: &QueueHandle<Self>,
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
                    let gpu = self.global_state.gpu.as_mut().unwrap();
                    app_window.draw(qh, gpu);
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
        }

        if capability == Capability::Pointer && self.global_state.pointer.is_none() {
            println!("Set pointer capability");
            let pointer = self
                .global_state
                .seat_state
                .get_pointer(qh, &seat)
                .expect("Failed to create pointer");
            self.global_state.pointer = Some(pointer);
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

        if capability == Capability::Pointer && self.global_state.pointer.is_some() {
            println!("Unset pointer capability");
            self.global_state.pointer.take().unwrap().release();
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
            println!("Keyboard focus on window with pressed syms: {keysyms:?}");
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
            println!("Release keyboard focus on window");
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
        println!("Key press: {event:?}");
        for window in &mut self.windows {
            if window.keyboard_focus() {
                window.handle_keyboard_event(event.clone(), true);
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
        println!("Key repeat: {event:?}");
        for window in &mut self.windows {
            if window.keyboard_focus() {
                window.handle_keyboard_event(event.clone(), true);
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
        println!("Key release: {event:?}");
        for window in &mut self.windows {
            if window.keyboard_focus() {
                window.handle_keyboard_event(event.clone(), false);
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
        println!("Update modifiers: {modifiers:?}");
        for window in &mut self.windows {
            window.update_modifiers(modifiers);
        }
    }
}

impl PointerHandler for Application {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _pointer: &WlPointer,
        events: &[PointerEvent],
    ) {
        use PointerEventKind::*;
        for event in events {
            // Ignore events for other window
            let mut target_window_idx = None;
            for (idx, w) in self.windows.iter().enumerate() {
                if w.contains_surface(&event.surface) {
                    target_window_idx = Some(idx);
                    break;
                }
            }

            if let Some(idx) = target_window_idx {
                self.windows[idx].handle_pointer_event(event, &self.global_state);

                match event.kind {
                    Enter { .. } => {
                        println!("Pointer entered @{:?}", event.position);
                    }
                    Leave { .. } => {
                        println!("Pointer left");
                    }
                    Motion { time: _ } => {
                        // println!("Pointer motion {:?}, position: {:?}", time, event.position);
                    }
                    Press { button, .. } => {
                        println!("Press {:x} @ {:?}", button, event.position);
                    }
                    Release { button, .. } => {
                        println!("Release {:x} @ {:?}", button, event.position);
                    }
                    Axis {
                        horizontal,
                        vertical,
                        ..
                    } => {
                        println!("Scroll H:{horizontal:?}, V:{vertical:?}");
                    }
                }
            }
        }
    }
}
