use crate::{Size, TextureHandle};
use futures::task::SpawnExt;
use raw_window_handle::HasRawWindowHandle;

mod texture;
mod viewport;

pub use viewport::Viewport;

pub struct Renderer {
    instance: wgpu::Instance,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    viewport: viewport::Viewport,
    staging_belt: wgpu::util::StagingBelt,
    local_pool: futures::executor::LocalPool,

    texture_pipeline: texture::Pipeline,
}

impl Renderer {
    const CHUNK_SIZE: u64 = 10 * 1024;

    // Creating some of the wgpu types requires async code
    pub async fn new(
        window: &impl HasRawWindowHandle,
        physical_size: Size,
        scale_factor: f64,
    ) -> Option<Self> {
        let viewport =
            Viewport::from_physical_size(physical_size, scale_factor);

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::Default,
                compatible_surface: Some(&surface),
            })
            .await?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits {
                        max_bind_groups: 2,
                        ..wgpu::Limits::default()
                    },
                    shader_validation: true,
                },
                None, // Trace path
            )
            .await
            .ok()?;

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: viewport.physical_size().width.ceil() as u32,
            height: viewport.physical_size().height.ceil() as u32,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let staging_belt = wgpu::util::StagingBelt::new(Self::CHUNK_SIZE);
        let local_pool = futures::executor::LocalPool::new();

        let texture_pipeline =
            texture::Pipeline::new(&device, &queue, sc_desc.format);

        Some(Self {
            instance,
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            viewport,
            staging_belt,
            local_pool,

            texture_pipeline,
        })
    }

    pub fn resize(&mut self, new_physical_size: Size, scale_factor: f64) {
        if self.viewport.physical_size() == new_physical_size
            && self.viewport.scale_factor() == scale_factor
        {
            return;
        }

        self.viewport =
            Viewport::from_physical_size(new_physical_size, scale_factor);

        self.sc_desc.width = new_physical_size.width.ceil() as u32;
        self.sc_desc.height = new_physical_size.height.ceil() as u32;
        self.swap_chain =
            self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn render(&mut self) {
        let frame = self
            .swap_chain
            .get_current_frame()
            .expect("Timeout getting next frame")
            .output;

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("render encoder"),
            },
        );

        let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        self.texture_pipeline.render(
            &self.device,
            &mut self.staging_belt,
            &mut encoder,
            self.viewport.projection_scale(),
            self.viewport.bounds(),
            &frame.view,
        );

        // Submit work
        self.staging_belt.finish();
        self.queue.submit(Some(encoder.finish()));

        // Recall staging buffers
        self.local_pool
            .spawner()
            .spawn(self.staging_belt.recall())
            .expect("Failed to recall staging belt");

        self.local_pool.run_until_stalled();
    }

    pub fn load_texture_handles<T: Into<TextureHandle> + Copy + Clone>(
        &mut self,
        textures: &[T],
        hi_dpi: bool,
    ) -> Result<(), texture::atlas::AtlasError> {
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("texture loader encoder"),
            },
        );

        self.texture_pipeline.load_texture_handles(
            textures,
            hi_dpi,
            &self.device,
            &mut encoder,
        )?;

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }
}
