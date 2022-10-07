use std::io::{BufRead, BufReader};
use std::fs::File;

use crate::particle::ParticleVertex;
  
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


pub struct ObjFile {
    pub vertices: Vec<ParticleVertex>,
    pub indices: Vec<u16>,
}

pub trait ReadObjFile {
    fn read_obj(&mut self) -> ObjFile;
}
impl ReadObjFile for BufReader<File> {
    fn read_obj(&mut self) -> ObjFile {
        let mut vertices: Vec<ParticleVertex> = Vec::new();
        let mut indices: Vec<u16> = Vec::new();

        let mut v  = Vec::<[f32; 3]>::new(); // Positions
        let mut vt = Vec::<[f32; 2]>::new(); // Texture coordinates
        let mut vn = Vec::<[f32; 3]>::new(); // Normals
        let mut line = String::new(); // Current line of obj file
        let mut floats = Vec::new();

        while self.read_line(&mut line).unwrap() != 0 {
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
                        let mut num = String::new();
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
                                    num.clear();
                                }
                                else {
                                    panic!("Error");
                                }
                                parse.next();
                            }
                            else {
                                num.push(c);
                            }
                        }
                        vertices.push(vertex);
                    }
                }
                _ => {}
            }
            line.clear();
            floats.clear();
        }
        ObjFile { vertices, indices }
    }
}
                