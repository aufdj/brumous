use std::fs;

use wgpu::util::DeviceExt;

use crate::particle::{ParticleMesh, ParticleVertex};
use crate::ParticleMeshType;
use crate::error::{BrumousResult, BrumousError};

const CUBE_OBJ: &str = include_str!("../obj/cube.obj");
  
enum Vertex {
    Position,
    TexCoords,
    Normal,
}
impl Vertex {
    fn next(&mut self) {
        *self = match self {
            Vertex::Position  => Vertex::TexCoords,
            Vertex::TexCoords => Vertex::Normal,
            Vertex::Normal    => Vertex::Position,
        }
    }
}

pub fn read_obj_file(device: &wgpu::Device, mesh_type: &ParticleMeshType) -> BrumousResult<ParticleMesh> {
    match mesh_type {
        ParticleMeshType::Custom(path) => {
            let data = fs::read_to_string(path)?;
            parse_obj_file(device, &data, path)
        }
        ParticleMeshType::Cube => {
            parse_obj_file(device, CUBE_OBJ, "cube.obj")
        }
    }
}

pub fn parse_obj_file(device: &wgpu::Device, data: &str, _path: &str) -> BrumousResult<ParticleMesh> {
    let mut vertices = Vec::<ParticleVertex>::new();
    let indices = Vec::<u16>::new();

    let mut v  = Vec::<[f32; 3]>::new(); // Positions
    let mut vt = Vec::<[f32; 2]>::new(); // Texture coordinates
    let mut vn = Vec::<[f32; 3]>::new(); // Normals

    let mut floats = Vec::new(); 
    let mut num = String::new();

    for (linecount, line) in data.lines().enumerate() {
        let mut string = line.split_whitespace();
        match string.next() {
            Some("v") => {
                while let Some(s) = string.next() {
                    floats.push(s.parse::<f32>()?);
                }
                v.push(floats[..3].try_into().unwrap());
            }
            Some("vt") => {
                while let Some(s) = string.next() {
                    floats.push(s.parse::<f32>()?);
                }
                vt.push(floats[..2].try_into().unwrap());
            }
            Some("vn") => {
                while let Some(s) = string.next() {
                    floats.push(s.parse::<f32>()?);
                }
                vn.push(floats[..3].try_into().unwrap());
            }
            Some("f") => {
                while let Some(s) = string.next() {
                    let mut parse = Vertex::Position;
                    let mut vertex = ParticleVertex::default();
                    for c in s.chars() {
                        if c == '/' {
                            let n = num.parse::<i32>()?;
                            let i = n as usize - 1;
                            match parse {
                                Vertex::Position => {
                                    vertex.position = v[i];
                                }
                                Vertex::TexCoords => {
                                    vertex.tex_coords = vt[i];
                                }
                                Vertex::Normal => {
                                    vertex.normal = vn[i];
                                }
                            }
                            parse.next();
                            num.clear();
                        }
                        else {
                            num.push(c);
                        }
                    }
                    vertices.push(vertex);
                    num.clear();
                }
            }
            _ => {}
        }
        floats.clear();
    }

    let vertex_count = vertices.len() as u32;
    let index_count = indices.len() as u32;

    let vertex_buf = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Particle Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        }
    );
    
    let index_buf = if index_count > 0 {
        Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Particle Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        ))
    }
    else {
        None
    };

    Ok(
        ParticleMesh { 
            vertex_buf, 
            index_buf, 
            vertex_count, 
            index_count 
        }
    )
}
                