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

use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::ops::Range;
use std::io::Read;

use crate::particle::*;
use crate::texture::Texture;
use crate::gpu::Gpu;
use crate::random::Randf32;
use crate::error::BrumousResult;
use crate::bufio::new_input_file;
use crate::particle_system_renderer::ParticleSystemRenderer;
use crate::particle_system::ParticleSystem;
use crate::particle_system::ParticleSystemDescriptor;

use wgpu::util::DeviceExt;
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
        desc: &ParticleSystemDescriptor, 
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
        desc: &ParticleSystemDescriptor, 
    ) -> BrumousResult<ParticleSystem> {
        ParticleSystem::new_with_renderer(self, config, queue, desc)
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