use std::num::NonZeroU64;
use std::time::Duration;
use std::collections::VecDeque;

use crate::particle::*;
use crate::random::Randf32;
use crate::error::BrumousResult;
use crate::ParticleSystemDescriptor;
use crate::ParticleSystemBounds;
use crate::vector::{Vec3, Vec4};

use wgpu::util::DeviceExt;


pub enum ParticleAnimation {
    Color(Box<dyn Fn(Vec4, f32) -> Vec4>),
    Scale(Box<dyn Fn(f32, f32) -> f32>),
}

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
    anims:      Vec<ParticleAnimation>,
    living:     Vec<Particle>,
    dummy:      Vec<ParticleInstance>,
}
impl ParticleSystem {
    pub fn new(
        device: &wgpu::Device,
        sys_desc: &ParticleSystemDescriptor,
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

        Ok(
            Self {
                particles,
                buf,
                spawnqueue,
                rate:       sys_desc.rate,
                position:   sys_desc.pos,
                name:       sys_desc.name.to_string(),
                life:       sys_desc.life,
                bounds:     sys_desc.bounds,
                attractors: Vec::new(),
                forces:     Vec::new(),
                rand:       Randf32::new(),
                anims:      Vec::new(),
                living:     Vec::with_capacity(sys_desc.max),
                dummy:      vec![ParticleInstance::default(); sys_desc.max],
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
    pub fn update(&mut self, delta: Duration, queue: &wgpu::Queue, vp: [f32; 3]) {
        let view_pos = Vec3::from(vp);

        let delta = delta.as_millis() as f32 / 1000.0;
        if self.life >= 0.0 {
            self.respawn_particles(self.rate);
        }
        self.life -= delta;

        for (index, particle) in self.particles.iter_mut().enumerate() {
            particle.life -= delta;
            if particle.life > 0.0 {
                particle.update_pos(delta, &self.attractors, &self.forces);
                particle.cam_dist = (particle.position - view_pos).len();
                self.living.push(*particle);
            }
            else {
                // Add dead particle to respawn queue if not already queued.
                if !particle.queued {
                    self.spawnqueue.push_back(index);
                    particle.queued = true;
                }
            }
        }

        self.living.sort_by(|p1, p2| p2.cam_dist.partial_cmp(&p1.cam_dist).unwrap());
        queue.write_buffer(
            &self.buf,
            0,
            bytemuck::cast_slice(&self.living.instance())
        );
        let rem = &self.dummy[self.living.len()..self.particles.len()];
        queue.write_buffer(
            &self.buf,
            self.living.len() as u64 * ParticleInstance::size(),
            bytemuck::cast_slice(&rem)
        );
        self.living.clear();
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

    pub fn add_animation(&mut self, anim: ParticleAnimation) {
        self.anims.push(anim);
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

pub struct ParticleSystemSet(pub Vec<ParticleSystem>);

impl ParticleSystemSet {
    pub fn new(systems: Vec<ParticleSystem>) -> Self {
        Self(systems)
    }

    pub fn systems(&self) -> &[ParticleSystem] {
        &self.0
    }

    pub fn update(&mut self, delta: Duration, queue: &wgpu::Queue, vp: [f32; 3]) {
        for sys in self.0.iter_mut() {
            sys.update(delta, queue, vp);
        }
    }

    pub fn add_attractor(&mut self, pos: [f32; 3], mass: f32) {
        for sys in self.0.iter_mut() {
            sys.attractors.push(ParticleAttractor::new(pos, mass));
        }
    }
}
