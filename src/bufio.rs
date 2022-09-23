use std::{
    fs::File,
    path::Path,
    io::{self, BufReader},
};

/// Takes a file path and returns an input file wrapped in a BufReader.
pub fn new_input_file(path: &Path) -> io::Result<BufReader<File>> {
    match File::open(path) {
        Ok(file) => {
            Ok(BufReader::with_capacity(4096, file))
        }
        Err(err) => {
            Err(err)
        }
    }
}