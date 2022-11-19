use std::fs;

use wgpu::util::DeviceExt;

use crate::particle::{ParticleMesh, ParticleVertex};
use crate::ParticleMeshType;
use crate::error::{BrumousResult, BrumousError};

const CUBE_OBJ: &str = include_str!("../obj/cube.obj");
const POINT_OBJ: &str = include_str!("../obj/point.obj");
  
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
        ParticleMeshType::Point => {
            parse_obj_file(device, POINT_OBJ, "point.obj")
        }
    }
}

pub fn parse_obj_file(device: &wgpu::Device, data: &str, path: &str) -> BrumousResult<ParticleMesh> {
    let path = path.to_string();
    let mut vertices = Vec::<ParticleVertex>::new();
    let indices = Vec::<u16>::new();

    let mut v  = Vec::<[f32; 3]>::new(); // Positions
    let mut vt = Vec::<[f32; 2]>::new(); // Texture coordinates
    let mut vn = Vec::<[f32; 3]>::new(); // Normals

    let mut floats = Vec::new();
    let mut num = String::new();

    for (line, count) in data.lines().zip(1..) {
        let mut string = line.split_whitespace();
        match string.next() {
            Some("v") => {
                for s in string {
                    floats.push(
                        s.parse::<f32>().map_err(|_| 
                            BrumousError::ParseFloat(path.to_string(), count)
                        )?
                    );
                }
                v.push(
                    floats[..3].try_into().map_err(|_| 
                        BrumousError::InvalidVertexData(path.to_string(), count)
                    )?
                );
            }
            Some("vt") => {
                for s in string {
                    floats.push(
                        s.parse::<f32>().map_err(|_| 
                            BrumousError::ParseFloat(path.to_string(), count)
                        )?
                    );
                }
                vt.push(
                    floats[..2].try_into().map_err(|_| 
                        BrumousError::InvalidVertexData(path.to_string(), count)
                    )?
                );
            }
            Some("vn") => {
                for s in string {
                    floats.push(
                        s.parse::<f32>().map_err(|_| 
                            BrumousError::ParseFloat(path.to_string(), count)
                        )?
                    );
                }
                vn.push(
                    floats[..3].try_into().map_err(|_| 
                        BrumousError::InvalidVertexData(path.to_string(), count)
                    )?
                );
            }
            Some("f") => {
                for s in string {
                    let mut parse = Vertex::Position;
                    let mut vertex = ParticleVertex::default();
                    for c in s.chars() {
                        if c == '/' {
                            if !num.is_empty() {
                                let n = num.parse::<i32>().map_err(|_| 
                                    BrumousError::ParseInt(path.to_string(), count+1)
                                )?;
    
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
                                num.clear();
                            }
                            parse.next();
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
                