use std::io::{BufRead, BufReader};
use std::fs::File;
use std::path::Path;

use wgpu::util::DeviceExt;

use crate::particle::{ParticleMesh, ParticleVertex};
use crate::bufio::new_input_file;
use crate::error::BrumousResult;
  
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

fn f32x3(vec: &mut Vec<f32>) -> [f32; 3] {
    let mut a = [0.0; 3];
    for i in 0..3 {
        a[i] = vec[i];
    }
    a
}
fn f32x2(vec: &mut Vec<f32>) -> [f32; 2] {
    let mut a = [0.0; 2];
    for i in 0..2 {
        a[i] = vec[i];
    }
    a
}

pub fn read_obj(device: &wgpu::Device, path: &Path) -> BrumousResult<ParticleMesh> {
    let mut file = new_input_file(path)?;

    let mut vertices = Vec::<ParticleVertex>::new();
    let mut indices = Vec::<u16>::new();

    let mut v  = Vec::<[f32; 3]>::new(); // Positions
    let mut vt = Vec::<[f32; 2]>::new(); // Texture coordinates
    let mut vn = Vec::<[f32; 3]>::new(); // Normals

    let mut line = String::new(); // Current line of obj file
    let mut floats = Vec::new();
    let mut num = String::new();

    while file.read_line(&mut line)? != 0 {
        let mut string = line.split_whitespace();
        match string.next() {
            Some("v") => {
                while let Some(s) = string.next() {
                    let f = s.parse::<f32>().unwrap();
                    floats.push(f);
                }
                v.push(f32x3(&mut floats));
            }
            Some("vt") => {
                while let Some(s) = string.next() {
                    let f = s.parse::<f32>().unwrap();
                    floats.push(f);
                }
                vt.push(f32x2(&mut floats));
            }
            Some("vn") => {
                while let Some(s) = string.next() {
                    let f = s.parse::<f32>().unwrap();
                    floats.push(f);
                }
                vn.push(f32x3(&mut floats));
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
                                panic!("Error");
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
                