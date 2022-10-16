use std::num::NonZeroU64;
use std::time::Duration;

use crate::particle::*;
use crate::random::Randf32;
use crate::error::BrumousResult;
use crate::particle_system_renderer::ParticleSystemRenderer;
use crate::particle_system_renderer::ParticleSystemRendererDescriptor;

use wgpu::util::DeviceExt;
use cgmath::Vector3;

/// A ParticleSystem manages a set of particles.
pub struct ParticleSystem {
    particles:    Vec<Particle>,
    particle_buf: wgpu::Buffer,
    search_pos:   usize,
    rate:         usize,
    position:     Vector3<f32>,
    name:         String,
    life:         f32,
    gravity:      f32,
    bounds:       ParticleSystemBounds,
    rand:         Randf32,
    pub renderer: Option<ParticleSystemRenderer>,
}
impl ParticleSystem {
    pub fn new(
        device: &wgpu::Device,
        desc:   &ParticleSystemDescriptor, 
    ) -> BrumousResult<Self> {
        let particles = vec![Particle::default(); desc.max];

        let particle_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&particles.to_raw()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        Ok(
            Self {
                particles,
                particle_buf,
                search_pos:   0,
                rate:         desc.rate,
                position:     desc.pos,
                name:         desc.name.to_string(),
                life:         desc.life,
                gravity:      desc.gravity,
                bounds:       desc.bounds.clone(),
                rand:         Randf32::new(),
                renderer:     None,
            }
        )
    }
    pub fn new_with_renderer(
        device:    &wgpu::Device,
        config:    &wgpu::SurfaceConfiguration,
        queue:     &wgpu::Queue,
        sys_desc:  &ParticleSystemDescriptor,
        rend_desc: &ParticleSystemRendererDescriptor, 
    ) -> BrumousResult<Self> {
        let particles = vec![Particle::default(); sys_desc.max];

        let particle_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&particles.to_raw()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let renderer = Some(
            ParticleSystemRenderer::new(device, queue, config, rend_desc)?
        );

        Ok(
            Self {
                particles,
                particle_buf,
                search_pos:    0,
                rate:          sys_desc.rate,
                position:      sys_desc.pos,
                name:          sys_desc.name.to_string(),
                life:          sys_desc.life,
                gravity:       sys_desc.gravity,
                bounds:        sys_desc.bounds.clone(),
                rand:          Randf32::new(),
                renderer,
            }
        )
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
            pos:   self.rand.vec3_spread(&self.bounds.spawn_range) + self.position,
            vel:   self.rand.vec3_spread(&self.bounds.init_vel), 
            rot:   self.rand.quat_spread(&self.bounds.rot), 
            color: self.rand.vec4_spread(&self.bounds.color),
            scale: self.rand.spread(&self.bounds.scale),
            life:  self.rand.spread(&self.bounds.life), 
            mass:  self.rand.spread(&self.bounds.mass),
        }
    }
    /// Spawn new particles and update existing particles, should be called every frame.
    pub fn update(&mut self, delta: Duration, queue: &wgpu::Queue) {
        let delta = delta.as_millis() as f32 / 1000.0;
        if self.life >= 0.0 {
            for _ in 0..self.rate {
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
    pub fn set_rate(&mut self, rate: usize) {
        self.rate = rate;
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
    pub fn set_mass_spread(&mut self, mass: Spread) {
        self.bounds.mass = mass;
    }
    /// Set minimum and maximum initial particle velocity.
    pub fn set_initial_velocity_spread(&mut self, init_vel: [Spread; 3]) {
        self.bounds.init_vel = init_vel;
    }
    /// Set dimensions of area in which particles spawn.
    pub fn set_spawn_range(&mut self, spawn_range: [Spread; 3]) {
        self.bounds.spawn_range = spawn_range;
    }
    /// Set minimum and maximum particle lifetimes.
    pub fn set_life_spread(&mut self, life: Spread) {
        self.bounds.life = life;
    }
    /// Set minimum and maximum particle RGBA values.
    pub fn set_color_spread(&mut self, color: [Spread; 4]) {
        self.bounds.color = color;
    }
    /// Set minimum and maximum particle size.
    pub fn set_scale_spread(&mut self, scale: Spread) {
        self.bounds.scale = scale;
    }
    pub fn set_view_proj(&mut self, queue: &wgpu::Queue, vp: [[f32; 4]; 4]) {
        if let Some(renderer) = &self.renderer {
            queue.write_buffer(&renderer.view_proj, 0, bytemuck::cast_slice(&[vp]));
        }
    }
    pub fn set_renderer(&mut self, renderer: ParticleSystemRenderer) {
        self.renderer = Some(renderer);
    }
}

/// Describes characteristics of a particle system.
pub struct ParticleSystemDescriptor<'a> {
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

#[derive(Copy, Clone)]
pub struct Spread {
    pub mean:     f32,
    pub variance: f32,
}
impl Spread {
    pub fn new(mean: f32, variance: f32) -> Self {
        Self {
            mean, variance
        }
    }
}


/// Describes the range of possible values
/// of a particle's traits.
#[derive(Copy, Clone)]
pub struct ParticleSystemBounds {
    pub spawn_range: [Spread; 3],
    pub init_vel:    [Spread; 3],
    pub rot:         [Spread; 4],
    pub color:       [Spread; 4],
    pub life:        Spread,
    pub mass:        Spread,
    pub scale:       Spread,
}
impl Default for ParticleSystemBounds {
    fn default() -> Self {
        Self {
            spawn_range: [Spread::new(0.0, 0.0); 3],
            life:        Spread::new(5.0, 2.0),
            init_vel:    [Spread::new(0.0, 0.2), Spread::new(0.7, 0.2), Spread::new(0.0, 0.2)],
            rot:         [Spread::new(0.0, 0.5); 4],
            color:       [Spread::new(0.5, 0.5); 4],
            mass:        Spread::new(0.5, 0.1),
            scale:       Spread::new(0.007, 0.002),
        }
    }
}