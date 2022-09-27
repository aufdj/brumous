use std::mem;
use std::path::PathBuf;

use cgmath::{Vector3, Vector4, Matrix3, Matrix4, Quaternion};
use bytemuck;
use wgpu::util::DeviceExt;

use crate::model::{Vertex, VertexLayout};

#[derive(Clone)]
pub struct Particle {
    pub pos:    Vector3<f32>,
    pub vel:    Vector3<f32>,
    pub rot:    Quaternion<f32>,
    pub scale:  f32,
    pub life:   f32,
    pub weight: f32,
    pub color:  Vector4<f32>,
}
impl Particle {
    pub fn update(&mut self, delta: f32, gravity: f32) {
        self.vel += Vector3::new(0.0, gravity * self.weight, 0.0) * delta * 0.5;
        self.pos += self.vel * delta;
    }
    pub fn to_raw(&self) -> ParticleRaw {
        ParticleRaw {
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
            pos:    Vector3::new(0.0, -100.0, 0.0), 
            rot:    Quaternion::new(0.0, 0.0, 0.0, 0.0), 
            vel:    Vector3::new(0.0, 0.0, 0.0), 
            scale:  0.008,
            life:   0.0, 
            weight: 1.0,
            color:  Vector4::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleRaw {
    model:  [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
    color:  [f32; 4],
}
impl ParticleRaw {
    pub fn size() -> u64 {
        mem::size_of::<Self>() as u64
    }
}
impl VertexLayout for ParticleRaw {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ParticleRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 26]>() as wgpu::BufferAddress,
                    shader_location: 12,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub trait ToRaw {
    fn to_raw(&self) -> Vec<ParticleRaw>;
}
impl ToRaw for Vec<Particle> {
    fn to_raw(&self) -> Vec<ParticleRaw> {
        self.iter().map(Particle::to_raw).collect::<Vec<ParticleRaw>>()
    }
}


#[derive(Default)]
pub enum ParticleMeshType {
    #[default]
    Cube,
    Custom(PathBuf),
}

pub struct ParticleMesh {
    pub index_count: u32,
    pub vertex_buf:  wgpu::Buffer,
    pub index_buf:   wgpu::Buffer,
}
impl ParticleMesh {
    pub fn new(device: &wgpu::Device, mesh_type: &ParticleMeshType) -> Self {
        let (vertices, indices) = match mesh_type {
            ParticleMeshType::Custom(path) => {
                (vec![], vec![])
            }
            ParticleMeshType::Cube => {
                (vec![
                    // top (0, 0, 1)
                    Vertex { position: [-1.0, -1.0,  1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [ 1.0, -1.0,  1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [ 1.0,  1.0,  1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [-1.0,  1.0,  1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    // bottom (0, 0, -1)
                    Vertex { position: [-1.0,  1.0, -1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [ 1.0,  1.0, -1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [ 1.0, -1.0, -1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    // right (1, 0, 0)
                    Vertex { position: [ 1.0, -1.0, -1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [ 1.0,  1.0, -1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [ 1.0,  1.0,  1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [ 1.0, -1.0,  1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    // left (-1, 0, 0)
                    Vertex { position: [-1.0, -1.0,  1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [-1.0,  1.0,  1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [-1.0,  1.0, -1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    // front (0, 1, 0)
                    Vertex { position: [ 1.0,  1.0, -1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [-1.0,  1.0, -1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [-1.0,  1.0,  1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [ 1.0,  1.0,  1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    // back (0, -1, 0)
                    Vertex { position: [ 1.0, -1.0,  1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [-1.0, -1.0,  1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
                    Vertex { position: [ 1.0, -1.0, -1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
                ],
                vec![
                    0u16, 1,  2,  2,  3,  0, // top
                    4,    5,  6,  6,  7,  4, // bottom
                    8,    9, 10, 10, 11,  8, // right
                    12,  13, 14, 14, 15, 12, // left
                    16,  17, 18, 18, 19, 16, // front
                    20,  21, 22, 22, 23, 20, // back
                ])
            }
        };

        let index_count = indices.len() as u32;

        let vertex_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
    
        let index_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
    
        Self {
            index_count, vertex_buf, index_buf, 
        }
    }
}

