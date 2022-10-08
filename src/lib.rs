pub mod particle;
pub mod texture;
pub mod camera;
pub mod random;
pub mod vec;
pub mod gpu;
pub mod delta;
pub mod bufio;
pub mod obj;

use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::ops::Range;

use crate::particle::*;
use crate::texture::Texture;
use crate::gpu::Gpu;
use crate::random::Randf32;

use wgpu::util::DeviceExt;
use cgmath::Vector3;

/// A ParticleSystem manages a set of particles.
pub struct ParticleSystem {
    particles:     Vec<Particle>,
    pub mesh:      ParticleMesh,
    particle_buf:  wgpu::Buffer,
    search_pos:    usize,
    particle_rate: usize,
    position:      Vector3<f32>,
    texture:       Option<Texture>,
    name:          String,
    life:          f32,
    gravity:       f32,
    bounds:        ParticleSystemBounds,
    rand:          Randf32,
}
impl ParticleSystem {
    pub fn new(
        device: &wgpu::Device,
        sys_desc: &ParticleSystemDescriptor, 
    ) -> Self {
        let particles = vec![Particle::default(); sys_desc.max];
        let mesh = ParticleMesh::new(device, &sys_desc.mesh_type);

        let particle_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&particles.to_raw()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        Self {
            particles,
            particle_buf,
            mesh,
            texture:       None,
            search_pos:    0,
            particle_rate: sys_desc.rate,
            position:      sys_desc.pos,
            name:          sys_desc.name.to_string(),
            life:          sys_desc.life,
            gravity:       sys_desc.gravity,
            bounds:        sys_desc.bounds.clone(),
            rand:          Randf32::new(),
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
    /// Create new particle 
    fn new_particle(&mut self) -> Particle {
        Particle {
            pos:   self.rand.vec3_in_range(&self.bounds.spawn_range) + self.position,
            vel:   self.rand.vec3_in_range(&self.bounds.init_vel), 
            rot:   self.rand.quat_in_range(&self.bounds.rot), 
            color: self.rand.vec4_in_range(&self.bounds.color),
            scale: self.rand.in_range(&self.bounds.scale),
            life:  self.rand.in_range(&self.bounds.life), 
            mass:  self.rand.in_range(&self.bounds.mass),
        }
    }
    /// Spawn new particles and update existing particles, should be called every frame.
    pub fn update(&mut self, delta: f32, queue: &wgpu::Queue) {
        if self.life >= 0.0 {
            for _ in 0..self.particle_rate {
                let idx = self.find_unused_particle();
                self.particles[idx] = self.new_particle();
            }
        }
        self.life -= delta;

        for (index, particle) in self.particles.iter_mut().enumerate() {
            particle.life -= delta;
            if particle.life > 0.0 {
                particle.update(delta, self.gravity);
                queue.write_buffer(
                    &self.particle_buf,
                    index as u64 * ParticleInstance::size(),
                    bytemuck::cast_slice(&[particle.to_raw()])
                );
            }
        }
        if self.search_pos > self.particles.len() {
            self.search_pos = self.particles.len() - 1;
        }
    }
    /// Clear particle system's particle buffer.
    pub fn clear(&mut self, encoder: &mut wgpu::CommandEncoder) {
        encoder.clear_buffer(&self.particle_buf, 0, self.particle_buf_size());
    }
    /// Return number of particles in particle system.
    pub fn particle_count(&self) -> u32 {
        self.particles.len() as u32
    }
    /// Return reference to particle buffer.
    pub fn particle_buf(&self) -> &wgpu::Buffer {
        &self.particle_buf
    }
    /// Return particle buffer size in bytes.
    pub fn particle_buf_size(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.particles.len() as u64 * ParticleInstance::size())
    }
    /// Set max number of particles.
    pub fn set_max_particles(&mut self, max: usize, device: &wgpu::Device) {
        self.particles.resize(max, Particle::default());
        self.particle_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Buffer"),
                contents: bytemuck::cast_slice(&self.particles.to_raw()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
    }
    /// Set position of particle system.
    pub fn set_position(&mut self, pos: [f32; 3]) {
        self.position = Vector3::new(pos[0], pos[1], pos[2]);
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
    /// Set minimum and maximum particle mass.
    pub fn set_mass_bounds(&mut self, mass: Range<f32>) {
        self.bounds.mass = mass;
    }
    /// Set minimum and maximum initial particle velocity.
    pub fn set_initial_velocity_bounds(&mut self, init_vel: [Range<f32>; 3]) {
        self.bounds.init_vel = init_vel;
    }
    /// Set dimensions of area in which particles spawn.
    pub fn set_spawn_range(&mut self, spawn_range: [Range<f32>; 3]) {
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
}

/// Describes characteristics of a particle system.
pub struct ParticleSystemDescriptor<'a> {
    pub mesh_type: ParticleMeshType,
    pub max:       usize,
    pub rate:      usize,
    pub pos:       Vector3<f32>,
    pub name:      &'a str,
    pub life:      f32,
    pub gravity:   f32,
    pub bounds:    ParticleSystemBounds,
}
impl<'a> Default for ParticleSystemDescriptor<'a> {
    fn default() -> Self {
        Self {
            mesh_type: ParticleMeshType::default(),
            max:       500,
            rate:      3,
            pos:       Vector3::new(0.0, 0.0, 0.0),
            name:      "Particle System",
            life:      5.0,
            gravity:   -9.81,
            bounds:    ParticleSystemBounds::default(),
        }
    }
}

/// Defines model of each particle.
#[derive(Default)]
pub enum ParticleMeshType {
    #[default]
    Cube,
    Custom(PathBuf),
}


/// Describes the range of possible values 
/// of a particle's traits.
#[derive(Clone)]
pub struct ParticleSystemBounds {
    pub spawn_range: [Range<f32>; 3],
    pub init_vel:    [Range<f32>; 3],
    pub rot:         [Range<f32>; 4],
    pub color:       [Range<f32>; 4],
    pub life:        Range<f32>,
    pub mass:        Range<f32>,
    pub scale:       Range<f32>,
}
impl Default for ParticleSystemBounds {
    fn default() -> Self {
        Self {
            spawn_range: [0.0..0.0, 0.0..0.0, 0.0..0.0],
            life:        1.0..10.0,
            init_vel:    [-0.2..0.2, 0.5..1.0, -0.2..0.2],
            rot:         [-0.5..0.5, -0.5..0.5, -0.5..0.5, -0.5..0.5],
            color:       [0.0..1.0, 0.0..1.0, 0.0..1.0, 0.0..1.0],
            mass:        0.1..1.0,
            scale:       0.005..0.010,
        }
    }
}

/// Creates a new particle system.
pub trait CreateParticleSystem {
    fn create_particle_system(
        &self, 
        desc: &ParticleSystemDescriptor, 
    ) -> ParticleSystem;
}
impl CreateParticleSystem for wgpu::Device {
    fn create_particle_system(
        &self, 
        desc: &ParticleSystemDescriptor,
    ) -> ParticleSystem {
        ParticleSystem::new(self, desc)
    }
}

/// Draws a new particle system
pub trait DrawParticleSystem<'a, 'b> where 'a: 'b {
    fn draw_particle_system(
        &'b mut self, 
        sys: &'a ParticleSystem, 
        pipeline: &'a wgpu::RenderPipeline,
        bind_groups: &[&'a wgpu::BindGroup]
    );
}
impl<'a, 'b> DrawParticleSystem<'a, 'b> for wgpu::RenderPass<'a> where 'a: 'b {
    fn draw_particle_system(
        &'b mut self, 
        sys: &'a ParticleSystem, 
        pipeline: &'a wgpu::RenderPipeline,
        bind_groups: &[&'a wgpu::BindGroup]
    ) {
        self.set_pipeline(pipeline);

        for (i, group) in bind_groups.iter().enumerate() {
            self.set_bind_group(i as u32, group, &[]);
        }
        
        self.set_vertex_buffer(0, sys.mesh.vertex_buf.slice(..));
        self.set_vertex_buffer(1, sys.particle_buf().slice(..));

        if let Some(index_buf) = &sys.mesh.index_buf {
            self.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint16);
            self.draw_indexed(0..sys.mesh.index_count, 0, 0..sys.particle_count());
        }
        else {
            self.draw(0..sys.mesh.vertex_count, 0..sys.particle_count());
        }
    }
}