use std::num::NonZeroU64;
use std::time::Duration;

use crate::particle::*;
use crate::random::Randf32;
use crate::error::BrumousResult;
use crate::particle_system_renderer::ParticleSystemRenderer;
use crate::ParticleSystemRendererDescriptor;
use crate::ParticleSystemDescriptor;
use crate::ParticleSystemBounds;
use crate::vector::Vec3;

use wgpu::util::DeviceExt;


/// A ParticleSystem manages a set of particles.
pub struct ParticleSystem {
    particles:  Vec<Particle>,
    buf:        wgpu::Buffer,
    search_pos: usize,
    rate:       usize,
    position:   Vec3,
    name:       String,
    life:       f32,
    attractors: Vec<ParticleAttractor>,
    bounds:     ParticleSystemBounds,
    forces:     Vec<Vec3>,
    rand:       Randf32,
    renderer:   ParticleSystemRenderer,
}
impl ParticleSystem {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
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

        let renderer = ParticleSystemRenderer::new(device, queue, config, rend_desc)?;

        Ok(
            Self {
                particles,
                buf,
                search_pos: 0,
                rate:       sys_desc.rate,
                position:   sys_desc.pos,
                name:       sys_desc.name.to_string(),
                life:       sys_desc.life,
                attractors: Vec::new(),
                bounds:     sys_desc.bounds,
                forces:     Vec::new(),
                rand:       Randf32::new(),
                renderer,
            }
        )
    }

    fn respawn_particles(&mut self, mut rate: usize) {
        for (i, particle) in self.particles
        .iter_mut()
        .skip(self.search_pos)
        .enumerate() {
            if particle.life < 0.0 {
                self.search_pos = self.search_pos + i;
                *particle = Particle::new(&mut self.rand, &self.bounds, &self.position);
                rate -= 1;
                if rate == 0 {
                    return;
                }
            }
        }
        for (i, particle) in self.particles
        .iter_mut()
        .take(self.search_pos)
        .enumerate() {
            if particle.life < 0.0 {
                self.search_pos = self.search_pos + i;
                *particle = Particle::new(&mut self.rand, &self.bounds, &self.position);
                rate -= 1;
                if rate == 0 {
                    return;
                }
            }
        }
    }

    /// Spawn new particles and update existing particles, should be called every frame.
    pub fn update(&mut self, delta: Duration, queue: &wgpu::Queue) {
        let delta = delta.as_millis() as f32 / 1000.0;
        if self.life >= 0.0 {
            self.respawn_particles(self.rate);
        }
        self.life -= delta;

        for (index, particle) in self.particles.iter_mut().enumerate() {
            particle.life -= delta;
            if particle.life > 0.0 {
                particle.update(delta, &self.attractors, &self.forces);
                queue.write_buffer(
                    &self.buf,
                    index as u64 * ParticleInstance::size(),
                    bytemuck::cast_slice(&[particle.instance()])
                );
            }
            else {
                queue.write_buffer(
                    &self.buf,
                    index as u64 * ParticleInstance::size(),
                    bytemuck::cast_slice(&[ParticleInstance::empty()])
                );
            }
        }
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
    pub fn set_position(&mut self, position: [f32; 3]) {
        self.position = position.into();
    }

    /// Set number of particles spawned per frame.
    pub fn set_rate(&mut self, rate: usize) {
        self.rate = rate;
    }

    /// Set name of particle system.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    /// Set minimum and maximum particle mass.
    pub fn set_mass_variance(&mut self, mass: (f32, f32)) {
        self.bounds.mass = mass;
    }

    /// Set minimum and maximum initial particle velocity.
    pub fn set_initial_velocity_variance(&mut self, velocity: [(f32, f32); 3]) {
        self.bounds.velocity = velocity;
    }

    /// Set dimensions of area in which particles spawn.
    pub fn set_spawn_variance(&mut self, area: [(f32, f32); 3]) {
        self.bounds.area = area;
    }

    /// Set minimum and maximum particle lifetimes.
    pub fn set_life_variance(&mut self, life: (f32, f32)) {
        self.bounds.life = life;
    }

    /// Set minimum and maximum particle RGBA values.
    pub fn set_color_variance(&mut self, color: [(f32, f32); 4]) {
        self.bounds.color = color;
    }

    /// Set minimum and maximum particle size.
    pub fn set_scale_variance(&mut self, scale: (f32, f32)) {
        self.bounds.scale = scale;
    }

    pub fn add_force(&mut self, force: [f32; 3]) {
        self.forces.push(force.into());
    }

    pub fn add_attractor(&mut self, pos: [f32; 3], mass: f32) {
        self.attractors.push(ParticleAttractor::new(pos.into(), mass));
    }

    pub fn set_view_proj(&mut self, queue: &wgpu::Queue, vp: [[f32; 4]; 4]) {
        queue.write_buffer(&self.renderer.view_data, 0, bytemuck::cast_slice(&[vp]));
    }

    pub fn set_view_pos(&mut self, queue: &wgpu::Queue, vp: [f32; 4]) {
        queue.write_buffer(&self.renderer.view_data, 64, bytemuck::cast_slice(&[vp]));
    }

    pub fn renderer(&self) -> &ParticleSystemRenderer {
        &self.renderer
    }
}

pub struct ParticleAttractor {
    pub pos: Vec3,
    pub mass: f32,
}
impl ParticleAttractor {
    fn new(pos: [f32; 3], mass: f32) -> Self {
        Self { 
            pos: pos.into(), 
            mass 
        }
    }
}
