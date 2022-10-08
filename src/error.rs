use std::io;

pub type BrumousResult<T> = Result<T, BrumousError>;

pub enum BrumousError {
    IoError(io::Error),
}
impl From<io::Error> for BrumousError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}
