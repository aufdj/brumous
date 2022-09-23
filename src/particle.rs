use std::mem;
use std::num::NonZeroU64;
use std::path::Path;
use std::io::Read;
use std::ops::Range;

use cgmath::{Vector3, Vector4, Matrix3, Matrix4, Quaternion, SquareMatrix};
use wgpu::util::DeviceExt;
use bytemuck;

use crate::model::{Vertex, VertexLayout};
use crate::random::Randf32;
use crate::texture::Texture;
use crate::gpu::Gpu;
use crate::bufio::new_input_file;

#[derive(Default, Copy, Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl From<[f32; 3]> for Position {
    fn from(pos: [f32; 3]) -> Self {
        Self {
            x: pos[0],
            y: pos[1],
            z: pos[2],
        }
    }
}
impl From<Position> for [f32; 3] {
    fn from(pos: Position) -> Self {
        [pos.x, pos.y, pos.z]
    }
}

pub enum Area {
    Cube([Range<f32>; 3]),
}
impl Default for Area {
    fn default() -> Self {
        Self::Cube([-0.2..0.2, 0.0..0.0, -0.2..0.2])
    }
}

const IDENTITY: [[f32; 4]; 4] = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 0.0]
];


pub struct ParticleSystem {
    pub particles:      Vec<Particle>,
    pub mesh:           ParticleMesh,
    pub vbuf:           wgpu::Buffer,
    pub ibuf:           wgpu::Buffer,
    pub particle_buf:   wgpu::Buffer,
    last_used_particle: usize,
    particle_rate:      usize,
    position:           Position,
    texture:            Option<Texture>,
    name:               String,
    gravity:            f32,
    settings:           ParticleSettings,
    pipeline:           wgpu::RenderPipeline,
    cam_buffer:         wgpu::Buffer,
    cam_bind_group:     wgpu::BindGroup,
}
impl ParticleSystem {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let particles = vec![Particle::default(); 5000];
        let mesh = ParticleMesh::cube();

        let vbuf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let ibuf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Vertex Buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let particle_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&particles.to_raw()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let cam_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&IDENTITY),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let cam_bind_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ]
            }
        );
        let cam_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Camera Bind Group"),
                layout: &cam_bind_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: cam_buffer.as_entire_binding(),
                    }
                ]
            }
        );

        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("particle.wgsl").into()),
            }
        );

        let pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &cam_bind_layout,
                ],
                push_constant_ranges: &[],
            }
        );

        let pipeline = device.create_render_pipeline(
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
                    entry_point: "fs_color",
                    targets: &[
                        Some(wgpu::ColorTargetState {
                            format: config.format,
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
            particles,
            mesh,
            vbuf,
            ibuf,
            particle_buf,
            last_used_particle: 0,
            particle_rate: 1,
            position: Position::default(),
            texture: None,
            name: String::from("Particle System"),
            gravity: -9.81,
            settings: ParticleSettings::default(),
            pipeline,
            cam_buffer,
            cam_bind_group,
        }
    }
    fn find_unused_particle(&mut self) -> usize {
        for i in self.last_used_particle..self.particles.len() {
            if self.particles[i].life < 0.0 {
                self.last_used_particle = i;
                return i;
            }
        }

        for i in 0..self.last_used_particle {
            if self.particles[i].life < 0.0 {
                self.last_used_particle = i;
                return i;
            }
        }
        self.last_used_particle = 0;
        0
    }
    pub fn update_particles(&mut self, delta: f32, queue: &wgpu::Queue) {
        for _ in 0..self.particle_rate {
            let particle = self.find_unused_particle();
            self.particles[particle] = Particle::from(&mut self.settings);
        }

        for (index, particle) in self.particles.iter_mut().enumerate() {
            particle.life -= delta;
            if particle.life > 0.0 {
                particle.update(delta, self.gravity);
                queue.write_buffer(
                    &self.particle_buf,
                    index as u64 * ParticleRaw::size(),
                    bytemuck::cast_slice(&[particle.to_raw()])
                );
            }
        }
        if self.last_used_particle > self.particles.len() {
            self.last_used_particle = self.particles.len() - 1;
        }
    }
    pub fn resize(&mut self, count: i32, device: &wgpu::Device) {
        self.particles.resize(count as usize, Particle::default());
        self.particle_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Buffer"),
                contents: bytemuck::cast_slice(&self.particles.to_raw()),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
    }
    pub fn particle_count(&self) -> u32 {
        self.particles.len() as u32
    }
    pub fn particle_buf_size(&self) -> Option<NonZeroU64> {
        NonZeroU64::new(self.particles.len() as u64 * ParticleRaw::size())
    }
    pub fn set_position(&mut self, pos: [f32; 3]) {
        self.position = Position::from(pos);
    }
    pub fn set_texture(&mut self, gpu: &Gpu, texture_path: &Path) {
        let mut diffuse_data = Vec::new();
        new_input_file(&texture_path).unwrap().read_to_end(&mut diffuse_data).unwrap();
        let texture = Texture::new(&gpu.device, &gpu.queue, &diffuse_data, None).unwrap();
        self.texture = Some(texture);
    }
    pub fn set_particle_rate(&mut self, particle_rate: usize) {
        self.particle_rate = particle_rate;
    }
    pub fn set_gravity(&mut self, gravity: f32) {
        self.gravity = gravity;
    }
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    pub fn set_weight(&mut self, weight: Range<f32>) {
        self.settings.weight = weight;
    }
    pub fn set_initial_velocity(&mut self, init_vel: [Range<f32>; 3]) {
        self.settings.init_vel = init_vel;
    }
    pub fn set_area(&mut self, area: Area) {
        self.settings.area = area;
    }
    pub fn set_view_proj(&mut self, view_proj: [[f32; 4]; 4], queue: &wgpu::Queue) {
        queue.write_buffer(&self.cam_buffer, 0, bytemuck::cast_slice(&[view_proj]));
    }
    pub fn clear(&mut self, encoder: &mut wgpu::CommandEncoder) {
        encoder.clear_buffer(&self.particle_buf, 0, self.particle_buf_size());
    }
}

struct ParticleSettings {
    pos:      Position,
    area:     Area,
    init_vel: [Range<f32>; 3],
    color:    [Range<f32>; 4],
    life:     Range<f32>,
    weight:   Range<f32>,
    scale:    Range<f32>,
    rand:     Randf32,
}
impl Default for ParticleSettings {
    fn default() -> Self {
        Self {
            pos:      Position::from([0.0, 0.0, 0.0]),
            area:     Area::default(),
            life:     1.0..10.0,
            init_vel: [-0.2..0.2, 0.5..1.0, -0.2..0.2],
            color:    [0.0..1.0, 0.0..1.0, 0.0..1.0, 0.0..1.0],
            weight:   0.1..1.0,
            scale:    0.005..0.010,
            rand:     Randf32::new(),
        }
    }
}

#[derive(Clone)]
pub struct Particle {
    pub pos:    Vector3<f32>,
    pub vel:    Vector3<f32>,
    pub rot:    Quaternion<f32>,
    pub scale:  f32,
    pub life:   f32,
    pub weight: f32,
    pub color:  Vector4<f32>,
}
impl Particle {
    pub fn update(&mut self, delta: f32, gravity: f32) {
        self.vel += Vector3::new(0.0, gravity * self.weight, 0.0) * delta * 0.5;
        self.pos += self.vel * delta;
    }
    pub fn to_raw(&self) -> ParticleRaw {
        ParticleRaw {
            model: (
                Matrix4::from_translation(self.pos) *
                Matrix4::from(self.rot) *
                Matrix4::from_scale(self.scale)
            ).into(),
            normal: Matrix3::from(self.rot).into(),
            color: self.color.into(),
        }
    }
}
impl Default for Particle {
    fn default() -> Self {
        Self {
            pos: Vector3::new(0.0, -100.0, 0.0), 
            rot: Quaternion::new(0.0, 0.0, 0.0, 0.0), 
            vel: Vector3::new(0.0, 0.0, 0.0), 
            scale: 0.008,
            life: 0.0, 
            weight: 1.0,
            color: Vector4::new(0.0, 0.0, 0.0, 0.0),
        }
    }
}
impl From<&mut ParticleSettings> for Particle {
    fn from(settings: &mut ParticleSettings) -> Particle {
        let pos = match &settings.area {
            Area::Cube(dim) => {
                let s1 = settings.rand.in_range(&dim[0]);
                let s2 = settings.rand.in_range(&dim[1]);
                let s3 = settings.rand.in_range(&dim[2]);
                Vector3::new(s1 + settings.pos.x, s2 + settings.pos.y, s3 + settings.pos.z)
            }
        };

        let v1 = settings.rand.in_range(&settings.init_vel[0]);
        let v2 = settings.rand.in_range(&settings.init_vel[1]);
        let v3 = settings.rand.in_range(&settings.init_vel[2]);
        let vel = Vector3::new(v1, v2, v3);

        let r = settings.rand.in_range(&settings.color[0]);
        let g = settings.rand.in_range(&settings.color[1]);
        let b = settings.rand.in_range(&settings.color[2]);
        let a = settings.rand.in_range(&settings.color[3]);
        let color = Vector4::new(a, r, g, b);

        let life = settings.rand.in_range(&settings.life);
        let weight = settings.rand.in_range(&settings.weight);
        let scale = settings.rand.in_range(&settings.scale);

        Particle {
            pos,
            vel,
            color,
            life,
            scale,
            rot: Quaternion::new(0.0, 0.0, 0.0, 0.0),
            weight,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleRaw {
    model:  [[f32; 4]; 4],
    normal: [[f32; 3]; 3],
    color:  [f32; 4],
}
impl ParticleRaw {
    pub fn size() -> u64 {
        mem::size_of::<Self>() as u64
    }
}
impl VertexLayout for ParticleRaw {
    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ParticleRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 16]>() as wgpu::BufferAddress,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 19]>() as wgpu::BufferAddress,
                    shader_location: 10,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 22]>() as wgpu::BufferAddress,
                    shader_location: 11,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 26]>() as wgpu::BufferAddress,
                    shader_location: 12,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

pub trait ToRaw {
    fn to_raw(&self) -> Vec<ParticleRaw>;
}
impl ToRaw for Vec<Particle> {
    fn to_raw(&self) -> Vec<ParticleRaw> {
        self.iter().map(Particle::to_raw).collect::<Vec<ParticleRaw>>()
    }
}

#[derive(Clone, Debug)]
pub struct ParticleMesh {
    pub vertices:  Vec<Vertex>,
    pub indices:   Vec<u16>,
}
impl ParticleMesh {
    pub fn cube() -> Self {
        let vertices = vec![
            // top (0, 0, 1)
            Vertex { position: [-1.0, -1.0,  1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [ 1.0, -1.0,  1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [ 1.0,  1.0,  1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [-1.0,  1.0,  1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
            // bottom (0, 0, -1)
            Vertex { position: [-1.0,  1.0, -1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [ 1.0,  1.0, -1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [ 1.0, -1.0, -1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
            // right (1, 0, 0)
            Vertex { position: [ 1.0, -1.0, -1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [ 1.0,  1.0, -1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [ 1.0,  1.0,  1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [ 1.0, -1.0,  1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
            // left (-1, 0, 0)
            Vertex { position: [-1.0, -1.0,  1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [-1.0,  1.0,  1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [-1.0,  1.0, -1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
            // front (0, 1, 0)
            Vertex { position: [ 1.0,  1.0, -1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [-1.0,  1.0, -1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [-1.0,  1.0,  1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [ 1.0,  1.0,  1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
            // back (0, -1, 0)
            Vertex { position: [ 1.0, -1.0,  1.0], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [-1.0, -1.0,  1.0], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] },
            Vertex { position: [ 1.0, -1.0, -1.0], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] },
        ];
        let indices = vec![
            0u16, 1,  2,  2,  3,  0, // top
            4,    5,  6,  6,  7,  4, // bottom
            8,    9, 10, 10, 11,  8, // right
            12,  13, 14, 14, 15, 12, // left
            16,  17, 18, 18, 19, 16, // front
            20,  21, 22, 22, 23, 20, // back
        ];
        Self {
            vertices, indices,
        }
    }
}

pub trait DrawParticleSystem<'a, 'b> where 'a: 'b {
    fn draw_particle_system(&'b mut self, sys: &'a ParticleSystem);
}
impl<'a, 'b> DrawParticleSystem<'a, 'b> for wgpu::RenderPass<'a> where 'a: 'b {
    fn draw_particle_system(&'b mut self, sys: &'a ParticleSystem) {
        self.set_pipeline(&sys.pipeline);
        self.set_bind_group(0, &sys.cam_bind_group, &[]);
        self.set_vertex_buffer(0, sys.vbuf.slice(..));
        self.set_vertex_buffer(1, sys.particle_buf.slice(..));

        self.set_index_buffer(sys.ibuf.slice(..), wgpu::IndexFormat::Uint16);
        
        self.draw_indexed(0..sys.mesh.indices.len() as u32, 0, 0..sys.particle_count());
    }
}