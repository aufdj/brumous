use std::{
    fmt,
    path::PathBuf,
};

use crate::{
    error::ConfigError,
    particle::ParticleMesh,
};


/// Parsing states.
enum Parse {
    None,
    NewGenerator,
    ParticleMax,
    ParticleRate,
    Texture,
    Mesh,
}


/// User defined configuration settings.
#[derive(Clone, Debug)]
pub struct Config {
    pub systems: Vec<SystemConfig>,
}
impl Config {
    /// Create a new Config with the specified command line arguments.
    pub fn new(args: Vec<String>) -> Result<Config, ConfigError> {
        let mut parser = Parse::None;
        let mut cfg    = Config::default();
        let mut scfg   = SystemConfig::default();
        
        for arg in args.into_iter() {
            match arg.as_str() {
                "g" => {
                    parser = Parse::NewGenerator;
                }
                "-max" => {
                    parser = Parse::ParticleMax;
                    continue;
                }
                "-rate" => {
                    parser = Parse::ParticleRate;
                    continue;
                }
                "-mesh" => {
                    parser = Parse::Mesh;
                    continue;
                }
                "-texture" => {
                    parser = Parse::Texture;
                    continue;
                }
                _ => {},
            }
            match parser {
                Parse::NewGenerator => {
                    cfg.systems.push(scfg);
                    scfg = SystemConfig::default();
                }
                Parse::Texture => {
                    let path = PathBuf::from(&arg);
                    if path.exists() {
                        scfg.texture = Some(path);
                    }
                    else {
                        return Err(
                            ConfigError::NotFound(arg.clone())
                        );
                    }
                }
                Parse::ParticleMax => {
                    if let Ok(max) = arg.parse::<usize>() {
                        scfg.max = max;   
                    }
                    else {
                        return Err(
                            ConfigError::InvalidParticleMax(arg.clone())
                        );
                    }
                }
                Parse::ParticleRate => {
                    if let Ok(rate) = arg.parse::<usize>() {
                        scfg.rate = rate;  
                    }
                    else {
                        return Err(
                            ConfigError::InvalidParticleRate(arg.clone())
                        );
                    }
                }
                Parse::Mesh => {
                    if arg.as_str() == "cube" {
                        scfg.mesh = ParticleMesh::cube();
                    }
                    else {
                        return Err(
                            ConfigError::InvalidParticleMesh(arg.clone())
                        );
                    }
                }
                Parse::None => {},
            }
        } 
        if cfg.systems.is_empty() {
            cfg.systems.push(scfg);
        }
        Ok(cfg)
    }
}
impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "
            \rParticle Systems
            \r=============================================================",
        )?;
        for (i, scfg) in self.systems.iter().enumerate() {
            write!(f, "
                \rSystem {i}:
                \r{scfg}",
            )?;
        }
        writeln!(f, "")
    }
}
impl Default for Config {
    fn default() -> Self {
        Self {
            systems: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SystemConfig {
    pub max: usize,
    pub rate: usize,
    pub mesh: ParticleMesh,
    pub texture: Option<PathBuf>,
    pub pos: (f32, f32),
}
impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            max: 5000,
            rate: 10,
            mesh: ParticleMesh::cube(),
            texture: None,
            pos: (0.0, 0.0),
        }
    }
}
impl fmt::Display for SystemConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "
            \rMax Particles: {},
            \rParticle Rate: {},
            \rPosition: {}, {}
            \rTexture: {}",
            self.max,
            self.rate,
            self.pos.0,
            self.pos.1,
            if let Some(t) = &self.texture {
                t.to_str().unwrap()
            }
            else {
                "None"
            },
        )
    }
}

