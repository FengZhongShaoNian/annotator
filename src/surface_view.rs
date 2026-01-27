use log::info;
use crate::application::{Application, Globals};
use crate::dpi::{LogicalSize, PhysicalSize};
use crate::egui_input::EguiInput;
use crate::egui_skia_painter::EguiSkiaPainter;
use crate::gpu::GpuSurface;
use crate::view::View;
use sctk::seat::keyboard::{KeyEvent, Modifiers};
use sctk::seat::pointer::PointerEvent;
use wayland_client::protocol::wl_surface::WlSurface;
use wayland_client::QueueHandle;
use wayland_protocols::wp::viewporter::client::wp_viewport::WpViewport;
use crate::context::{AnnotatorContext, Output};

pub struct SurfaceView {
    /// Wayland Surface 句柄
    surface: WlSurface,
    /// 逻辑尺寸
    size: LogicalSize<u32>,
    /// 当前缩放倍数
    scale_factor: f64,
    /// 视口（用于分数缩放适配）
    viewport: WpViewport,
    /// GPU 渲染表面
    pub gpu_surface: Option<GpuSurface>,
    /// Egui 上下文
    egui_ctx: egui::Context,
    /// Egui 输入状态管理
    egui_input: EguiInput,
    /// Egui Skia 渲染器
    egui_painter: EguiSkiaPainter,
    /// 用于构建UI的函数
    build_view: Box<dyn Fn(egui::RawInput, &mut egui::Context, &mut AnnotatorContext) -> Output>,
    annotator_context: AnnotatorContext
}

impl SurfaceView {
    pub fn new(
        surface: WlSurface,
        size: LogicalSize<u32>,
        viewport: WpViewport,
        build_view: Box<dyn Fn(egui::RawInput, &mut egui::Context, &mut AnnotatorContext) -> Output>,
    ) -> Self {
        viewport.set_destination(size.width as i32, size.height as i32);

        // 初始化 Egui 环境
        let egui_ctx = egui::Context::default();
        let egui_input = EguiInput::new();
        let egui_painter = EguiSkiaPainter::new();

        Self {
            surface,
            size,
            scale_factor: 1.,
            viewport,
            gpu_surface: None,
            egui_ctx,
            egui_input,
            egui_painter,
            build_view,
            annotator_context: Default::default(),
        }
    }

    pub fn handle_keyboard_event(&mut self, event: sctk::seat::keyboard::KeyEvent, pressed: bool) {
        self.egui_input.handle_keyboard_event(event, pressed);
    }

    pub fn update_modifiers(&mut self, modifiers: sctk::seat::keyboard::Modifiers) {
        self.egui_input.update_modifiers(modifiers);
    }

    pub fn handle_pointer_event(&mut self, event: &sctk::seat::pointer::PointerEvent, _globals: &crate::application::Globals) {
        self.egui_input.handle_pointer_event(event, self.scale_factor);
    }

    pub fn set_gpu_surface(&mut self, gpu_surface: GpuSurface) {
        self.gpu_surface = Some(gpu_surface);
    }

    fn create_gpu_surface(&mut self, gpu: &mut crate::gpu::GpuContext, size: PhysicalSize<u32>) -> anyhow::Result<GpuSurface> {
        // 为主视图创建 GPU 渲染表面
        crate::gpu::GpuSurface::new(
            gpu,
            &self.surface,
            size.width as i32,
            size.height as i32,
        )
    }

    /// 获取关联的 Wayland Surface。
    pub fn surface(&self) -> &WlSurface {
        &self.surface
    }

    /// 获取物理像素尺寸。
    pub fn physical_size(&self) -> PhysicalSize<u32> {
        self.size.to_physical(self.scale_factor)
    }
}

impl View for SurfaceView {
    fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    fn set_scale_factor(&mut self, scale_factor: f64) {
        self.scale_factor = scale_factor;
    }

    fn size(&self) -> LogicalSize<u32> {
        self.size
    }

    fn viewport(&self) -> &WpViewport {
        &self.viewport
    }

    fn surface(&self) -> &WlSurface {
        self.surface()
    }

    fn handle_keyboard_event(&mut self, event: KeyEvent, pressed: bool) {
        self.egui_input.handle_keyboard_event(event, pressed);
    }

    fn update_modifiers(&mut self, modifiers: Modifiers) {
        self.egui_input.update_modifiers(modifiers);
    }

    fn handle_pointer_event(&mut self, event: &PointerEvent, globals: &Globals) {
        self.egui_input.handle_pointer_event(event, self.scale_factor);
    }

    /// 使用 GPU 渲染视图内容
    fn draw(&mut self, queue_handle: &QueueHandle<Application>, gpu: &mut crate::gpu::GpuContext) {

        // 准备 Egui 输入
        let mut raw_input = self.egui_input.raw.take();
        let scale_factor = self.scale_factor;
                
        // 设置正确的像素每点比例
        self.egui_ctx.set_pixels_per_point(scale_factor as f32);
                
        // screen_rect 是逻辑尺寸
        let screen_rect = egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(self.size.width as f32, self.size.height as f32)
        );
        raw_input.screen_rect = Some(screen_rect);
                
        let build_ui_fn = self.build_view.as_mut();
        let Output{ egui_output , preferred_size} =  build_ui_fn(raw_input, &mut self.egui_ctx, &mut self.annotator_context);

        // 重要：为下一帧准备新的 RawInput
        // egui 在渲染后需要重置输入状态，保留持久性信息
        self.egui_input.raw = egui::RawInput::default();

        let physical_size = self.physical_size();
        // let physical_size = if let Some(size) = preferred_size {
        //     size.to_physical(scale_factor)
        // }else {
        //     self.physical_size()
        // };

        info!("physical_size: {:?}", physical_size);
        // self.resize(physical_size.to_logical(scale_factor));
                
        if self.gpu_surface.is_none() {
            let surface = self.create_gpu_surface(gpu, physical_size).unwrap();
            self.gpu_surface = Some(surface);
        }
                
        if let Some(gpu_surface) = &mut self.gpu_surface {
            // 1. 同步 GPU 表面尺寸
            gpu_surface
                .resize(gpu, physical_size.width as i32, physical_size.height as i32)
                .ok();
        
            // 2. 切换上下文
            gpu_surface.make_current(gpu).ok();
        
            let canvas = gpu_surface.skia_surface.canvas();
            canvas.clear(skia_safe::Color::from_rgb(34, 34, 38));
        
            // 3. 准备渲染数据
            // 使用正确的像素每点比例进行镶嵌
            let pixels_per_point = self.egui_ctx.pixels_per_point();
                    
            // 保存shapes的副本以供失败时回退使用
            let shapes_backup = egui_output.shapes.clone();
            let primitives = self.egui_ctx.tessellate(shapes_backup, pixels_per_point);

            // 4. 执行绘图
            self.egui_painter.paint(
                canvas,
                primitives,
                egui_output.textures_delta,
                pixels_per_point,
            );
        
            // 5. 提交并交换
            gpu_surface.swap_buffers(gpu).ok();
        }
        
        // 请求frame回调以确保持续渲染
        self.surface.frame(&queue_handle, self.surface.clone());
        
        // 提交surface以触发frame回调
        self.surface.commit();
    }
}
