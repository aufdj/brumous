pub mod particle;
pub mod texture;
pub mod camera;
pub mod model;
pub mod random;
pub mod vec;
pub mod gpu;
pub mod delta;
pub mod bufio;

use std::num::NonZeroU64;
use std::path::Path;
use std::ops::Range;
use std::io::Read;

use crate::particle::*;
use crate::texture::Texture;
use crate::gpu::Gpu;
use crate::bufio::new_input_file;

use wgpu::util::DeviceExt;
use cgmath::{Vector3, Quaternion};

pub struct ParticleSystem {
    pub particles:      Vec<Particle>,
    pub mesh:           ParticleMesh,
    pub vbuf:           wgpu::Buffer,
    pub ibuf:           wgpu::Buffer,
    pub particle_buf:   wgpu::Buffer,
    last_used_particle: usize,
    particle_rate:      usize,
    position:           Vector3<f32>,
    texture:            Option<Texture>,
    name:               String,
    life:               f32,
    gravity:            f32,
    bounds:             ParticleSystemBounds,
    pipeline:           wgpu::RenderPipeline,
}
impl ParticleSystem {
    pub fn new(device: &wgpu::Device, desc: ParticleSystemDescriptor, pipeline_desc: &wgpu::RenderPipelineDescriptor) -> Self {
        let particles = vec![Particle::default(); desc.count];

        let vbuf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Vertex Buffer"),
                contents: bytemuck::cast_slice(&desc.mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let ibuf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Index Buffer"),
                contents: bytemuck::cast_slice(&desc.mesh.indices),
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

        let pipeline = device.create_render_pipeline(pipeline_desc);

        Self {
            particles,
            vbuf,
            ibuf,
            particle_buf,
            last_used_particle: 0,
            mesh: desc.mesh,
            particle_rate: desc.rate,
            position: desc.pos,
            texture: None,
            name: desc.name,
            life: desc.life,
            gravity: desc.gravity,
            bounds: desc.bounds,
            pipeline,
        }
    }
    fn find_unused_particle(&mut self) -> usize {
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
    pub fn new_particle(&mut self) -> Particle {
        Particle {
            pos:    self.position + self.bounds.random_spawn_range(),
            rot:    Quaternion::new(0.0, 0.0, 0.0, 0.0), 
            vel:    self.bounds.random_initial_velocity(), 
            scale:  self.bounds.random_scale(),
            life:   self.bounds.random_life(), 
            weight: self.bounds.random_weight(),
            color:  self.bounds.random_color(),
        }
    }
    pub fn update(&mut self, delta: f32, queue: &wgpu::Queue) {
        self.life -= delta;
        if self.life > 0.0 {
            for _ in 0..self.particle_rate {
                let idx = self.find_unused_particle();
                self.particles[idx] = self.new_particle();
            }
        }

        for (index, particle) in self.particles.iter_mut().enumerate() {
            particle.life -= delta;
            if particle.life > 0.0 {
                particle.update(delta, self.gravity);
                queue.write_buffer(
                    &self.particle_buf,
                    index as u64 * ParticleRaw::size(),
                    bytemuck::cast_slice(&[particle.to_raw()])
                );
            }
        }
        if self.last_used_particle > self.particles.len() {
            self.last_used_particle = self.particles.len() - 1;
        }
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
    pub fn particle_count(&self) -> u32 {
        self.particles.len() as u32
    }
    pub fn particle_buf_size(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.particles.len() as u64 * ParticleRaw::size())
    }
    pub fn set_position(&mut self, pos: [f32; 3]) {
        self.position = Vector3::new(pos[0], pos[1], pos[2]);
    }
    pub fn set_texture(&mut self, gpu: &Gpu, texture_path: &Path) {
        let mut diffuse_data = Vec::new();
        new_input_file(&texture_path).unwrap().read_to_end(&mut diffuse_data).unwrap();
        let texture = Texture::new(&gpu.device, &gpu.queue, &diffuse_data, None).unwrap();
        self.texture = Some(texture);
    }
    pub fn set_particle_rate(&mut self, particle_rate: usize) {
        self.particle_rate = particle_rate;
    }
    pub fn set_gravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn set_weight(&mut self, weight: Range<f32>) {
        self.bounds.weight = weight;
    }
    pub fn set_initial_velocity(&mut self, init_vel: [Range<f32>; 3]) {
        self.bounds.init_vel = init_vel;
    }
    pub fn set_spawn_range(&mut self, spawn_range: SpawnRange) {
        self.bounds.spawn_range = spawn_range;
    }
    pub fn set_life(&mut self, life: Range<f32>) {
        self.bounds.life = life;
    }
    pub fn clear(&mut self, encoder: &mut wgpu::CommandEncoder) {
        encoder.clear_buffer(&self.particle_buf, 0, self.particle_buf_size());
    }
}

pub struct ParticleSystemDescriptor {
    mesh:     ParticleMesh,
    count:    usize,
    rate:     usize,
    pos:      Vector3<f32>,
    name:     String,
    life:     f32,
    gravity:  f32,
    bounds:   ParticleSystemBounds,
}
impl Default for ParticleSystemDescriptor {
    fn default() -> Self {
        Self {
            mesh: ParticleMesh::default(),
            count: 500,
            rate: 3,
            pos: Vector3::new(0.0, 0.0, 0.0),
            name: String::from("Particle System"),
            life: 5.0,
            gravity: -9.81,
            bounds: ParticleSystemBounds::default(),
        }
    }
}


pub trait DrawParticleSystem<'a, 'b> where 'a: 'b {
    fn draw_particle_system(
        &'b mut self, 
        sys: &'a ParticleSystem, 
        bind_groups: &[&'a wgpu::BindGroup]
    );
}
impl<'a, 'b> DrawParticleSystem<'a, 'b> for wgpu::RenderPass<'a> where 'a: 'b {
    fn draw_particle_system(
        &'b mut self, 
        sys: &'a ParticleSystem, 
        bind_groups: &[&'a wgpu::BindGroup]
    ) {
        self.set_pipeline(&sys.pipeline);

        for (i, group) in bind_groups.iter().enumerate() {
            self.set_bind_group(i as u32, group, &[]);
        }
        
        self.set_vertex_buffer(0, sys.vbuf.slice(..));
        self.set_vertex_buffer(1, sys.particle_buf.slice(..));

        self.set_index_buffer(sys.ibuf.slice(..), wgpu::IndexFormat::Uint16);
        
        self.draw_indexed(0..sys.mesh.indices.len() as u32, 0, 0..sys.particle_count());
    }
}

pub trait CreateParticleSystem {
    fn create_particle_system(
        &self, 
        desc: ParticleSystemDescriptor, 
        pipeline: &wgpu::RenderPipelineDescriptor
    ) -> ParticleSystem;
}
impl CreateParticleSystem for wgpu::Device {
    fn create_particle_system(
        &self, 
        desc: ParticleSystemDescriptor, 
        pipeline: &wgpu::RenderPipelineDescriptor
    ) -> ParticleSystem {
        ParticleSystem::new(self, desc, pipeline)
    }
}