use std::fs;
use std::io::Error;
use std::path::Path;

pub fn read_bin_file(path: impl AsRef<Path>) -> Result<Vec<u8>, Error> {
    fs::read(path)
}