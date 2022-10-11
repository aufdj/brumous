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

use crate::error::BrumousResult;
use crate::particle_system::ParticleSystem;
use crate::particle_system::ParticleSystemDescriptor;
use crate::particle_system_renderer::ParticleSystemRendererDescriptor;


/// Creates a new particle system.
pub trait CreateParticleSystem {
    fn create_particle_system(
        &self, 
        desc: &ParticleSystemDescriptor, 
    ) -> BrumousResult<ParticleSystem>;
    fn create_particle_system_with_renderer(
        &self, 
        config:    &wgpu::SurfaceConfiguration,
        queue:     &wgpu::Queue,
        sys_desc:  &ParticleSystemDescriptor, 
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
        config:    &wgpu::SurfaceConfiguration,
        queue:     &wgpu::Queue,
        sys_desc:  &ParticleSystemDescriptor, 
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