use crate::{
    settings, texture, viewport::Viewport, Background, Color, PhySize, Point,
};
use futures::task::SpawnExt;
use raw_window_handle::HasRawWindowHandle;

mod background;
mod quad_pipeline;
mod text_pipeline;
mod texture_pipeline;
mod triangle_pipeline;

use background::BackgroundRenderer;

pub use texture_pipeline::atlas;

pub struct Renderer {
    pub texture_pipeline: texture_pipeline::Pipeline,
    pub text_pipeline: text_pipeline::Pipeline,
    pub quad_pipeline: quad_pipeline::Pipeline,
    pub triangle_pipeline: triangle_pipeline::Pipeline,

    background_renderer: BackgroundRenderer,

    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    sc_desc: wgpu::SwapChainDescriptor,
    swap_chain: wgpu::SwapChain,
    viewport: Viewport,
    staging_belt: wgpu::util::StagingBelt,
    local_pool: futures::executor::LocalPool,
}

impl Renderer {
    const CHUNK_SIZE: u64 = 10 * 1024;

    // Creating some of the wgpu types requires async code
    pub async fn new(
        window: &impl HasRawWindowHandle,
        physical_size: PhySize,
        scale_factor: f64,
        antialiasing: settings::Antialiasing,
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
            width: viewport.physical_size().width as u32,
            height: viewport.physical_size().height as u32,
            present_mode: wgpu::PresentMode::Mailbox,
        };
        let swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let staging_belt = wgpu::util::StagingBelt::new(Self::CHUNK_SIZE);
        let local_pool = futures::executor::LocalPool::new();

        let texture_pipeline =
            texture_pipeline::Pipeline::new(&device, sc_desc.format);

        let text_pipeline =
            text_pipeline::Pipeline::new(&device, sc_desc.format, None);

        let quad_pipeline =
            quad_pipeline::Pipeline::new(&device, sc_desc.format);

        let triangle_pipeline = triangle_pipeline::Pipeline::new(
            &device,
            sc_desc.format,
            Some(antialiasing),
        );

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
            quad_pipeline,
            triangle_pipeline,
        })
    }

    pub fn set_background(&mut self, background: Background) {
        self.background_renderer.set_background(background);
    }

    pub fn resize(&mut self, new_physical_size: PhySize, scale_factor: f64) {
        if self.viewport.physical_size() == new_physical_size
            && self.viewport.scale_factor() == scale_factor
        {
            return;
        }

        self.viewport =
            Viewport::from_physical_size(new_physical_size, scale_factor);

        self.sc_desc.width = new_physical_size.width as u32;
        self.sc_desc.height = new_physical_size.height as u32;
        self.swap_chain =
            self.device.create_swap_chain(&self.surface, &self.sc_desc);
    }

    pub fn render(&mut self) {
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
            self.viewport.projection(),
            self.viewport.bounds(),
            &frame.view,
        );

        self.text_pipeline.render(
            &self.device,
            &mut self.staging_belt,
            &mut encoder,
            self.viewport.bounds(),
            &frame.view,
        );

        self.quad_pipeline.render(
            &self.device,
            &mut self.staging_belt,
            &mut encoder,
            self.viewport.projection(),
            self.viewport.bounds(),
            &frame.view,
        );

        use lyon::math::{point, Point};
        use lyon::path::builder::*;
        use lyon::path::Path;
        use lyon::tessellation::*;

        // Build a Path.
        let mut builder = Path::builder();
        builder.move_to(point(0.0, 0.0));
        builder.line_to(point(100.0, 0.0));
        builder.quadratic_bezier_to(point(200.0, 0.0), point(200.0, 100.0));
        builder.cubic_bezier_to(
            point(100.0, 100.0),
            point(0.0, 100.0),
            point(0.0, 0.0),
        );
        builder.close();
        let path = builder.build();

        let mut geometry: VertexBuffers<crate::primitive::Vertex2D, u32> =
            VertexBuffers::new();
        let mut tessellator = FillTessellator::new();
        {
            // Compute the tessellation.
            tessellator
                .tessellate_path(
                    &path,
                    &FillOptions::default(),
                    &mut BuffersBuilder::new(
                        &mut geometry,
                        |pos: Point, _: FillAttributes| {
                            crate::primitive::Vertex2D {
                                position: pos.to_array(),
                                color: crate::Color::WHITE.into(),
                            }
                        },
                    ),
                )
                .unwrap();
        }

        let mesh = crate::primitive::Mesh2D {
            vertices: geometry.vertices,
            indices: geometry.indices,
        };

        let instance = triangle_pipeline::Instance {
            origin: crate::Point::new(50.0, 50.0),
            buffers: &mesh,
            clip_bounds: self.viewport.bounds(),
        };

        self.triangle_pipeline.render(
            &self.device,
            &mut self.staging_belt,
            &mut encoder,
            self.viewport.projection(),
            &frame.view,
            self.viewport.bounds().size.width() as u32,
            self.viewport.bounds().size.height() as u32,
            &[instance],
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

    pub fn new_texture_atlas(
        &mut self,
        texture_loaders: &mut [texture::Loader],
    ) -> Result<(), texture_pipeline::atlas::AtlasError> {
        let mut encoder = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("goldenrod: texture loader encoder"),
            },
        );

        self.texture_pipeline.new_texture_atlas(
            texture_loaders,
            self.viewport.is_hi_dpi(),
            &self.device,
            &mut encoder,
        )?;

        self.queue.submit(Some(encoder.finish()));

        Ok(())
    }
}
