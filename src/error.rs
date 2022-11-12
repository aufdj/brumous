use std::io;
use std::fmt;
use image::error::ImageError;

pub type BrumousResult<T> = Result<T, BrumousError>;

#[derive(Debug)]
pub enum BrumousError {
    FileError(io::Error),
    ParseInt(String, usize),
    ParseFloat(String, usize),
    InvalidVertexData(String, usize),
    OpenTexture(String, io::Error),
    LoadTexture(String, ImageError),
}

impl From<io::Error> for BrumousError {
    fn from(err: io::Error) -> Self {
        Self::FileError(err)
    }
}

impl fmt::Display for BrumousError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrumousError::FileError(err) => {
                write!(f, "
                    \r{err}",
                )
            }
            BrumousError::ParseInt(path, line) => {
                write!(f, "
                    \rError parsing int on line {line} of file {path}",
                )
            }
            BrumousError::ParseFloat(path, line) => {
                write!(f, "
                    \rError parsing float on line {line} of file {path}",
                )
            }
            BrumousError::InvalidVertexData(path, line) => {
                write!(f, "
                    \rInvalid vertex data: line {line} of file {path}",
                )
            }
            BrumousError::OpenTexture(path, err) => {
                write!(f, "
                    \rError opening file {path}:
                    \r{err}",
                )
            }
            BrumousError::LoadTexture(path, err) => {
                write!(f, "
                    \rError loading texture {path}:
                    \r{err}",
                )
            }
        }
    }
}
