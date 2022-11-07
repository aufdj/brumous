use std::io;
use std::fmt;
use std::num;
use image::error::ImageError;

pub type BrumousResult<T> = Result<T, BrumousError>;

#[derive(Debug)]
pub enum BrumousError {
    FileError(io::Error),
    ObjInvalidInt(num::ParseIntError),
    ObjInvalidFloat(num::ParseFloatError),
    OpenTexture(String, io::Error),
    LoadTexture(String, ImageError),
}

impl From<io::Error> for BrumousError {
    fn from(err: io::Error) -> Self {
        Self::FileError(err)
    }
}

impl From<num::ParseIntError> for BrumousError {
    fn from(err: num::ParseIntError) -> Self {
        Self::ObjInvalidInt(err)
    }
}

impl From<num::ParseFloatError> for BrumousError {
    fn from(err: num::ParseFloatError) -> Self {
        Self::ObjInvalidFloat(err)
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
            BrumousError::ObjInvalidInt(err) => {
                write!(f, "
                    \r{err}",
                )
            }
            BrumousError::ObjInvalidFloat(err) => {
                write!(f, "
                    \r{err}",
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
