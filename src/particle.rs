use std::mem;

use crate::{ParticleSystemBounds, ParticleMeshType};
use crate::obj::read_obj_file;
use crate::error::BrumousResult;
use crate::vector::{Vec3, Vec4};
use crate::matrix::{Mat3x3, Mat4x4};
use crate::quaternion::Quaternion;
use crate::random::Randf32;
use crate::particle_system::ParticleAttractor;

const G: f32 = 0.00000000006674;


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

#[derive(Clone, Copy)]
pub struct Particle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Quaternion,
    pub scale:    f32,
    pub life:     f32,
    pub mass:     f32,
    pub color:    Vec4,
    pub queued:   bool,
}
impl Particle {
    pub fn new(rand: &mut Randf32, bounds: &ParticleSystemBounds, pos: &Vec3) -> Self {
        Self {
            position: rand.vec3_in(&bounds.area) + *pos,
            velocity: rand.vec3_in(&bounds.velocity),
            rotation: rand.quat_in(&bounds.rotation),
            color:    rand.vec4_in(&bounds.color),
            scale:    rand.f32_in(&bounds.scale),
            life:     rand.f32_in(&bounds.life),
            mass:     rand.f32_in(&bounds.mass),
            queued:   false,
        }
    }
    pub fn update(&mut self, delta: f32, atts: &[ParticleAttractor], forces: &[Vec3]) {
        for att in atts.iter() {
            let pa = att.pos - self.position;
            let dist = pa.len();
            let force = (G * att.mass * self.mass) / (dist * dist);
            let acc = force / self.mass;
            let grav_dir = pa.normalized();
            self.velocity += grav_dir * (acc * delta);
        }
        
        let acc = forces.iter().sum::<Vec3>() / self.mass;
        self.velocity += acc * delta * 0.5;

        self.position += self.velocity * delta;
    }
    pub fn instance(&self) -> ParticleInstance {
        ParticleInstance {
            model: (
                Mat4x4::from_translation(self.position) *
                Mat4x4::from(self.rotation) *
                Mat4x4::from_scale(self.scale)
            ).into(),
            normal: Mat3x3::from(self.rotation).into(),
            color: self.color.into(),
        }
    }
}
impl Default for Particle {
    fn default() -> Self {
        Self {
            position: Vec3::zero(),
            velocity: Vec3::zero(),
            rotation: Quaternion::zero(),
            scale:    0.0,
            life:     0.0,
            mass:     0.0,
            color:    Vec4::zero(),
            queued:   true,
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
    pub fn empty() -> Self {
        Self {
            model: [[0.0; 4]; 4],
            normal: [[0.0; 3]; 3],
            color: [0.0; 4],
        }
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
        read_obj_file(device, mesh_type)
    }
}

