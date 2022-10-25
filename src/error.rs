use std::io;
use std::fmt;
use std::path::PathBuf;

pub type BrumousResult<T> = Result<T, BrumousError>;

#[derive(Debug)]
pub enum BrumousError {
    FileOpenError(String, io::Error),
    FileReadError(String, io::Error),
    ObjParseError(String, usize),
    LoadTextureError(String),
}
impl From<io::Error> for BrumousError {
    fn from(err: io::Error) -> Self {
        Self::FileReadError(String::new(), err)
    }
}
impl fmt::Display for BrumousError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrumousError::FileOpenError(path, err) => {
                write!(f, "
                    \r{path}
                    \r{err}",
                )
            }
            BrumousError::FileReadError(path, err) => {
                write!(f, "
                    \r{path}
                    \r{err}",
                )
            }
            BrumousError::ObjParseError(path, line) => {
                write!(f, "
                    \rError parsing file {path}: line {line}",
                )
            }
            BrumousError::LoadTextureError(path) => {
                write!(f, "
                    \rError loading texture {path}",
                )
            }
        }
    }
}
