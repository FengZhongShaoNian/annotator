use anyhow::Context;
use egui_wgpu::wgpu;
use egui_wgpu::wgpu::Instance;

/// GpuContext 封装了 wgpu 相关的内容
pub struct GpuContext {
    pub instance: Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GpuContext {
    pub fn new(instance: Instance, compatible_surface: &wgpu::Surface) -> anyhow::Result<Self> {
        // Pick a supported adapter
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            compatible_surface: Some(compatible_surface),
            ..Default::default()
        }))
        .context("Failed to find suitable adapter")?;

        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default()))
            .context("Failed to request device")?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
        })
    }
}
