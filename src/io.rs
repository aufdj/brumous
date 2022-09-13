use std::{
    fs::File,
    path::Path,
    io::{BufReader, ErrorKind},
};

use crate::error::ConfigError;


/// Takes a file path and returns an input file wrapped in a BufReader.
pub fn new_input_file(path: &Path) -> Result<BufReader<File>, ConfigError> {
    match File::open(path) {
        Ok(file) => Ok(BufReader::with_capacity(4096, file)),
        Err(err) => {
            match err.kind() {
                ErrorKind::PermissionDenied => {
                    return Err(
                        ConfigError::AccessDenied(path.to_path_buf())
                    );
                }
                _ => {
                    return Err(
                        ConfigError::IoError(err)
                    );
                }
            }
        }
    }
}