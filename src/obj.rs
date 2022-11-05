use std::io::BufRead;
use std::path::Path;

use wgpu::util::DeviceExt;

use crate::particle::{ParticleMesh, ParticleVertex};
use crate::bufio::new_input_file;
use crate::error::{BrumousResult, BrumousError};
  
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

pub fn read_obj_file(device: &wgpu::Device, path: &str) -> BrumousResult<ParticleMesh> {
    let mut file = match new_input_file(Path::new(path)) {
        Ok(f) => {
            f
        }
        Err(e) => {
            return Err(
                BrumousError::FileOpenError(path.to_string(), e)
            );
        }
    };

    let mut vertices = Vec::<ParticleVertex>::new();
    let indices = Vec::<u16>::new();

    let mut v  = Vec::<[f32; 3]>::new(); // Positions
    let mut vt = Vec::<[f32; 2]>::new(); // Texture coordinates
    let mut vn = Vec::<[f32; 3]>::new(); // Normals

    let mut line = String::new(); // Current line of obj file
    let mut linecount = 0usize;
    let mut floats = Vec::new(); 
    let mut num = String::new();

    while file.read_line(&mut line)? != 0 {
        linecount += 1;
        let mut string = line.split_whitespace();
        match string.next() {
            Some("v") => {
                while let Some(s) = string.next() {
                    if let Ok(f) = s.parse::<f32>() {
                        floats.push(f);
                    }
                    else {
                        return Err(
                            BrumousError::ObjParseError(
                                path.to_string(), 
                                linecount
                            )
                        );
                    }
                }
                v.push(floats[..3].try_into().unwrap());
            }
            Some("vt") => {
                while let Some(s) = string.next() {
                    if let Ok(f) = s.parse::<f32>() {
                        floats.push(f);
                    }
                    else {
                        return Err(
                            BrumousError::ObjParseError(
                                path.to_string(), 
                                linecount
                            )
                        );
                    }
                }
                vt.push(floats[..2].try_into().unwrap());
            }
            Some("vn") => {
                while let Some(s) = string.next() {
                    if let Ok(f) = s.parse::<f32>() {
                        floats.push(f);
                    }
                    else {
                        return Err(
                            BrumousError::ObjParseError(
                                path.to_string(), 
                                linecount
                            )
                        );
                    }
                }
                vn.push(floats[..3].try_into().unwrap());
            }
            Some("f") => {
                while let Some(s) = string.next() {
                    let mut parse = Vertex::Position;
                    let mut vertex = ParticleVertex::default();
                    for c in s.chars() {
                        if c == '/' {
                            if let Ok(n) = num.parse::<i32>() {
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
                            }
                            else {
                                return Err(
                                    BrumousError::ObjParseError(
                                        path.to_string(), 
                                        linecount
                                    )
                                );
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
        line.clear();
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

pub fn read_obj_str(device: &wgpu::Device, s: &str) -> ParticleMesh {
    let mut vertices = Vec::<ParticleVertex>::new();
    let indices = Vec::<u16>::new();

    let mut v  = Vec::<[f32; 3]>::new(); // Positions
    let mut vt = Vec::<[f32; 2]>::new(); // Texture coordinates
    let mut vn = Vec::<[f32; 3]>::new(); // Normals

    let mut floats = Vec::new(); 
    let mut num = String::new();

    for line in s.lines() {
        let mut string = line.split_whitespace();
        match string.next() {
            Some("v") => {
                while let Some(s) = string.next() {
                    floats.push(s.parse::<f32>().unwrap());
                }
                v.push(floats[..3].try_into().unwrap());
            }
            Some("vt") => {
                while let Some(s) = string.next() {
                    floats.push(s.parse::<f32>().unwrap());
                }
                vt.push(floats[..2].try_into().unwrap());
            }
            Some("vn") => {
                while let Some(s) = string.next() {
                    floats.push(s.parse::<f32>().unwrap());
                }
                vn.push(floats[..3].try_into().unwrap());
            }
            Some("f") => {
                while let Some(s) = string.next() {
                    let mut parse = Vertex::Position;
                    let mut vertex = ParticleVertex::default();
                    for c in s.chars() {
                        if c == '/' {
                            let n = num.parse::<i32>().unwrap();
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

    ParticleMesh { 
        vertex_buf, 
        index_buf, 
        vertex_count, 
        index_count 
    }
}
                