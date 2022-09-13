mod texture;
mod camera;
mod model;
mod random;
mod vec;
mod light;
mod particle;
pub mod config;
mod error;
mod gpu;
mod delta;
mod io;
mod gui;

use std::time::Instant;
use std::io::Read;

use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
    dpi::PhysicalPosition,
};
use bytemuck;
use cgmath::prelude::*;
use cgmath::{Vector3, Quaternion};

use camera::*;
use model::{VertexLayout, Vertex};
use texture::{Texture, DepthTexture};
use light::Light;
use particle::*;
use config::Config;
use gpu::Gpu;
use delta::Delta;
use io::new_input_file;
use gui::Gui;

struct Pipeline {
    particles: wgpu::RenderPipeline,
    light: wgpu::RenderPipeline,
}


struct State {
    gpu: Gpu,
    gui: Gui,
    camera: Camera,
    systems: Vec<ParticleSystem>,
    pipeline: Pipeline,
    diffuse_texture: Texture,
    depth_texture: DepthTexture,
    light: Light,
    delta: Delta,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: &Window, cfg: Config) -> Self {
        let gpu = Gpu::init(&window).await;
        let gui = Gui::new(&window, &gpu);
        let camera = Camera::new(&gpu.config, &gpu.device);
        let depth_texture = DepthTexture::new(&gpu.device, &gpu.config, "Depth Texture");

        let mut systems = Vec::new();

        for scfg in cfg.systems.iter() {
            systems.push(ParticleSystem::new(&gpu.device, scfg));
        }

        let fs_name = "fs_color";

        let mut diffuse_data = Vec::new();
        for (scfg, system) in cfg.systems.iter().zip(systems.iter_mut()) {
            if let Some(file) = &scfg.texture {
                let mut diffuse_file = new_input_file(&file).unwrap();
                diffuse_file.read_to_end(&mut diffuse_data).unwrap();
                let texture = Texture::new(&gpu.device, &gpu.queue, &diffuse_data, None).unwrap();
                system.texture(texture);
            }
        }

        let diffuse_bytes = include_bytes!("C:/Rust/pg/image/stone.png");
        let diffuse_texture = Texture::new(&gpu.device, &gpu.queue, diffuse_bytes, None).unwrap();


        let light = Light::new(&gpu.device);

        let light_shader = gpu.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("light.wgsl").into()),
            }
        );
        let light_pipeline_layout = gpu.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera.bind_layout,
                    &light.bind_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        
        let shader = gpu.device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("particle.wgsl").into()),
            }
        );
        let pipeline_layout = gpu.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &diffuse_texture.bind_layout,
                    &camera.bind_layout,
                    &light.bind_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let pipeline = Pipeline {
            particles: gpu.device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[
                            Vertex::layout(),
                            ParticleRaw::layout(),
                        ],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: &fs_name,
                        targets: &[
                            Some(wgpu::ColorTargetState {
                                format: gpu.config.format,
                                blend: Some(wgpu::BlendState::REPLACE),
                                write_mask: wgpu::ColorWrites::ALL,
                            })
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
                        format: texture::Texture::DEPTH_FORMAT,
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
            ),
            light: gpu.device.create_render_pipeline(
                &wgpu::RenderPipelineDescriptor {
                    label: Some("Light Render Pipeline"),
                    layout: Some(&light_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &light_shader,
                        entry_point: "vs_main",
                        buffers: &[
                            Vertex::layout(),
                        ],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &light_shader,
                        entry_point: "fs_main",
                        targets: &[
                            Some(wgpu::ColorTargetState {
                                format: gpu.config.format,
                                blend: Some(wgpu::BlendState::REPLACE),
                                write_mask: wgpu::ColorWrites::ALL,
                            })
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
                        format: texture::Texture::DEPTH_FORMAT,
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
            )
        };

        Self {
            gpu,
            pipeline,
            diffuse_texture,
            camera,
            depth_texture,
            light,
            systems,
            gui,
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

    fn update(&mut self, delta: f32) {
        let old_pos: Vector3<_> = self.light.uniform.position.into();
        let angle = Quaternion::from_axis_angle((0.0, 1.0, 0.0).into(), cgmath::Deg(1.0));
        self.light.uniform.position = (angle * old_pos).into();
        self.gpu.queue.write_buffer(&self.light.buffer, 0, bytemuck::cast_slice(&[self.light.uniform]));

        for system in self.systems.iter_mut() {
            system.update_particles(delta, &self.gpu.queue);
        }
        
        self.camera.update();
        self.camera.update_view_proj();
        self.gpu.queue.write_buffer(&self.camera.buffer, 0, bytemuck::cast_slice(&[self.camera.uniform]));
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.gpu.surface.get_current_texture().unwrap();
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

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
                    })
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


        // rpass.set_pipeline(&self.pipeline.light);

        // rpass.set_bind_group(0, &self.camera.bind_group, &[]);
        // rpass.set_bind_group(1, &self.light.bind_group, &[]);

        // rpass.set_vertex_buffer(0, self.generator.vbuf.slice(..));

        // rpass.set_index_buffer(self.generator.ibuf.slice(..), wgpu::IndexFormat::Uint16);

        // rpass.draw_indexed(0..self.generator.mesh.indices.len() as u32, 0, 0..self.generator.particle_count());

        
        rpass.set_pipeline(&self.pipeline.particles);

        rpass.set_bind_group(0, &self.diffuse_texture.bind_group, &[]);
        rpass.set_bind_group(1, &self.camera.bind_group, &[]);
        rpass.set_bind_group(2, &self.light.bind_group, &[]);

        for system in self.systems.iter() {
            rpass.set_vertex_buffer(0, system.vbuf.slice(..));
            rpass.set_vertex_buffer(1, system.particle_buf.slice(..));

            rpass.set_index_buffer(system.ibuf.slice(..), wgpu::IndexFormat::Uint16);
        
            rpass.draw_indexed(0..system.mesh.indices.len() as u32, 0, 0..system.particle_count());
        }
        
        drop(rpass);

        for system in self.systems.iter() {
            encoder.clear_buffer(&system.particle_buf, 0, system.particle_buf_size());
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
 
pub async fn run(cfg: Config) {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window, cfg).await;

    // let mut delta = Delta::new();

    let mut window_pos = PhysicalPosition::<f64>::new(0.0, 0.0);

    let mut last_cursor = None;
    let mut last_frame = Instant::now();
    let mut focused = 0;
    let mut particle_count = 0;
    let mut name = String::new();
    let mut particle_rate = 0;
    let mut position = [0.0, 0.0, 0.0];

    // Opens the window and starts processing events
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
    
        match event {
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
                        WindowEvent::CursorMoved { position, .. } => {
                            window_pos = *position;
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                let delta = (state.delta.from(Instant::now()).as_millis() as f32) / 1000.0;
                state.update(delta);
                match state.render() {
                    Ok(_) => {},
                    Err(wgpu::SurfaceError::Lost) => {
                        state.resize(state.gpu.size);
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        *control_flow = ControlFlow::Exit;
                    }
                    Err(e) => {
                        eprintln!("{e:?}");
                    }
                }
            }
            Event::RedrawEventsCleared => {
                let delta_s = last_frame.elapsed();
                let now = Instant::now();
                state.gui.context.io_mut().update_delta_time(now - last_frame);
                last_frame = now;

                // state.gui.context.io_mut().update_delta_time(delta.from(Instant::now()));

                let frame = match state.gpu.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        eprintln!("dropped frame: {:?}", e);
                        return;
                    }
                };
                state.gui.platform.prepare_frame(state.gui.context.io_mut(), &window).expect("Failed to prepare frame");
                let ui = state.gui.context.frame();
                {
                    ui.text(format!("Frametime: {:?}", delta_s));
                    ui.separator();
                    ui.columns(2, "", true);

                    if ui.button("New Particle System   ") {
                        state.systems.push(ParticleSystem::default(&state.gpu.device));
                    }
                    if ui.button("Delete Particle System") {
                        state.systems.remove(focused as usize);
                        if focused >= state.systems.len() as i32 {
                            focused = std::cmp::max(state.systems.len() as i32 - 1, 0);
                        }
                    }

                    ui.list_box(
                        "",
                        &mut focused,
                        &state.systems.iter().map(|s| s.name.as_str()).collect::<Vec<&str>>().as_slice(),
                        state.systems.len() as i32,
                    );

                    ui.next_column();

                    if !state.systems.is_empty() {
                        let sys = &mut state.systems[focused as usize];
                        particle_count = sys.particles.len() as i32;
                        name = sys.name.clone();
                        particle_rate = sys.particle_rate as i32;
                        position = sys.position.into();

                        if ui.input_text("Name", &mut name).enter_returns_true(true).build() {
                            sys.name = name.clone();
                        }
                        if ui.input_int("Max Particles", &mut particle_count).enter_returns_true(true).build() {
                            if particle_count > 0 {
                                sys.resize(particle_count, &state.gpu.device);
                            }
                        }
                        if ui.input_int("Particle Rate", &mut particle_rate).enter_returns_true(true).build() {
                            if particle_rate > 0 {
                                sys.particle_rate(particle_rate);
                            }
                        }
                        if ui.input_float3("Position", &mut position).enter_returns_true(true).build() {
                            sys.position(position);
                        }
                    }
                }

                let mut encoder: wgpu::CommandEncoder =
                    state.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

                if last_cursor != Some(ui.mouse_cursor()) {
                    last_cursor = Some(ui.mouse_cursor());
                    state.gui.platform.prepare_render(&ui, &window);
                }

                let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }
                            ),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                state.gui.renderer.render(ui.render(), &state.gpu.queue, &state.gpu.device, &mut rpass).expect("Rendering failed");

                drop(rpass);

                state.gpu.queue.submit(Some(encoder.finish()));

                frame.present();
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {},
        }
        state.gui.platform.handle_event(state.gui.context.io_mut(), &window, &event);
    });
}