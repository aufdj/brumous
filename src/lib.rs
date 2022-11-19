mod particle;
mod texture;
mod random;
mod vector;
mod matrix;
mod quaternion;
mod obj;
pub mod particle_system_renderer;
pub mod error;
pub mod particle_system;

use crate::error::BrumousResult;
use crate::particle_system::ParticleSystem;
use crate::particle_system_renderer::ParticleSystemRenderer;
use crate::vector::Vec3;

/// Creates a new particle system.
pub trait CreateParticleSystem {
    fn create_particle_system(
        &self, 
        desc: &ParticleSystemDescriptor,
    ) -> BrumousResult<ParticleSystem>;

    fn create_particle_system_renderer(
        &self,
        queue: &wgpu::Queue, 
        config: &wgpu::SurfaceConfiguration,
        desc: &ParticleSystemRendererDescriptor, 
    ) -> BrumousResult<ParticleSystemRenderer>;
}
impl CreateParticleSystem for wgpu::Device {
    fn create_particle_system(
        &self, 
        desc: &ParticleSystemDescriptor,
    ) -> BrumousResult<ParticleSystem> {
        ParticleSystem::new(self, desc)
    }

    fn create_particle_system_renderer(
        &self,
        queue: &wgpu::Queue, 
        config: &wgpu::SurfaceConfiguration,
        desc: &ParticleSystemRendererDescriptor, 
    ) -> BrumousResult<ParticleSystemRenderer> {
        ParticleSystemRenderer::new(self, queue, config, desc)
    }
}

/// Draw particles in particle system
pub trait DrawParticleSystem<'a, 'b> where 'a: 'b {
    fn draw_particle_system(
        &'b mut self, 
        sys: &'a ParticleSystem, 
        rend: &'a ParticleSystemRenderer
    );
}
impl<'a, 'b> DrawParticleSystem<'a, 'b> for wgpu::RenderPass<'a> where 'a: 'b {
    fn draw_particle_system(
        &'b mut self, 
        sys: &'a ParticleSystem, 
        rend: &'a ParticleSystemRenderer
    ) {
        self.set_pipeline(&rend.pipeline);

        for (i, group) in rend.bind_groups.iter().enumerate() {
            self.set_bind_group(i as u32, group, &[]);
        }

        self.set_vertex_buffer(0, rend.mesh.vertex_buf.slice(..));
        self.set_vertex_buffer(1, sys.particle_buf().slice(..));

        if let Some(index_buf) = &rend.mesh.index_buf {
            self.set_index_buffer(index_buf.slice(..), wgpu::IndexFormat::Uint16);
            self.draw_indexed(0..rend.mesh.index_count, 0, 0..sys.particle_count());
        }
        else {
            self.draw(0..rend.mesh.vertex_count, 0..sys.particle_count());
        }
    }
}


/// Describe characteristics of a particle system.
pub struct ParticleSystemDescriptor<'a> {
    pub max:    usize,
    pub rate:   usize,
    pub pos:    Vec3,
    pub name:   &'a str,
    pub life:   f32,
    pub bounds: ParticleSystemBounds,
}
impl<'a> Default for ParticleSystemDescriptor<'a> {
    fn default() -> Self {
        Self {
            max:    500,
            rate:   1,
            pos:    Vec3::zero(),
            name:   "Particle System",
            life:   1000.0,
            bounds: ParticleSystemBounds::default(),
        }
    }
}

/// Describes the range of possible values of a particle's traits.
#[derive(Copy, Clone)]
pub struct ParticleSystemBounds {
    pub area:     [(f32, f32); 3],
    pub velocity: [(f32, f32); 3],
    pub rotation: [(f32, f32); 4],
    pub color:    [(f32, f32); 4],
    pub life:     (f32, f32),
    pub mass:     (f32, f32),
    pub scale:    (f32, f32),
}
impl Default for ParticleSystemBounds {
    fn default() -> Self {
        Self {
            area:     [(0.0, 0.0); 3],
            velocity: [(0.0, 0.2), (0.7, 0.2), (0.0, 0.2)],
            rotation: [(0.0, 0.0); 4],
            color:    [(0.5, 0.5); 4],
            life:     (5.0, 2.0),
            mass:     (1.0, 0.1),
            scale:    (0.007, 0.002),
        }
    }
}


#[derive(Default)]
pub struct ParticleSystemRendererDescriptor<'a> {
    pub texture: Option<&'a str>,
    pub mesh_type: ParticleMeshType<'a>,
}

/// Defines model of each particle.
#[derive(Default)]
pub enum ParticleMeshType<'a> {
    #[default]
    Cube,
    Point,
    Custom(&'a str),
}