use std::num::NonZeroU64;
use std::time::Duration;

use crate::particle::*;
use crate::random::Randf32;
use crate::error::BrumousResult;
use crate::particle_system_renderer::ParticleSystemRenderer;
use crate::ParticleSystemRendererDescriptor;
use crate::ParticleSystemDescriptor;
use crate::ParticleSystemBounds;
use crate::MVar;

use wgpu::util::DeviceExt;
use crate::vector::Vec3;

/// A ParticleSystem manages a set of particles.
pub struct ParticleSystem {
    particles:  Vec<Particle>,
    buf:        wgpu::Buffer,
    search_pos: usize,
    rate:       usize,
    position:   Vec3,
    name:       String,
    life:       f32,
    gravity:    Vec3,
    bounds:     ParticleSystemBounds,
    forces:     Vec<Vec3>,
    rand:       Randf32,
    renderer:   Option<ParticleSystemRenderer>,
}
impl ParticleSystem {
    pub fn new(
        device: &wgpu::Device,
        desc: &ParticleSystemDescriptor, 
    ) -> BrumousResult<Self> {
        let particles = vec![Particle::default(); desc.max];

        let buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&particles.instance()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        Ok(
            Self {
                particles,
                buf,
                search_pos:   0,
                rate:         desc.rate,
                position:     desc.pos,
                name:         desc.name.to_string(),
                life:         desc.life,
                gravity:      desc.gravity,
                bounds:       desc.bounds,
                forces:       Vec::new(),
                rand:         Randf32::new(),
                renderer:     None,
            }
        )
    }

    pub fn with_renderer(mut self, renderer: ParticleSystemRenderer) -> Self {
        self.renderer = Some(renderer);
        self
    }

    pub fn new_with_renderer(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        sys_desc: &ParticleSystemDescriptor,
        rend_desc: &ParticleSystemRendererDescriptor, 
    ) -> BrumousResult<Self> {
        let particles = vec![Particle::default(); sys_desc.max];

        let buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&particles.instance()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let renderer = Some(
            ParticleSystemRenderer::new(device, queue, config, rend_desc)?
        );

        Ok(
            Self {
                particles,
                buf,
                search_pos: 0,
                rate:       sys_desc.rate,
                position:   sys_desc.pos,
                name:       sys_desc.name.to_string(),
                life:       sys_desc.life,
                gravity:    sys_desc.gravity,
                bounds:     sys_desc.bounds,
                forces:     Vec::new(),
                rand:       Randf32::new(),
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
            pos:   self.rand.vec3_in_variance(&self.bounds.spawn_range) + self.position,
            vel:   self.rand.vec3_in_variance(&self.bounds.init_vel), 
            rot:   self.rand.quat_in_variance(&self.bounds.rot), 
            color: self.rand.vec4_in_variance(&self.bounds.color),
            scale: self.rand.in_variance(&self.bounds.scale),
            life:  self.rand.in_variance(&self.bounds.life), 
            mass:  self.rand.in_variance(&self.bounds.mass),
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
                particle.update(delta, self.gravity, &self.forces);
                queue.write_buffer(
                    &self.buf,
                    index as u64 * ParticleInstance::size(),
                    bytemuck::cast_slice(&[particle.instance()])
                );
            }
        }
        if self.search_pos > self.particles.len() {
            self.search_pos = self.particles.len() - 1;
        }
    }

    /// Clear particle system's particle buffer.
    pub fn clear(&mut self, encoder: &mut wgpu::CommandEncoder) {
        encoder.clear_buffer(&self.buf, 0, self.particle_buf_size());
    }

    /// Return number of particles in particle system.
    pub fn particle_count(&self) -> u32 {
        self.particles.len() as u32
    }

    /// Return reference to particle buffer.
    pub fn particle_buf(&self) -> &wgpu::Buffer {
        &self.buf
    }

    /// Return particle buffer size in bytes.
    pub fn particle_buf_size(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.particles.len() as u64 * ParticleInstance::size())
    }

    /// Set max number of particles.
    pub fn set_max_particles(&mut self, max: usize, device: &wgpu::Device) {
        self.particles.resize(max, Particle::default());
        self.buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Buffer"),
                contents: bytemuck::cast_slice(&self.particles.instance()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
    }

    /// Set position of particle system.
    pub fn set_position(&mut self, pos: [f32; 3]) {
        self.position = Vec3::new(pos[0], pos[1], pos[2]);
    }

    /// Set number of particles spawned per frame.
    pub fn set_rate(&mut self, rate: usize) {
        self.rate = rate;
    }

    /// Set gravity of particle system.
    pub fn set_gravity(&mut self, gravity: [f32; 3]) {
        self.gravity = Vec3::new(gravity[0], gravity[1], gravity[2]);
    }

    /// Set name of particle system.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Set minimum and maximum particle mass.
    pub fn set_mass_variance(&mut self, mass: MVar) {
        self.bounds.mass = mass;
    }

    /// Set minimum and maximum initial particle velocity.
    pub fn set_initial_velocity_variance(&mut self, init_vel: [MVar; 3]) {
        self.bounds.init_vel = init_vel;
    }

    /// Set dimensions of area in which particles spawn.
    pub fn set_spawn_variance(&mut self, spawn_range: [MVar; 3]) {
        self.bounds.spawn_range = spawn_range;
    }

    /// Set minimum and maximum particle lifetimes.
    pub fn set_life_variance(&mut self, life: MVar) {
        self.bounds.life = life;
    }

    /// Set minimum and maximum particle RGBA values.
    pub fn set_color_variance(&mut self, color: [MVar; 4]) {
        self.bounds.color = color;
    }

    /// Set minimum and maximum particle size.
    pub fn set_scale_variance(&mut self, scale: MVar) {
        self.bounds.scale = scale;
    }

    pub fn apply_force(&mut self, force: [f32; 3]) {
        self.forces.push(force.into());
    }

    pub fn set_view_proj(&mut self, queue: &wgpu::Queue, vp: [[f32; 4]; 4]) {
        if let Some(renderer) = &self.renderer {
            queue.write_buffer(&renderer.view_data, 0, bytemuck::cast_slice(&[vp]));
        }
    }

    pub fn set_view_pos(&mut self, queue: &wgpu::Queue, vp: [f32; 4]) {
        if let Some(renderer) = &self.renderer {
            queue.write_buffer(&renderer.view_data, 64, bytemuck::cast_slice(&[vp]));
        }
    }

    pub fn renderer(&self) -> Option<&ParticleSystemRenderer> {
        self.renderer.as_ref()
    }

}
