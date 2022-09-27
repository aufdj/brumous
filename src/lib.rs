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

/// A ParticleSystem manages a set of particles
pub struct ParticleSystem {
    pipeline:         wgpu::RenderPipeline,
    pub particles:    Vec<Particle>,
    pub mesh:         ParticleMesh,
    pub particle_buf: wgpu::Buffer,
    search_pos:       usize,
    particle_rate:    usize,
    position:         Vector3<f32>,
    texture:          Option<Texture>,
    name:             String,
    life:             f32,
    gravity:          f32,
    bounds:           ParticleSystemBounds,
}
impl ParticleSystem {
    pub fn new(
        device: &wgpu::Device, 
        sys_desc: ParticleSystemDescriptor, 
        rpipe_desc: &wgpu::RenderPipelineDescriptor
    ) -> Self {
        let particles = vec![Particle::default(); sys_desc.count];
        let mesh = ParticleMesh::new(device, &sys_desc.mesh_type);

        let particle_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&particles.to_raw()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let pipeline = device.create_render_pipeline(rpipe_desc);

        Self {
            pipeline,
            particles,
            particle_buf,
            search_pos:    0,
            texture:       None,
            mesh,
            particle_rate: sys_desc.rate,
            position:      sys_desc.pos,
            name:          sys_desc.name,
            life:          sys_desc.life,
            gravity:       sys_desc.gravity,
            bounds:        sys_desc.bounds,
            
        }
    }
    fn find_unused_particle(&mut self) -> usize {
        for i in self.search_pos..self.particles.len() {
            if self.particles[i].life < 0.0 {
                self.search_pos = i;
                return i;
            }
        }

        for i in 0..self.search_pos {
            if self.particles[i].life < 0.0 {
                self.search_pos = i;
                return i;
            }
        }
        self.search_pos = 0;
        0
    }
    fn new_particle(&mut self) -> Particle {
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
        if self.search_pos > self.particles.len() {
            self.search_pos = self.particles.len() - 1;
        }
    }
    pub fn resize(&mut self, count: usize, device: &wgpu::Device) {
        self.particles.resize(count, Particle::default());
        self.particle_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Buffer"),
                contents: bytemuck::cast_slice(&self.particles.to_raw()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
    }
    /// Return number of particles in particle system.
    pub fn particle_count(&self) -> u32 {
        self.particles.len() as u32
    }
    pub fn particle_buf_size(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.particles.len() as u64 * ParticleRaw::size())
    }
    /// Set position of particle system.
    pub fn set_position(&mut self, pos: [f32; 3]) {
        self.position = Vector3::new(pos[0], pos[1], pos[2]);
    }
    pub fn set_texture(&mut self, gpu: &Gpu, texture_path: &Path) {
        let mut diffuse_data = Vec::new();
        new_input_file(&texture_path).unwrap().read_to_end(&mut diffuse_data).unwrap();
        let texture = Texture::new(&gpu.device, &gpu.queue, &diffuse_data, None).unwrap();
        self.texture = Some(texture);
    }
    /// Set number of particles spawned per frame.
    pub fn set_particle_rate(&mut self, particle_rate: usize) {
        self.particle_rate = particle_rate;
    }
    /// Set gravity of particle system.
    pub fn set_gravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }
    /// Set name of particle system.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    /// Set minimum and maximum particle weight.
    pub fn set_weight_bounds(&mut self, weight: Range<f32>) {
        self.bounds.weight = weight;
    }
    /// Set minimum and maximum initial particle velocity.
    pub fn set_initial_velocity_bounds(&mut self, init_vel: [Range<f32>; 3]) {
        self.bounds.init_vel = init_vel;
    }
    /// Set dimensions of area in which particles spawn.
    pub fn set_spawn_range(&mut self, spawn_range: SpawnRange) {
        self.bounds.spawn_range = spawn_range;
    }
    /// Set minimum and maximum particle lifetimes.
    pub fn set_life_bounds(&mut self, life: Range<f32>) {
        self.bounds.life = life;
    }
    /// Set minimum and maximum particle RGBA values.
    pub fn set_color_bounds(&mut self, color: [Range<f32>; 4]) {
        self.bounds.color = color;
    }
    /// Set minimum and maximum particle size.
    pub fn set_scale_bounds(&mut self, scale: Range<f32>) {
        self.bounds.scale = scale;
    }
    /// Clear particle system's particle buffer.
    pub fn clear(&mut self, encoder: &mut wgpu::CommandEncoder) {
        encoder.clear_buffer(&self.particle_buf, 0, self.particle_buf_size());
    }
}

pub struct ParticleSystemDescriptor {
    mesh_type: ParticleMeshType,
    count:     usize,
    rate:      usize,
    pos:       Vector3<f32>,
    name:      String,
    life:      f32,
    gravity:   f32,
    bounds:    ParticleSystemBounds,
}
impl Default for ParticleSystemDescriptor {
    fn default() -> Self {
        Self {
            mesh_type: ParticleMeshType::default(),
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
        
        self.set_vertex_buffer(0, sys.mesh.vbuf.slice(..));
        self.set_vertex_buffer(1, sys.particle_buf.slice(..));

        self.set_index_buffer(sys.mesh.ibuf.slice(..), wgpu::IndexFormat::Uint16);
        
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