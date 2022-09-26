use std::mem;
use std::ops::Range;

use cgmath::{Vector3, Vector4, Matrix3, Matrix4, Quaternion};
use bytemuck;

use crate::model::{Vertex, VertexLayout};
use crate::random::Randf32;

pub enum SpawnRange {
    Point,
    Area([Range<f32>; 3]),
}
impl Default for SpawnRange {
    fn default() -> Self {
        Self::Point
    }
}

pub struct ParticleSystemBounds {
    pub spawn_range: SpawnRange,
    pub init_vel:    [Range<f32>; 3],
    pub color:       [Range<f32>; 4],
    pub life:        Range<f32>,
    pub weight:      Range<f32>,
    pub scale:       Range<f32>,
    pub rand:        Randf32,
}
impl Default for ParticleSystemBounds {
    fn default() -> Self {
        Self {
            spawn_range: SpawnRange::default(),
            life:        1.0..10.0,
            init_vel:    [-0.2..0.2, 0.5..1.0, -0.2..0.2],
            color:       [0.0..1.0, 0.0..1.0, 0.0..1.0, 0.0..1.0],
            weight:      0.1..1.0,
            scale:       0.005..0.010,
            rand:        Randf32::new(),
        }
    }
}
impl ParticleSystemBounds {
    pub fn random_initial_velocity(&mut self) -> Vector3<f32> {
        let v1 = self.rand.in_range(&self.init_vel[0]);
        let v2 = self.rand.in_range(&self.init_vel[1]);
        let v3 = self.rand.in_range(&self.init_vel[2]);
        Vector3::new(v1, v2, v3)
    }
    pub fn random_color(&mut self) -> Vector4<f32> {
        let r = self.rand.in_range(&self.color[0]);
        let g = self.rand.in_range(&self.color[1]);
        let b = self.rand.in_range(&self.color[2]);
        let a = self.rand.in_range(&self.color[3]);
        Vector4::new(a, r, g, b)
    }
    pub fn random_life(&mut self) -> f32 {
        self.rand.in_range(&self.life)
    }
    pub fn random_weight(&mut self) -> f32 {
        self.rand.in_range(&self.weight)
    }
    pub fn random_scale(&mut self) -> f32 {
        self.rand.in_range(&self.scale)
    }
    pub fn random_spawn_range(&mut self) -> Vector3<f32> {
        match &self.spawn_range {
            SpawnRange::Area(dim) => {
                Vector3::new(
                    self.rand.in_range(&dim[0]),
                    self.rand.in_range(&dim[1]),
                    self.rand.in_range(&dim[2]),
                )
            }
            SpawnRange::Point => {
                Vector3::new(0.0, 0.0, 0.0)
            }
        }
    }
}

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
            pos: Vector3::new(0.0, -100.0, 0.0), 
            rot: Quaternion::new(0.0, 0.0, 0.0, 0.0), 
            vel: Vector3::new(0.0, 0.0, 0.0), 
            scale: 0.008,
            life: 0.0, 
            weight: 1.0,
            color: Vector4::new(0.0, 0.0, 0.0, 0.0),
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

#[derive(Clone, Debug)]
pub struct ParticleMesh {
    pub vertices:  Vec<Vertex>,
    pub indices:   Vec<u16>,
}
impl ParticleMesh {
    pub fn cube() -> Self {
        let vertices = vec![
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
        ];
        let indices = vec![
            0u16, 1,  2,  2,  3,  0, // top
            4,    5,  6,  6,  7,  4, // bottom
            8,    9, 10, 10, 11,  8, // right
            12,  13, 14, 14, 15, 12, // left
            16,  17, 18, 18, 19, 16, // front
            20,  21, 22, 22, 23, 20, // back
        ];
        Self {
            vertices, indices,
        }
    }
}
impl Default for ParticleMesh {
    fn default() -> Self {
        Self::cube()
    }
}

