use crate::{Color, Point, Rect, Size};
use glam::Mat4;
use std::fmt::Debug;
use std::mem;
use zerocopy::AsBytes;

#[derive(Debug)]
pub struct Pipeline {
    pipeline: wgpu::RenderPipeline,
    uniforms_bind_group: wgpu::BindGroup,
    uniforms_buffer: wgpu::Buffer,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instances_buffer: wgpu::Buffer,

    instances: Vec<Instance>,
}

impl Pipeline {
    pub fn new(
        device: &wgpu::Device,
        texture_format: wgpu::TextureFormat,
    ) -> Self {
        use wgpu::util::DeviceExt;

        let uniforms_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("goldenrod::quad uniforms layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: wgpu::BufferSize::new(
                            mem::size_of::<Uniforms>() as u64,
                        ),
                    },
                    count: None,
                }],
            });

        let uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("goldenrod::quad uniforms buffer"),
            size: mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let uniforms_bind_group =
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("goldenrod::quad uniforms bind group"),
                layout: &uniforms_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(
                        uniforms_buffer.slice(..),
                    ),
                }],
            });

        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("goldenrod::quad pipeline layout"),
                push_constant_ranges: &[],
                bind_group_layouts: &[&uniforms_layout],
            });

        let vs_module = device.create_shader_module(wgpu::include_spirv!(
            "./shader/quad.vert.spv"
        ));

        let fs_module = device.create_shader_module(wgpu::include_spirv!(
            "./shader/quad.frag.spv"
        ));

        let pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("goldenrod::quad pipeline"),
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
                label: Some("goldenrod::quad vertex buffer"),
                contents: QUAD_VERTICES.as_bytes(),
                usage: wgpu::BufferUsage::VERTEX,
            });

        let index_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("goldenrod::quad index buffer"),
                contents: QUAD_INDICES.as_bytes(),
                usage: wgpu::BufferUsage::INDEX,
            });

        let instances_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("goldenrod::quad instance buffer"),
            size: mem::size_of::<Instance>() as u64 * Instance::MAX as u64,
            usage: wgpu::BufferUsage::VERTEX | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            uniforms_bind_group,
            uniforms_buffer,
            vertex_buffer,
            index_buffer,
            instances_buffer,
            instances: Vec::with_capacity(Instance::MAX),
        }
    }

    pub fn render(
        &mut self,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        encoder: &mut wgpu::CommandEncoder,
        projection: &Mat4,
        bounds: Rect,
        target: &wgpu::TextureView,
    ) {
        if self.instances.len() == 0 {
            return;
        }

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
                    projection: projection.to_cols_array(),
                }
                .as_bytes(),
            );
        }

        let mut i = 0;
        let total = self.instances.len();
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
                .copy_from_slice(self.instances[i..i + amount].as_bytes());

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
            render_pass.set_bind_group(0, &self.uniforms_bind_group, &[]);
            render_pass.set_index_buffer(self.index_buffer.slice(..));
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, self.instances_buffer.slice(..));

            render_pass.set_scissor_rect(
                bounds.top_left.x as u32,
                bounds.top_left.y as u32,
                bounds.size.width() as u32,
                // TODO: Address anti-aliasing adjustments properly
                bounds.size.height() as u32 + 1,
            );

            render_pass.draw_indexed(
                0..QUAD_INDICES.len() as u32,
                0,
                0..amount as u32,
            );

            i += Instance::MAX;
        }

        self.instances.clear();
    }

    pub fn add_instance(
        &mut self,
        position: Point,
        size: Size,
        color: &Color,
        border_color: &Color,
        border_radius: f32,
        border_width: f32,
    ) {
        self.instances.push(Instance {
            _position: position.into(),
            _size: size.into(),
            _color: (*color).into(),
            _border_color: (*border_color).into(),
            _border_radius: border_radius,
            _border_width: border_width,
        })
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
    _color: [f32; 4],
    _border_color: [f32; 4],
    _border_radius: f32,
    _border_width: f32,
}

impl Instance {
    pub const MAX: usize = 1_000;

    fn desc<'a>() -> wgpu::VertexBufferDescriptor<'a> {
        wgpu::VertexBufferDescriptor {
            stride: mem::size_of::<Instance>() as u64,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: &[
                // _position: [f32; 2],
                wgpu::VertexAttributeDescriptor {
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float2,
                    offset: 0,
                },
                // _size: [f32; 2],
                wgpu::VertexAttributeDescriptor {
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float2,
                    offset: (std::mem::size_of::<[f32; 2]>() * 1)
                        as wgpu::BufferAddress,
                },
                // _color: [f32; 4],
                wgpu::VertexAttributeDescriptor {
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float4,
                    offset: (std::mem::size_of::<[f32; 2]>() * 2)
                        as wgpu::BufferAddress,
                },
                // _border_color: [f32; 4],
                wgpu::VertexAttributeDescriptor {
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float4,
                    offset: ((std::mem::size_of::<[f32; 2]>() * 2)
                        + (std::mem::size_of::<[f32; 4]>() * 1))
                        as wgpu::BufferAddress,
                },
                // _border_radius: f32,
                wgpu::VertexAttributeDescriptor {
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float,
                    offset: ((std::mem::size_of::<[f32; 2]>() * 2)
                        + (std::mem::size_of::<[f32; 4]>() * 2))
                        as wgpu::BufferAddress,
                },
                // _border_width f32,
                wgpu::VertexAttributeDescriptor {
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float,
                    offset: ((std::mem::size_of::<[f32; 2]>() * 2)
                        + (std::mem::size_of::<[f32; 4]>() * 2)
                        + (std::mem::size_of::<f32>() * 1))
                        as wgpu::BufferAddress,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, AsBytes)]
struct Uniforms {
    projection: [f32; 16],
}
