pub mod particle;
pub mod texture;
pub mod camera;
pub mod random;
pub mod vec;
pub mod gpu;
pub mod delta;
pub mod bufio;
pub mod obj;
pub mod error;
pub mod particle_system_renderer;
pub mod particle_system;

use std::path::PathBuf;

use crate::error::BrumousResult;
use crate::particle_system::ParticleSystem;

use cgmath::Vector3;

/// Creates a new particle system.
pub trait CreateParticleSystem {
    fn create_particle_system(
        &self, 
        desc: &ParticleSystemDescriptor, 
    ) -> BrumousResult<ParticleSystem>;
    fn create_particle_system_with_renderer(
        &self, 
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        sys_desc: &ParticleSystemDescriptor, 
        rend_desc: &ParticleSystemRendererDescriptor,
    ) -> BrumousResult<ParticleSystem>;

}
impl CreateParticleSystem for wgpu::Device {
    fn create_particle_system(
        &self, 
        desc: &ParticleSystemDescriptor,
    ) -> BrumousResult<ParticleSystem> {
        ParticleSystem::new(self, desc)
    }
    fn create_particle_system_with_renderer(
        &self, 
        config: &wgpu::SurfaceConfiguration,
        queue: &wgpu::Queue,
        sys_desc: &ParticleSystemDescriptor, 
        rend_desc: &ParticleSystemRendererDescriptor,
    ) -> BrumousResult<ParticleSystem> {
        ParticleSystem::new_with_renderer(self, config, queue, sys_desc, rend_desc)
    }
}

/// Draws a new particle system
pub trait DrawParticleSystem<'a, 'b> where 'a: 'b {
    fn draw_particle_system(
        &'b mut self, 
        sys: &'a ParticleSystem, 
    );
}
impl<'a, 'b> DrawParticleSystem<'a, 'b> for wgpu::RenderPass<'a> where 'a: 'b {
    fn draw_particle_system(
        &'b mut self, 
        sys: &'a ParticleSystem, 
    ) {
        if let Some(renderer) = &sys.renderer {
            self.set_pipeline(&renderer.pipeline);

            for (i, group) in renderer.bind_groups.iter().enumerate() {
                self.set_bind_group(i as u32, group, &[]);
            }
            self.set_vertex_buffer(0, renderer.mesh.vertex_buf.slice(..));
            self.set_vertex_buffer(1, sys.particle_buf().slice(..));

            if let Some(index_buf) = &renderer.mesh.index_buf {
                self.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint16);
                self.draw_indexed(0..renderer.mesh.index_count, 0, 0..sys.particle_count());
            }
            else {
                self.draw(0..renderer.mesh.vertex_count, 0..sys.particle_count());
            }
        }
    }
}

pub struct ParticleSystemRendererDescriptor<'a> {
    pub texture: Option<&'a str>,
    pub mesh_type: ParticleMeshType,
}
impl<'a> Default for ParticleSystemRendererDescriptor<'a> {
    fn default() -> Self {
        Self {
            texture: Some("image/default.png"),
            mesh_type: ParticleMeshType::default(),
        }
    }
}

/// Describes characteristics of a particle system.
pub struct ParticleSystemDescriptor<'a> {
    pub max:     usize,
    pub rate:    usize,
    pub pos:     Vector3<f32>,
    pub name:    &'a str,
    pub life:    f32,
    pub gravity: f32,
    pub bounds:  ParticleSystemBounds,
}
impl<'a> Default for ParticleSystemDescriptor<'a> {
    fn default() -> Self {
        Self {
            max:       500,
            rate:      1,
            pos:       Vector3::new(0.0, 0.0, 0.0),
            name:      "Particle System",
            life:      1000.0,
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


/// Describes the range of possible values of a particle's traits.
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
            rot:         [Spread::new(0.0, 0.0); 4],
            color:       [Spread::new(0.5, 0.5); 4],
            mass:        Spread::new(0.5, 0.1),
            scale:       Spread::new(0.007, 0.002),
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