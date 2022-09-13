use std::mem;
use std::num::NonZeroU64;

use cgmath::{Vector3, Vector4, Matrix3, Matrix4, Quaternion};
use wgpu::util::DeviceExt;
use bytemuck;

use crate::model::{Vertex, VertexLayout};
use crate::random::Randf64;
use crate::config::SystemConfig;
use crate::texture::Texture;

#[derive(Default, Copy, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl From<[f32; 3]> for Position {
    fn from(pos: [f32; 3]) -> Self {
        Self {
            x: pos[0],
            y: pos[1],
            z: pos[2],
        }
    }
}
impl From<Position> for [f32; 3] {
    fn from(pos: Position) -> Self {
        [pos.x, pos.y, pos.z]
    }
}

pub struct ParticleSystem {
    pub particles: Vec<Particle>,
    pub mesh: ParticleMesh,
    pub vbuf: wgpu::Buffer,
    pub ibuf: wgpu::Buffer,
    pub particle_buf: wgpu::Buffer,
    last_used_particle: usize,
    pub particle_rate: usize,
    pub position: Position,
    rand: Randf64,
    texture: Option<Texture>,
    // pipeline: wgpu::RenderPipeline,
    pub name: String,
}
impl ParticleSystem {
    pub fn new(device: &wgpu::Device, scfg: &SystemConfig) -> Self {
        let particles = vec![Particle::default(); scfg.max];

        let vbuf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Vertex Buffer"),
                contents: bytemuck::cast_slice(&scfg.mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let ibuf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Vertex Buffer"),
                contents: bytemuck::cast_slice(&scfg.mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let particle_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Buffer"),
                contents: bytemuck::cast_slice(&particles.to_raw()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        Self {
            particles,
            mesh: scfg.mesh.clone(),
            vbuf,
            ibuf,
            particle_buf,
            last_used_particle: 0,
            particle_rate: 10,
            position: Position { x: scfg.pos.0, y: scfg.pos.1, z: 0.0 },
            rand: Randf64::new(),
            texture: None,
            name: String::from("Particle System"),
        }
    }
    pub fn particle_count(&self) -> u32 {
        self.particles.len() as u32
    }
    pub fn find_unused_particle(&mut self) -> usize {
        if self.last_used_particle > self.particles.len() {
            self.last_used_particle = self.particles.len() - 1;
        }
        for i in self.last_used_particle..self.particles.len() {
            if self.particles[i].life < 0.0 {
                self.last_used_particle = i;
                return i;
            }
        }

        for i in 0..self.last_used_particle {
            if self.particles[i].life < 0.0 {
                self.last_used_particle = i;
                return i;
            }
        }
        self.last_used_particle = 0;
        0
    }
    pub fn update_particles(&mut self, delta: f32, queue: &wgpu::Queue) {
        for _ in 0..self.particle_rate {
            let particle = self.find_unused_particle();
            self.particles[particle].respawn(&mut self.rand, self.position);
        }

        for (index, particle) in self.particles.iter_mut().enumerate() {
            particle.life -= delta;
            if particle.life > 0.0 {
                particle.update(delta);
                queue.write_buffer(
                    &self.particle_buf, 
                    index as u64 * ParticleRaw::size(), 
                    bytemuck::cast_slice(&[particle.to_raw()])
                );
            }
        }
    }
    pub fn position(&mut self, pos: [f32; 3]) {
        self.position = Position::from(pos);
    }
    pub fn texture(&mut self, texture: Texture) {
        self.texture = Some(texture);
    }
    pub fn particle_rate(&mut self, particle_rate: i32) {
        self.particle_rate = particle_rate as usize;
    }
    pub fn particle_buf_size(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.particles.len() as u64 * ParticleRaw::size())
    }
    pub fn resize(&mut self, count: i32, device: &wgpu::Device) {
        self.particles.resize(count as usize, Particle::default());
        self.particle_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Buffer"),
                contents: bytemuck::cast_slice(&self.particles.to_raw()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
    }
    pub fn default(device: &wgpu::Device) -> Self {
        let particles = vec![Particle::default(); 5000];
        let mesh = ParticleMesh::cube();

        let vbuf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let ibuf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let particle_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&particles.to_raw()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        Self {
            particles,
            mesh,
            vbuf,
            ibuf,
            particle_buf,
            last_used_particle: 0,
            particle_rate: 10,
            position: Position::default(),
            rand: Randf64::new(),
            texture: None,
            name: String::from("Particle System"),
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
    pub fn update(&mut self, delta: f32) {
        self.vel += Vector3::new(0.0, -5.00, 0.0) * delta * 0.5;
        self.pos += self.vel * delta;
    }
    pub fn respawn(&mut self, rand: &mut Randf64, pos: Position) {
        let p1 = rand.in_range(-0.002, 0.002) as f32;
        let p2 = rand.in_range(-0.002, 0.002) as f32;
        self.pos = Vector3::new(p1 + pos.x, p2 + pos.y, 0.0 + pos.z);

        let v1 = rand.in_range(-0.2, 0.2) as f32;
        let v2 = rand.in_range(-0.2, 0.2) as f32;
        self.vel = Vector3::new(v1, 1.0, v2);

        let r = rand.in_range(0.0, 1.0) as f32;
        let g = rand.in_range(0.0, 1.0) as f32;
        let b = rand.in_range(0.0, 1.0) as f32;
        self.color = Vector4::new(1.0, r, g, b);
        self.life = 1.0;
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
            scale: 0.001,
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