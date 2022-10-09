use std::time::Instant;
use std::io::{self, Write, Read};
use std::path::{Path, PathBuf};

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use brumous::camera::*;
use brumous::texture::{Texture, DepthTexture};
use brumous::gpu::Gpu;
use brumous::delta::Delta;
use brumous::particle::*;
use brumous::bufio::new_input_file;

use brumous::CreateParticleSystem;
use brumous::DrawParticleSystem;

fn main() {
    pollster::block_on(run());
}

struct State {
    gpu: Gpu,
    camera: Camera,
    system: brumous::ParticleSystem,
    depth_texture: DepthTexture,
    delta: Delta,
    texture: Texture,
    pipeline: wgpu::RenderPipeline,
}
impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window) -> Self {
        let gpu = Gpu::init(&window).await;
        let camera = Camera::new(&gpu.config, &gpu.device);
        let depth_texture = DepthTexture::new(&gpu.device, &gpu.config, "Depth Texture");
        let texture = Texture::new(&gpu.device, &gpu.queue, Path::new("image/fire.jpg")).unwrap();

        let system = match gpu.device.create_particle_system(
            &brumous::ParticleSystemDescriptor {
                bounds: brumous::ParticleSystemBounds {
                    spawn_range: [0.0..0.0, 0.0..0.0, 0.0..0.0],
                    life:        1.0..10.0,
                    init_vel:    [-0.2..0.2, 0.05..0.1, -0.2..0.2],
                    rot:         [0.0..0.0, 0.0..0.0, 0.0..0.0, 0.0..0.0],
                    color:       [0.0..1.0, 0.0..1.0, 0.0..1.0, 0.0..1.0],
                    mass:        0.1..0.5,
                    scale:       0.005..0.010,
                },
                max: 500,
                rate: 3,
                ..Default::default()
            },
        ) {
            Ok(sys) => sys,
            Err(e) => panic!("{e}"),
        };
        
        let mut shader_str = String::new();

        new_input_file(Path::new("src/particle.wgsl")).unwrap()
        .read_to_string(&mut shader_str).unwrap();

        let shader = gpu.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(shader_str.into()),
            }
        );

        let pipeline_layout = gpu.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera.bind_layout,
                    &texture.bind_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let pipeline = gpu.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[
                        ParticleVertex::layout(),
                        ParticleInstance::layout(),
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_texture",
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format: gpu.config.format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        }),
                    ],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            }
        );

        Self {
            gpu,
            camera,
            depth_texture,
            system,
            delta: Delta::new(),
            texture,
            pipeline,
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

    fn update(&mut self, delta: f32) {
        self.camera.update(&self.gpu.queue);
        self.system.update(delta, &self.gpu.queue);
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

        rpass.draw_particle_system(&self.system, &self.pipeline, &[&self.camera.bind_group, &self.texture.bind_group]);
        
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
                state.update(state.delta.frame_time_f32());
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
 