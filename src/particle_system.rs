use std::num::NonZeroU64;
use std::time::Duration;
use std::collections::VecDeque;

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
    spawnqueue: VecDeque<usize>,
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

        let spawnqueue = VecDeque::from((0..sys_desc.max).collect::<Vec<usize>>());

        let renderer = ParticleSystemRenderer::new(device, queue, config, rend_desc)?;

        Ok(
            Self {
                particles,
                buf,
                spawnqueue,
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

    fn respawn_particles(&mut self, rate: usize) {
        for _ in 0..rate {
            if let Some(idx) = self.spawnqueue.pop_front() {
                self.particles[idx] = Particle::new(&mut self.rand, &self.bounds, &self.position);
            }
            else {
                return;
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
                // Add dead particle to respawn queue if not already queued.
                if !particle.queued {
                    self.spawnqueue.push_back(index);
                    particle.queued = true;

                    queue.write_buffer(
                        &self.buf,
                        index as u64 * ParticleInstance::size(),
                        bytemuck::cast_slice(&[ParticleInstance::empty()])
                    );
                }
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
    pub fn set_max_particles(&mut self, new_max: usize, device: &wgpu::Device) {
        let old_max = self.particles.len();

        if new_max == old_max {
            return;
        }

        self.particles.resize(new_max, Particle::default());

        if new_max < old_max {
            // If max particles is decreased, existing indices in the spawn
            // queue larger than the new max particles will be invalidated.
            self.spawnqueue.retain(|i| *i < new_max);
        }
        else {
            // If max particles is increased, add new indices to spawn queue.
            let diff = new_max - old_max;
            for i in 0..diff {
                self.spawnqueue.push_back(old_max + i);
            }
        }
        
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
        self.attractors.push(ParticleAttractor::new(pos, mass));
    }

    pub fn add_light(&mut self, queue: &wgpu::Queue, position: [f32; 4], color: [f32; 4]) {
        self.renderer.add_light(queue, Light::new(position, color));
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

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Light {
    position: [f32; 4],
    color: [f32; 4],
    pad1: [f32; 4],
    pad2: [f32; 4],
}
impl Light {
    fn new(position: [f32; 4], color: [f32; 4]) -> Self {
        Self {
            position,
            color,
            pad1: [0.0; 4],
            pad2: [0.0; 4]
        }
    }
    pub fn green() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 0.0], 
            color: [0.0, 1.0, 0.0, 1.0],
            pad1: [0.0; 4],
            pad2: [0.0; 4]
        }
    }
    pub fn red() -> Self {
        Self {
            position: [1.0, 0.0, 0.0, 0.0], 
            color: [1.0, 0.0, 0.0, 1.0],
            pad1: [0.0; 4],
            pad2: [0.0; 4]
        }
    }
    pub fn size() -> u64 {
        std::mem::size_of::<Self>() as u64
    }
}
