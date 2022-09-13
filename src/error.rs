use std::{
    fmt,
    io,
    path::PathBuf,
};

/// Possible errors encountered while parsing Config arguments.
#[derive(Debug)]
pub enum ConfigError {
    InvalidParticleRate(String),
    InvalidParticleMax(String),
    InvalidGeneratorPos(String),
    InvalidParticleMesh(String),
    TextureNotFound(String),
    NotFound(String),
    AccessDenied(PathBuf),
    IoError(io::Error),
}
impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> ConfigError {
        ConfigError::IoError(err)
    }
}
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::InvalidParticleRate(count) => {
                write!(f,  "
                    \r{count} is not a valid particle rate.\n"
                )
            }
            ConfigError::InvalidParticleMax(count) => {
                write!(f,  "
                    \r{count} is not a valid particle max.\n"
                )
            }
            ConfigError::InvalidGeneratorPos(pos) => {
                write!(f,  "
                    \r{pos} is not a valid generator position.\n"
                )
            }
            ConfigError::InvalidParticleMesh(mesh) => {
                write!(f,  "
                    \r{mesh} is not a valid particle mesh.\n"
                )
            }
            ConfigError::TextureNotFound(path) => {
                write!(f,  "
                    \rTexture {path} not found.\n"
                )
            }
            ConfigError::NotFound(path) => {
                write!(f, "
                    \rFile {path} not found.\n"
                )
            }
            ConfigError::AccessDenied(path) => {
                write!(f, "
                    \rAccess to {} denied.\n",
                    path.display(),
                )
            }
            ConfigError::IoError(err) => {
                write!(f, "
                    \r{err}\n",
                )
            }
        }
    }
}