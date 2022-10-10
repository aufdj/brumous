use std::mem;
use std::path::Path;

use cgmath::{Vector3, Vector4, Matrix3, Matrix4, Quaternion};
use bytemuck;

use crate::particle_system::ParticleMeshType;
use crate::obj::read_obj;
use crate::error::BrumousResult;


pub trait ToRaw {
    fn to_raw(&self) -> Vec<ParticleInstance>;
}
impl ToRaw for Vec<Particle> {
    fn to_raw(&self) -> Vec<ParticleInstance> {
        self.iter().map(Particle::to_raw).collect::<Vec<ParticleInstance>>()
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
    pub fn to_raw(&self) -> ParticleInstance {
        ParticleInstance {
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
pub struct ParticleInstance {
    model:  [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
    color:  [f32; 4],
}
impl ParticleInstance {
    pub fn size() -> u64 {
        mem::size_of::<Self>() as u64
    }
    const ATTRIBUTES: [wgpu::VertexAttribute; 8] = wgpu::vertex_attr_array![
        5 => Float32x4,  6 => Float32x4,  7 => Float32x4, 8 => Float32x4,
        9 => Float32x3, 10 => Float32x3, 11 => Float32x3,
       12 => Float32x4
    ];
}
impl VertexLayout for ParticleInstance {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
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
    pub fn new(device: &wgpu::Device, mesh_type: &ParticleMeshType) -> BrumousResult<Self> {
        match mesh_type {
            ParticleMeshType::Custom(path) => {
                read_obj(device, &path)
            },
            ParticleMeshType::Cube => {
                read_obj(device, Path::new("obj/cube.obj"))
            }
        }
    }
}

