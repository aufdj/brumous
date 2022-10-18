use std::path::{Path, PathBuf};
use std::io::Read;

use wgpu::util::DeviceExt;

use crate::error::BrumousResult;
use crate::bufio::new_input_file;
use crate::texture::Texture;
use crate::particle::{
    ParticleVertex, 
    ParticleInstance, 
    VertexLayout,
    ParticleMesh
};

use crate::ParticleSystemRendererDescriptor;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ViewData {
    view_proj: [[f32; 4]; 4],
    view_pos: [f32; 4],
}
impl ViewData {
    fn new() -> Self {
        Self {
            view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
            ],
            view_pos: [0.0, 1.0, 0.0, 0.0],
        }
        
    }
}

pub struct ParticleSystemRenderer {
    pub pipeline:    wgpu::RenderPipeline,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub view_data:   wgpu::Buffer,
    pub mesh:        ParticleMesh,
}
impl ParticleSystemRenderer {
    pub fn new(
        device: &wgpu::Device, 
        queue:  &wgpu::Queue, 
        config: &wgpu::SurfaceConfiguration,
        desc:   &ParticleSystemRendererDescriptor,
    ) -> BrumousResult<Self> {
        let texture = if let Some(tex) = desc.texture {
            Texture::new(device, queue, Path::new(tex))?
        }
        else {
            Texture::new(device, queue, Path::new("image/default.png"))?
        };

        let mesh = ParticleMesh::new(device, &desc.mesh_type)?;

        let view_data = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("View Projection Buffer"),
                contents: bytemuck::cast_slice(&[ViewData::new()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_layouts = [
            &device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("View Data Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        }
                    ]
                }
            ),
            &device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("Texture Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { 
                                    filterable: true 
                                },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(
                                wgpu::SamplerBindingType::Filtering
                            ),
                            count: None,
                        }
                    ]
                }
            )
        ];

        let bind_groups = vec![
            device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Camera Bind Group"),
                    layout: &bind_layouts[0],
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: view_data.as_entire_binding(),
                        }
                    ]
                }
            ),
            device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Texture Bind Group"),
                    layout: &bind_layouts[1],
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&texture.sampler),
                        },
                    ],
                }
            )
        ];

        let mut shader_str = String::new();

        new_input_file(Path::new("src/particle.wgsl"))?.read_to_string(&mut shader_str)?;

        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(shader_str.into()),
            }
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: bind_layouts.as_slice(),
                push_constant_ranges: &[],
            }
        );

        let pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        ParticleVertex::layout(),
                        ParticleInstance::layout(),
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_texture",
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            }
        );

        Ok(
            Self {
                pipeline,
                bind_groups,
                view_data,
                mesh,
            }
        )
    }
}

