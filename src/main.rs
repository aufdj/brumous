use std::time::{Duration, Instant};
use std::io::{self, Write};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use brumous::camera::*;
use brumous::texture::DepthTexture;
use brumous::gpu::Gpu;
use brumous::delta::Delta;

use brumous::CreateParticleSystem;
use brumous::DrawParticleSystem;
use brumous::MVar;

fn main() {
    pollster::block_on(run());
}

struct State {
    gpu: Gpu,
    camera: Camera,
    system: brumous::particle_system::ParticleSystem,
    depth_texture: DepthTexture,
    delta: Delta,
}
impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let gpu = Gpu::init(&window).await;
        let camera = Camera::new(&gpu.config, &gpu.device);
        let depth_texture = DepthTexture::new(&gpu.device, &gpu.config, "Depth Texture");

        let renderer = gpu.device.create_particle_system_renderer(
            &gpu.queue,
            &gpu.config,
            &brumous::ParticleSystemRendererDescriptor {
                texture: Some("image/metal.jpg"),
                ..Default::default()
            }
        ).unwrap();

        let system = gpu.device.create_particle_system(
            &brumous::ParticleSystemDescriptor {
                bounds: brumous::ParticleSystemBounds {
                    scale: MVar(0.005, 0.0005),
                    mass: MVar(1.5, 0.005),
                    ..Default::default()
                },
                max: 1000,
                rate: 3,
                gravity: [1.0, 1.0, 0.0].into(),
                ..Default::default()
            },
        ).unwrap().with_renderer(renderer);
    
        Self {
            gpu,
            camera,
            depth_texture,
            system,
            delta: Delta::new(),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.gpu.resize_window(new_size);
            self.depth_texture = DepthTexture::new(&self.gpu.device, &self.gpu.config, "Depth Texture");
        }
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera.controller.process_events(event)
    }

    fn update(&mut self, delta: Duration) {
        self.camera.update(&self.gpu.queue);
        self.system.update(delta, &self.gpu.queue);
        self.system.set_view_proj(&self.gpu.queue, self.camera.uniform.view_proj);
        self.system.set_view_pos(&self.gpu.queue, self.camera.view_pos());
    }

    fn render(&mut self, view: &wgpu::TextureView) -> Result<(), wgpu::SurfaceError> {
        let mut encoder = self.gpu.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            }
        );

        let mut rpass = encoder.begin_render_pass(
            &wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color::BLACK
                            ),
                            store: true,
                        },
                    }),
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            }
        );

        rpass.draw_particle_system(&self.system);
        
        drop(rpass);

        self.system.clear(&mut encoder);

        self.gpu.queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
 
pub async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = State::new(&window).await;
    let mut stdout = io::stdout().lock();
    
    // Opens the window and starts processing events
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::NewEvents(StartCause::Poll) => {
                state.delta.update(Instant::now());
                stdout.write_fmt(
                    format_args!("\rframetime: {:?}  ", state.delta.frame_time())
                ).unwrap();
            }
            Event::WindowEvent { ref event, window_id, } if window_id == window.id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested | WindowEvent::KeyboardInput {
                            input: KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                            ..
                        } => {
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::MainEventsCleared => {
                state.update(state.delta.frame_time());
                let frame = match state.gpu.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("dropped frame: {:?}", e);
                        return;
                    }
                };
                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                
                if let Err(err) = state.render(&view) {
                    match err {
                        wgpu::SurfaceError::Lost => {
                            state.resize(state.gpu.size);
                        }
                        wgpu::SurfaceError::OutOfMemory => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {
                            eprintln!("{err:?}");
                        }
                    }
                }
                frame.present();
            
                window.request_redraw();
            }
            _ => {},
        }
    });
}
 