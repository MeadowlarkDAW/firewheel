use crate::{texture, Background, IdGroup, Size, Viewport};
use futures::task::SpawnExt;
use raw_window_handle::HasRawWindowHandle;

mod background;
mod text_pipeline;
mod texture_pipeline;

use background::BackgroundRenderer;

pub use texture_pipeline::atlas;

pub(crate) struct Renderer<TexID: IdGroup> {
    pub texture_pipeline: texture_pipeline::Pipeline<TexID>,
    pub text_pipeline: text_pipeline::Pipeline,

    background_renderer: BackgroundRenderer<TexID>,

    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    viewport: Viewport,
    staging_belt: wgpu::util::StagingBelt,
    local_pool: futures::executor::LocalPool,
}

impl<TexID: IdGroup> Renderer<TexID> {
    const CHUNK_SIZE: u64 = 10 * 1024;

    // Creating some of the wgpu types requires async code
    pub async fn new(
        window: &impl HasRawWindowHandle,
        physical_size: Size<u16>,
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
            width: u32::from(viewport.physical_size().width),
            height: u32::from(viewport.physical_size().height),
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let staging_belt = wgpu::util::StagingBelt::new(Self::CHUNK_SIZE);
        let local_pool = futures::executor::LocalPool::new();

        let texture_pipeline =
            texture_pipeline::Pipeline::new(&device, sc_desc.format);

        let text_pipeline =
            text_pipeline::Pipeline::new(&device, sc_desc.format, None);

        Some(Self {
            surface,
            device,
            queue,
            sc_desc,
            swap_chain,
            viewport,
            staging_belt,
            local_pool,

            background_renderer: BackgroundRenderer::new(),

            texture_pipeline,
            text_pipeline,
        })
    }

    pub fn set_background(&mut self, background: Background<TexID>) {
        self.background_renderer.set_background(background);
    }

    pub fn resize(&mut self, new_physical_size: Size<u16>, scale_factor: f64) {
        if self.viewport.physical_size() == new_physical_size
            && self.viewport.scale_factor() == scale_factor
        {
            return;
        }

        self.viewport =
            Viewport::from_physical_size(new_physical_size, scale_factor);

        self.sc_desc.width = u32::from(new_physical_size.width);
        self.sc_desc.height = u32::from(new_physical_size.height);
        self.swap_chain =
            self.device.create_swap_chain(&self.surface, &self.sc_desc);

        self.background_renderer.queue_full_redraw();
    }

    pub fn render(&mut self) {
        // Only render when something has changed.
        if !self.background_renderer.changed() {
            return;
        }

        let frame = match self.swap_chain.get_current_frame() {
            Ok(frame) => frame.output,
            Err(_) => {
                // Missed frame. Try again next frame.
                return;
            }
        };

        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("goldenrod: render encoder"),
            },
        );

        self.background_renderer.render(
            &mut self.texture_pipeline,
            &mut encoder,
            &frame.view,
        );

        self.texture_pipeline.render(
            &self.device,
            &mut self.staging_belt,
            &mut encoder,
            self.viewport.projection_scale(),
            self.viewport.bounds(),
            &frame.view,
        );

        self.text_pipeline.render(
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

    pub fn replace_texture_atlas(
        &mut self,
        textures: &[(TexID, texture::Texture)],
    ) -> Result<(), texture_pipeline::atlas::AtlasError> {
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("goldenrod: texture loader encoder"),
            },
        );

        self.texture_pipeline.replace_texture_atlas(
            textures,
            self.viewport.is_hi_dpi(),
            &self.device,
            &mut encoder,
        )?;

        self.queue.submit(Some(encoder.finish()));

        self.background_renderer.queue_full_redraw();

        Ok(())
    }

    pub fn needs_full_redraw(&self) -> bool {
        self.background_renderer.needs_full_redraw()
    }
}
