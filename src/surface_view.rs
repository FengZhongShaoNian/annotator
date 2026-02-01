use crate::application::{Application, GlobalState};
use crate::context::WindowContext;
use crate::dpi::{LogicalSize, PhysicalSize};
use crate::egui_input::EguiInput;
use crate::gpu::GpuContext;
use crate::view::View;
use egui::{FullOutput, ImeEvent, PlatformOutput, RawInput};
use egui_wgpu::wgpu::TextureFormat;
use egui_wgpu::{RendererOptions, wgpu};
use log::info;
use sctk::seat::keyboard::{KeyEvent, Modifiers};
use sctk::seat::pointer::PointerEvent;
use wayland_client::QueueHandle;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_protocols::wp::viewporter::client::wp_viewport::WpViewport;
use crate::font::setup_chinese_fonts;

pub struct SurfaceView<'window> {
    /// Wayland Surface 句柄
    surface: WlSurface,
    /// WGPU 渲染表面
    pub wgpu_surface: wgpu::Surface<'window>,
    pub wgpu_surface_configuration: wgpu::SurfaceConfiguration,
    /// 逻辑尺寸
    size: LogicalSize<u32>,
    /// 当前缩放倍数
    scale_factor: f64,
    /// 视口（用于分数缩放适配）
    viewport: WpViewport,
    /// Egui 上下文
    egui_ctx: egui::Context,
    /// Egui 输入状态管理
    egui_input: EguiInput,
    /// Egui Skia 渲染器
    egui_renderer: Option<egui_wgpu::Renderer>,
    /// 用于构建UI的函数
    build_view: Box<dyn Fn(RawInput, &mut egui::Context, &mut WindowContext) -> FullOutput>,
}

impl<'window> SurfaceView<'window> {
    pub fn new(
        surface: WlSurface,
        wgpu_surface: wgpu::Surface<'window>,
        wgpu_surface_configuration: wgpu::SurfaceConfiguration,
        size: LogicalSize<u32>,
        viewport: WpViewport,
        build_view: Box<
            dyn Fn(egui::RawInput, &mut egui::Context, &mut WindowContext) -> FullOutput,
        >,
    ) -> Self {
        viewport.set_destination(size.width as i32, size.height as i32);

        // 初始化 Egui 环境
        let egui_ctx = egui::Context::default();
        setup_chinese_fonts(&egui_ctx);

        let egui_input = EguiInput::new();

        Self {
            surface,
            wgpu_surface,
            wgpu_surface_configuration,
            size,
            scale_factor: 1.,
            viewport,
            egui_ctx,
            egui_input,
            egui_renderer: None,
            build_view,
        }
    }

    

    /// 获取关联的 Wayland Surface。
    pub fn surface(&self) -> &WlSurface {
        &self.surface
    }
}

impl<'window> View for SurfaceView<'window> {
    fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    fn set_scale_factor(&mut self, scale_factor: f64, gpu: &mut GpuContext) {
        self.scale_factor = scale_factor;

        let physical_size = self.size.to_physical(scale_factor);
        self.resize_surface(physical_size, &gpu);
    }

    fn size(&self) -> LogicalSize<u32> {
        self.size
    }

    fn viewport_size(&self) -> PhysicalSize<u32> {
        self.size.to_physical(self.scale_factor)
    }

    fn viewport(&self) -> &WpViewport {
        &self.viewport
    }

    fn resize(&mut self, new_size: LogicalSize<u32>, gpu: &mut GpuContext) {
        info!("Resize viewport to: {:?}", new_size);

        let physical_size = new_size.to_physical(self.scale_factor());
        self.resize_surface(physical_size, &gpu);

        self.size = new_size;
        self.viewport()
            .set_destination(new_size.width as i32, new_size.height as i32);
    }

    fn surface(&self) -> &WlSurface {
        self.surface()
    }

    fn handle_keyboard_event(&mut self, event: KeyEvent, pressed: bool, repeat: bool) {
        self.egui_input.handle_keyboard_event(event, pressed, repeat);
    }

    fn update_modifiers(&mut self, modifiers: Modifiers) {
        self.egui_input.update_modifiers(modifiers);
    }

    fn handle_ime_event(&mut self, event: &ImeEvent) {
        self.egui_input.handle_ime_event(event);
    }

    fn handle_pointer_event(
        &mut self,
        event: &PointerEvent,
        _globals: &GlobalState,
    ) {
        self.egui_input
            .handle_pointer_event(event);
    }

    /// 使用 GPU 渲染视图内容
    fn draw(&mut self, global_state: &GlobalState, window_context: &mut WindowContext) -> Option<PlatformOutput> {
        let egui_output = self.run_egui(window_context);

        // 获取当前帧纹理
        let Ok(frame) = self.wgpu_surface.get_current_texture() else {
            // 跳过这一帧
            return None;
        };

        // 创建纹理视图
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let gpu = global_state.gpu.as_ref().unwrap();

        // 创建命令编码器
        let device = &gpu.device;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let egui_renderer = if let Some(ref mut renderer) = self.egui_renderer {
            renderer
        } else {
            let renderer_option = RendererOptions::default();
            let renderer =
                egui_wgpu::Renderer::new(device, TextureFormat::Bgra8UnormSrgb, renderer_option);
            self.egui_renderer = Some(renderer);
            self.egui_renderer.as_mut().unwrap()
        };

        let physical_size = self.size.to_physical(self.scale_factor);
        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [physical_size.width, physical_size.height],
            pixels_per_point: egui_output.pixels_per_point,
        };

        // 更新纹理
        // 将给定形状镶嵌成三角形网格
        let paint_jobs = self
            .egui_ctx
            .tessellate(egui_output.shapes, egui_output.pixels_per_point);
        for (id, image_delta) in &egui_output.textures_delta.set {
            egui_renderer.update_texture(&device, &gpu.queue, *id, image_delta);
        }

        // 更新EGUI顶点/索引缓冲区
        egui_renderer.update_buffers(
            &gpu.device,
            &gpu.queue,
            &mut encoder,
            &paint_jobs,
            &screen_descriptor,
        );
        {
            // 6. 执行渲染
            let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let mut render_pass = render_pass.forget_lifetime();

            egui_renderer.render(
                &mut render_pass,
                &paint_jobs, // 渲染同样的 shapes
                &screen_descriptor,
            );
        }

        // 提交命令到队列
        gpu.queue.submit(std::iter::once(encoder.finish()));

        // 请求frame回调以确保持续渲染
        self.surface.frame(&global_state.queue_handle, self.surface.clone());

        // 提交缓冲区到表面，然后提交
        frame.present();

        Some(egui_output.platform_output)
    }
}

impl<'window> SurfaceView<'window> {
    fn resize_surface(&mut self, physical_size: PhysicalSize<u32>, gpu: &&mut GpuContext) {
        let surface_config = &mut self.wgpu_surface_configuration;
        surface_config.width = physical_size.width;
        surface_config.height = physical_size.height;

        self.wgpu_surface.configure(&gpu.device, &surface_config);
    }
}

impl<'window> SurfaceView<'window> {
    fn run_egui(&mut self, window_context: &mut WindowContext) -> FullOutput {
        // 准备 Egui 输入
        let mut raw_input = self.egui_input.raw.take();

        // 设置逻辑像素与对应的物理像素的比例
        self.egui_ctx.set_pixels_per_point(self.scale_factor as f32);

        // screen_rect 是逻辑尺寸
        let screen_rect = egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(self.size.width as f32, self.size.height as f32),
        );
        raw_input.screen_rect = Some(screen_rect);

        let build_ui_fn = &self.build_view;
        let egui_output = build_ui_fn(raw_input, &mut self.egui_ctx, window_context);

        // 重要：为下一帧准备新的 RawInput
        // egui 在渲染后需要重置输入状态，保留持久性信息
        self.egui_input.raw = RawInput::default();

        egui_output
    }
}
