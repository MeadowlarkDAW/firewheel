use crate::{Rectangle, TextureHandle};
use std::fmt::Debug;
use std::mem;
use zerocopy::AsBytes;

pub mod atlas;

const ATLAS_SCALE: [f32; 2] = [1.0 / atlas::ATLAS_SIZE as f32, 1.0 / atlas::ATLAS_SIZE as f32];

pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    uniforms_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instances_buffer: wgpu::Buffer,
    constants_bind_group: wgpu::BindGroup,
    texture_bind_group: wgpu::BindGroup,
    texture_layout: wgpu::BindGroupLayout,
    texture_atlas: atlas::Atlas,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        use wgpu::util::DeviceExt;

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let constants_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("goldenrod::texture constants layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: wgpu::BufferSize::new(
                                mem::size_of::<Uniforms>() as u64,
                            ),
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler { comparison: false },
                        count: None,
                    },
                ],
            });

        let uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("goldenrod::texture uniforms buffer"),
            size: mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let constants_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("goldenrod::texture constants bind group"),
                layout: &constants_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            uniforms_buffer.slice(..),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

        let texture_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("goldenrod::texture texture layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        dimension: wgpu::TextureViewDimension::D2,
                        component_type: wgpu::TextureComponentType::Float,
                        multisampled: false,
                    },
                    count: None,
                }],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("goldenrod::texture pipeline layout"),
                push_constant_ranges: &[],
                bind_group_layouts: &[&constants_layout, &texture_layout],
            });

        let vs_module = device.create_shader_module(wgpu::include_spirv!(
            "shader/image.vert.spv"
        ));

        let fs_module = device.create_shader_module(wgpu::include_spirv!(
            "shader/image.frag.spv"
        ));

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("goldenrod::texture pipeline"),
                layout: Some(&pipeline_layout),
                vertex_stage: wgpu::ProgrammableStageDescriptor {
                    module: &vs_module,
                    entry_point: "main",
                },
                fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                    module: &fs_module,
                    entry_point: "main",
                }),
                rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: wgpu::CullMode::None,
                    ..Default::default()
                }),
                primitive_topology: wgpu::PrimitiveTopology::TriangleList,
                color_states: &[wgpu::ColorStateDescriptor {
                    format: texture_format,
                    color_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::SrcAlpha,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha_blend: wgpu::BlendDescriptor {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    write_mask: wgpu::ColorWrite::ALL,
                }],
                depth_stencil_state: None,
                vertex_state: wgpu::VertexStateDescriptor {
                    index_format: wgpu::IndexFormat::Uint16,
                    vertex_buffers: &[Vertex::desc(), Instance::desc()],
                },
                sample_count: 1,
                sample_mask: !0,
                alpha_to_coverage_enabled: false,
            });

        let vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("goldenrod::texture vertex buffer"),
                contents: QUAD_VERTICES.as_bytes(),
                usage: wgpu::BufferUsage::VERTEX,
            });

        let index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("goldenrod::texture index buffer"),
                contents: QUAD_INDICES.as_bytes(),
                usage: wgpu::BufferUsage::INDEX,
            });

        let instances_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("goldenrod::texture instance buffer"),
            size: mem::size_of::<Instance>() as u64 * Instance::MAX as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let texture_atlas = atlas::Atlas::new(device);

        let texture_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("goldenrod::texture texture atlas bind group"),
                layout: &texture_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(
                        &texture_atlas.view(),
                    ),
                }],
            });

        Self {
            pipeline,
            uniforms_buffer,
            vertex_buffer,
            index_buffer,
            instances_buffer,
            constants_bind_group,
            texture_bind_group,
            texture_layout,
            texture_atlas,
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        projection_scale: [f32; 2],
        bounds: Rectangle,
        target: &wgpu::TextureView,
    ) {
        // Update uniforms buffer
        {
            let mut uniforms_buffer = staging_belt.write_buffer(
                encoder,
                &self.uniforms_buffer,
                0, // offset
                wgpu::BufferSize::new(mem::size_of::<Uniforms>() as u64)
                    .unwrap(),
                device,
            );

            uniforms_buffer.copy_from_slice(
                Uniforms {
                    scale: projection_scale,
                    atlas_scale: ATLAS_SCALE,
                }
                .as_bytes(),
            );
        }

        let instances = INSTANCES.clone();

        let mut i = 0;
        let total = instances.len();
        while i < total {
            let end = (i + Instance::MAX).min(total);
            let amount = end - i;

            let mut instances_buffer = staging_belt.write_buffer(
                encoder,
                &self.instances_buffer,
                0,
                wgpu::BufferSize::new(
                    (amount * std::mem::size_of::<Instance>()) as u64,
                )
                .unwrap(),
                device,
            );

            instances_buffer
                .copy_from_slice(instances[i..i + amount].as_bytes());

            let mut render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[
                        wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: target,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: true,
                            },
                        },
                    ],
                    depth_stencil_attachment: None,
                });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.constants_bind_group, &[]);
            render_pass.set_bind_group(1, &self.texture_bind_group, &[]);
            render_pass.set_index_buffer(self.index_buffer.slice(..));
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instances_buffer.slice(..));

            render_pass.set_scissor_rect(
                bounds.x.round() as u32,
                bounds.y.round() as u32,
                bounds.width.round() as u32,
                bounds.height.round() as u32,
            );

            render_pass.draw_indexed(
                0..QUAD_INDICES.len() as u32,
                0,
                0..amount as u32,
            );

            i += Instance::MAX;
        }
    }

    pub fn load_texture_handles<T: Into<TextureHandle> + Copy + Clone>(
        &mut self,
        textures: &[T],
        hi_dpi: bool,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<(), atlas::AtlasError> {
        self.texture_atlas
            .load_texture_handles(device, textures, encoder, hi_dpi)
    }
}

#[repr(C)]
#[derive(Clone, Copy, AsBytes)]
struct Vertex {
    _position: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: &[wgpu::VertexAttributeDescriptor {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float2,
            }],
        }
    }
}

const QUAD_INDICES: [u16; 6] = [0, 1, 2, 0, 2, 3];

const QUAD_VERTICES: [Vertex; 4] = [
    Vertex {
        _position: [0.0, 0.0],
    },
    Vertex {
        _position: [1.0, 0.0],
    },
    Vertex {
        _position: [1.0, 1.0],
    },
    Vertex {
        _position: [0.0, 1.0],
    },
];

#[repr(C)]
#[derive(Debug, Clone, Copy, AsBytes)]
struct Instance {
    _position: [f32; 2],
    _size: [f32; 2],
    _atlas_position: [f32; 2],
    _atlas_size: [f32; 2],
    _atlas_layer: u32,
}

impl Instance {
    pub const MAX: usize = 1_000;

    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Instance>() as u64,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                wgpu::VertexAttributeDescriptor {
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                    offset: 0,
                },
                wgpu::VertexAttributeDescriptor {
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float2,
                    offset: std::mem::size_of::<[f32; 2]>()
                        as wgpu::BufferAddress,
                },
                wgpu::VertexAttributeDescriptor {
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float2,
                    offset: (std::mem::size_of::<[f32; 2]>() * 2)
                        as wgpu::BufferAddress,
                },
                wgpu::VertexAttributeDescriptor {
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float2,
                    offset: (std::mem::size_of::<[f32; 2]>() * 3)
                        as wgpu::BufferAddress,
                },
                wgpu::VertexAttributeDescriptor {
                    shader_location: 5,
                    format: wgpu::VertexFormat::Uint,
                    offset: (std::mem::size_of::<[f32; 2]>() * 4)
                        as wgpu::BufferAddress,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, AsBytes)]
struct Uniforms {
    scale: [f32; 2],
    atlas_scale: [f32; 2],
}

/// For testing purposes
const INSTANCES: [Instance; 1] = [Instance {
    _position: [100.0, 100.0],
    _size: [200.0, 200.0],
    _atlas_position: [0.0, 0.0],
    _atlas_size: [256.0, 256.0],
    _atlas_layer: 0,
}];
