use std::mem;
use std::path::Path;

use bytemuck;

use crate::ParticleMeshType;
use crate::obj::{read_obj_str, read_obj_file};
use crate::error::BrumousResult;
use crate::vector::{Vec3, Vec4};
use crate::matrix::{Mat3x3, Mat4x4};
use crate::quaternion::Quaternion;

const CUBE_OBJ: &'static str = include_str!("../obj/cube.obj");


pub trait Instance {
    fn instance(&self) -> Vec<ParticleInstance>;
}
impl Instance for Vec<Particle> {
    fn instance(&self) -> Vec<ParticleInstance> {
        self.iter().map(Particle::instance).collect::<Vec<ParticleInstance>>()
    }
}

pub trait VertexLayout {
    fn layout() -> wgpu::VertexBufferLayout<'static>;
}

#[derive(Clone)]
pub struct Particle {
    pub pos:   Vec3,
    pub vel:   Vec3,
    pub rot:   Quaternion,
    pub scale: f32,
    pub life:  f32,
    pub mass:  f32,
    pub color: Vec4,
}
impl Particle {
    pub fn update(&mut self, delta: f32, gravity: Vec3, forces: &[Vec3]) {
        let acc = (/*forces.iter().sum::<Vec3>() + */gravity) / self.mass;
        self.vel += acc * delta * 0.5;
        self.pos += self.vel * delta;
    }
    pub fn instance(&self) -> ParticleInstance {
        ParticleInstance {
            model: (
                Mat4x4::from_translation(self.pos) *
                Mat4x4::from(self.rot) *
                Mat4x4::from_scale(self.scale)
            ).into(),
            normal: Mat3x3::from(self.rot).into(),
            color: self.color.into(),
        }
    }
}
impl Default for Particle {
    fn default() -> Self {
        Self {
            pos:   Vec3::new(0.0, -100.0, 0.0),
            rot:   Quaternion::new(0.0, 0.0, 0.0, 0.0), 
            vel:   Vec3::zero(), 
            scale: 0.0,
            life:  0.0, 
            mass:  0.0,
            color: Vec4::zero(),
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
                read_obj_file(device, path)
            },
            ParticleMeshType::Cube => {
                Ok(read_obj_str(device, CUBE_OBJ))
            }
        }
    }
}

