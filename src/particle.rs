use std::mem;

use cgmath::{Vector3, Vector4, Matrix3, Matrix4, Quaternion};
use bytemuck;
use wgpu::util::DeviceExt;

use crate::ParticleMeshType;

pub trait ToRaw {
    fn to_raw(&self) -> Vec<ParticleModel>;
}
impl ToRaw for Vec<Particle> {
    fn to_raw(&self) -> Vec<ParticleModel> {
        self.iter().map(Particle::to_raw).collect::<Vec<ParticleModel>>()
    }
}

pub trait VertexLayout {
    fn layout() -> wgpu::VertexBufferLayout<'static>;
}

#[derive(Clone)]
pub struct Particle {
    pub pos:   Vector3<f32>,
    pub vel:   Vector3<f32>,
    pub rot:   Quaternion<f32>,
    pub scale: f32,
    pub life:  f32,
    pub mass:  f32,
    pub color: Vector4<f32>,
}
impl Particle {
    pub fn update(&mut self, delta: f32, gravity: f32) {
        self.vel += Vector3::new(0.0, gravity * self.mass, 0.0) * delta * 0.5;
        self.pos += self.vel * delta;
    }
    pub fn to_raw(&self) -> ParticleModel {
        ParticleModel {
            model: (
                Matrix4::from_translation(self.pos) *
                Matrix4::from(self.rot) *
                Matrix4::from_scale(self.scale)
            ).into(),
            normal: Matrix3::from(self.rot).into(),
            color: self.color.into(),
        }
    }
}
impl Default for Particle {
    fn default() -> Self {
        Self {
            pos:   Vector3::new(0.0, -100.0, 0.0), 
            rot:   Quaternion::new(0.0, 0.0, 0.0, 0.0), 
            vel:   Vector3::new(0.0, 0.0, 0.0), 
            scale: 0.0,
            life:  0.0, 
            mass:  0.0,
            color: Vector4::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleModel {
    model:  [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
    color:  [f32; 4],
}
impl ParticleModel {
    pub fn size() -> u64 {
        mem::size_of::<Self>() as u64
    }
    const ATTRIBUTES: [wgpu::VertexAttribute; 8] = wgpu::vertex_attr_array![
        5 => Float32x4,  6 => Float32x4,  7 => Float32x4, 8 => Float32x4,
        9 => Float32x3, 10 => Float32x3, 11 => Float32x3,
       12 => Float32x4
    ];
}
impl VertexLayout for ParticleModel {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}
impl ParticleVertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
        0 => Float32x3, 1 => Float32x2, 2 => Float32x3
    ];
}
impl VertexLayout for ParticleVertex {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}


pub struct ParticleMesh {
    pub vertex_buf:   wgpu::Buffer,
    pub index_buf:    Option<wgpu::Buffer>,
    pub vertex_count: u32,
    pub index_count:  u32,
}
impl ParticleMesh {
    pub fn new(device: &wgpu::Device, mesh_type: &ParticleMeshType) -> Self {
        match mesh_type {
            ParticleMeshType::Custom(path) => {
                let vertices: Vec<ParticleVertex> = vec![];
                let indices: Vec<u16> = vec![];

                let vertex_count = vertices.len() as u32;
                let index_count = indices.len() as u32;

                let vertex_buf = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Particle Vertex Buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    }
                );
    
                let index_buf = Some(device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Particle Index Buffer"),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    }
                ));
    
                Self {
                    vertex_buf, index_buf, vertex_count, index_count
                }
            },
            ParticleMeshType::CubeIndexed => {
                let vertices = vec![
                    // top (0, 0, 1)
                    ParticleVertex { position: [-1.0, -1.0,  1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [ 1.0, -1.0,  1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [ 1.0,  1.0,  1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [-1.0,  1.0,  1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    // bottom (0, 0, -1)
                    ParticleVertex { position: [-1.0,  1.0, -1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [ 1.0,  1.0, -1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [ 1.0, -1.0, -1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [-1.0, -1.0, -1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    // right (1, 0, 0)
                    ParticleVertex { position: [ 1.0, -1.0, -1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [ 1.0,  1.0, -1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [ 1.0,  1.0,  1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [ 1.0, -1.0,  1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    // left (-1, 0, 0)
                    ParticleVertex { position: [-1.0, -1.0,  1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [-1.0,  1.0,  1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [-1.0,  1.0, -1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [-1.0, -1.0, -1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    // front (0, 1, 0)
                    ParticleVertex { position: [ 1.0,  1.0, -1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [-1.0,  1.0, -1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [-1.0,  1.0,  1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [ 1.0,  1.0,  1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    // back (0, -1, 0)
                    ParticleVertex { position: [ 1.0, -1.0,  1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [-1.0, -1.0,  1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [-1.0, -1.0, -1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    ParticleVertex { position: [ 1.0, -1.0, -1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
                ];
                let indices = vec![
                    0u16, 1,  2,  2,  3,  0, // top
                    4,    5,  6,  6,  7,  4, // bottom
                    8,    9, 10, 10, 11,  8, // right
                    12,  13, 14, 14, 15, 12, // left
                    16,  17, 18, 18, 19, 16, // front
                    20,  21, 22, 22, 23, 20, // back
                ];

                let vertex_count = 0u32;
                let index_count = indices.len() as u32;

                let vertex_buf = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Particle Vertex Buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    }
                );
    
                let index_buf = Some(device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Particle Index Buffer"),
                        contents: bytemuck::cast_slice(&indices),
                        usage: wgpu::BufferUsages::INDEX,
                    }
                ));
    
                Self {
                    vertex_buf, index_buf, vertex_count, index_count
                }
            },
            ParticleMeshType::Cube => {
                let vertices: Vec<ParticleVertex> = vec![];

                let vertex_count = vertices.len() as u32;
                let index_count = 0u32;

                let vertex_buf = device.create_buffer_init(
                    &wgpu::util::BufferInitDescriptor {
                        label: Some("Particle Vertex Buffer"),
                        contents: bytemuck::cast_slice(&vertices),
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    }
                );
    
                let index_buf = None;
    
                Self {
                    vertex_buf, index_buf, vertex_count, index_count
                }
            }
        }
    }
}

