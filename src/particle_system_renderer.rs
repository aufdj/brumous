use wgpu::util::DeviceExt;

use crate::error::{BrumousError, BrumousResult};
use crate::texture::Texture;
use crate::particle::{
    ParticleVertex, 
    ParticleInstance, 
    VertexLayout,
    ParticleMesh
};
use crate::ParticleSystemRendererDescriptor;
use crate::matrix::Mat4x4;
use crate::vector::Vec4;
use crate::ParticleMeshType;

const SHADER: &str = include_str!("particle.wgsl");


pub struct ParticleSystemRenderer {
    pub pipeline:    wgpu::RenderPipeline,
    pub bind_groups: Vec<wgpu::BindGroup>,
    pub mesh:        ParticleMesh,
    pub view_data:   wgpu::Buffer,
    pub lights:      wgpu::Buffer,
    pub max_lights:  u64,
}
impl ParticleSystemRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        desc: &ParticleSystemRendererDescriptor,
    ) -> BrumousResult<Self> {
        let texture = Texture::new(device, queue, desc.texture)?;
        let fs_entry = if desc.texture.is_some() {
            "fs_texture"
        }
        else {
            "fs_main"
        };

        let mesh = ParticleMesh::new(device, &desc.mesh_type)?;

        let view_data = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("View Data Buffer"),
                contents: bytemuck::cast_slice(&[ViewData::default()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let lights = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Lights Buffer"),
                contents: bytemuck::cast_slice(&vec![Light::default(); desc.max_lights]),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
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
            ),
            &device.create_bind_group_layout(
                &wgpu::BindGroupLayoutDescriptor {
                    label: Some("Lights Bind Group Layout"),
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage {
                                    read_only: true,
                                },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ]
                }
            )
        ];

        let bind_groups = vec![
            device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("View Data Bind Group"),
                    layout: bind_layouts[0],
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
                    layout: bind_layouts[1],
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
            ),
            device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    label: Some("Lights Bind Group"),
                    layout: bind_layouts[2],
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: lights.as_entire_binding(),
                        },
                    ],
                }
            )
        ];

        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(SHADER.into()),
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
                    entry_point: fs_entry,
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format: config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::from(&desc.mesh_type),
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
                mesh,
                view_data,
                lights,
                max_lights: desc.max_lights as u64,
            }
        )
    }

    pub fn add_light(
        &mut self, 
        queue: &wgpu::Queue, 
        position: [f32; 4], 
        color: [f32; 4], 
        idx: u64
    ) -> BrumousResult<()> {
        if idx >= self.max_lights {
            return Err(
                BrumousError::InvalidLightIndex(idx, self.max_lights)
            );
        }
        queue.write_buffer(
            &self.lights, 
            idx * Light::size(), 
            bytemuck::cast_slice(&[Light::new(position, color)])
        );
        Ok(())
    }

    pub fn set_view_proj(&mut self, queue: &wgpu::Queue, vp: [[f32; 4]; 4]) {
        queue.write_buffer(&self.view_data, 0, bytemuck::cast_slice(&[vp]));
    }

    pub fn set_view_pos(&mut self, queue: &wgpu::Queue, vp: [f32; 3]) {
        let vp = [vp[0], vp[1], vp[2], 0.0];
        queue.write_buffer(&self.view_data, 64, bytemuck::cast_slice(&[vp]));
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    position: [f32; 4],
    color: [f32; 4],
}
impl Light {
    pub fn new(position: [f32; 4], color: [f32; 4]) -> Self {
        Self {
            position,
            color,
        }
    }
    pub fn size() -> u64 {
        std::mem::size_of::<Self>() as u64
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct ViewData {
    view_proj: [[f32; 4]; 4],
    view_pos: [f32; 4],
}
impl Default for ViewData {
    fn default() -> Self {
        Self {
            view_proj: Mat4x4::identity().into(),
            view_pos: Vec4::unit_y().into(),
        }
    }
}

impl From<&ParticleMeshType<'_>> for wgpu::PrimitiveTopology {
    fn from(mesh_type: &ParticleMeshType) -> Self {
        match mesh_type {
            ParticleMeshType::Point => {
                Self::PointList
            }
            ParticleMeshType::Cube => {
                Self::TriangleList
            }
            _ => {
                Self::TriangleList
            }
        }
    }
}
