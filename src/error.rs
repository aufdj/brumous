use std::io;
use std::fmt;
use std::path::PathBuf;

pub type BrumousResult<T> = Result<T, BrumousError>;

pub enum BrumousError {
    IoError(io::Error),
    ObjParseError(PathBuf, usize),
}
impl From<io::Error> for BrumousError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}
impl fmt::Display for BrumousError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrumousError::IoError(err) => {
                write!(f, "
                    \r{err}"
                )
            }
            BrumousError::ObjParseError(path, line) => {
                write!(f, "
                    \rError parsing file {}: line {line}",
                    path.display()
                )
            }
        }
    }
}
