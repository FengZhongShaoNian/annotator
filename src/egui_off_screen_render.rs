use std::mem;
use crate::dpi::LogicalSize;
use crate::gpu::GpuContext;
use egui::{pos2, vec2, FullOutput, RawInput, Rect};
use egui_wgpu::wgpu::{
    BufferDescriptor, BufferUsages, BufferView, CommandEncoderDescriptor, Extent3d, MapMode,
    Origin3d, PollType, RenderPassColorAttachment, RenderPassDescriptor, TexelCopyBufferInfo,
    TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages,
};
use egui_wgpu::RendererOptions;
use image::{ImageBuffer, Rgba};
use std::sync::oneshot;
use std::thread::spawn;

pub type BuildUI = Box<dyn FnOnce(RawInput, &mut egui::Context) -> FullOutput>;
pub fn render_egui_to_image(
    gpu_context: &GpuContext,
    virtual_screen_size: LogicalSize<u32>,
    pixels_per_point: f32,
    build_ui: BuildUI,
) /*-> ImageBuffer<Rgba<u8>, BufferView>*/ {
    let device = gpu_context.device.clone();
    let physical_size = virtual_screen_size.to_physical(pixels_per_point as f64);
    let texture_size = Extent3d {
        width: physical_size.width,
        height: physical_size.height,
        depth_or_array_layers: 1,
    };
    let texture_desc = TextureDescriptor {
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: TextureFormat::Bgra8UnormSrgb,
        usage: TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT,
        label: None,
        view_formats: &[],
    };
    let texture = device.create_texture(&texture_desc);
    let texture_view = texture.create_view(&Default::default());

    let mut egui_ctx = egui::Context::default();
    egui_ctx.set_pixels_per_point(pixels_per_point);

    let mut raw_input = RawInput::default();
    raw_input.screen_rect = Some(Rect::from_min_size(
        pos2(0., 0.),
        vec2(
            virtual_screen_size.width as f32,
            virtual_screen_size.height as f32,
        ),
    ));
    let full_output = build_ui(raw_input, &mut egui_ctx);

    // 更新纹理
    // 将给定形状镶嵌成三角形网格
    let paint_jobs = egui_ctx.tessellate(full_output.shapes, pixels_per_point); // 通常由 run 内部处理，但也可手动

    let physical_size = virtual_screen_size.to_physical(pixels_per_point as f64);
    let screen_descriptor = egui_wgpu::ScreenDescriptor {
        size_in_pixels: [physical_size.width, physical_size.height],
        pixels_per_point,
    };

    let renderer_option = RendererOptions::default();
    let mut renderer = egui_wgpu::Renderer::new(&device, texture.format(), renderer_option);

    for (id, image_delta) in &full_output.textures_delta.set {
        renderer.update_texture(&device, &gpu_context.queue, *id, image_delta);
    }

    // 创建命令编码器
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());

    let queue = gpu_context.queue.clone();
    // 更新EGUI顶点/索引缓冲区
    renderer.update_buffers(
        &device,
        &queue,
        &mut encoder,
        &paint_jobs,
        &screen_descriptor,
    );

    // 开始渲染通道，使用离屏纹理视图
    {
        let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("egui offscreen pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: egui_wgpu::wgpu::Operations {
                    load: egui_wgpu::wgpu::LoadOp::Clear(egui_wgpu::wgpu::Color::TRANSPARENT),
                    store: egui_wgpu::wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        let mut render_pass = render_pass.forget_lifetime();

        // 执行渲染
        renderer.render(
            &mut render_pass,
            &paint_jobs, // 渲染同样的 shapes
            &screen_descriptor,
        );
    }


    // 提交命令
    queue.submit(Some(encoder.finish()));

    // wgpu 需要使用 COPY_BYTES_PER_ROW_ALIGNMENT 对齐纹理 -> 缓冲区的复制
    // 因此，我们需要同时保存 padded_bytes_per_row 和 unpadded_bytes_per_row
    let pixel_size = size_of::<[u8;4]>() as u32;
    // 计算对齐后的 bytes_per_row
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let unpadded_bytes_per_row = texture_size.width * pixel_size;
    let padding = (align - unpadded_bytes_per_row % align) % align;
    let padded_bytes_per_row = unpadded_bytes_per_row + padding;
    let buffer_size = (padded_bytes_per_row * texture_size.height) as wgpu::BufferAddress;

    // 创建目标缓冲区，用于接收像素数据
    let buffer = device.create_buffer(&BufferDescriptor {
        label: Some("output buffer"),
        size: buffer_size as u64,
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor::default());
    encoder.copy_texture_to_buffer(
        TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        TexelCopyBufferInfo {
            buffer: &buffer,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded_bytes_per_row),
                rows_per_image: Some(texture_size.height),
            },
        },
        texture_size,
    );
    queue.submit(Some(encoder.finish()));

    // 需要对映射变量设置范围，以便我们能够解除缓冲区的映射
    {
        let buffer_slice = buffer.slice(..);
        let (tx, rx) = oneshot::channel();
        buffer_slice.map_async(MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        device.poll(PollType::wait_indefinitely()).unwrap();

        if let Ok(Ok(())) = rx.recv() {
            let padded_data = buffer_slice.get_mapped_range();
            let data = padded_data
                .chunks(padded_bytes_per_row as _)
                .map(|chunk| { &chunk[..unpadded_bytes_per_row as _]})
                .flatten()
                .map(|x| { *x })
                .collect::<Vec<_>>();
            let pixels_bgra = data.to_vec(); // 复制 BGRA 数据
            drop(data);

            // 转换为 RGBA
            let mut pixels_rgba = Vec::with_capacity(pixels_bgra.len());
            for chunk in pixels_bgra.chunks_exact(4) {
                pixels_rgba.extend_from_slice(&[chunk[2], chunk[1], chunk[0], chunk[3]]);
            }

            let image_buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(
                texture_size.width,
                texture_size.height,
                pixels_rgba,
            )
                .unwrap();

            image_buffer.save("/home/one/Pictures/image.png").unwrap();
        } else {
            panic!("从 GPU 读取数据失败！");
        }
    }
}
