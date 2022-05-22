use std::fs;
use std::fs::File;
use std::io::{Read, Write, BufRead, BufReader};
use std::path::PathBuf;

pub fn read(path: PathBuf) -> Result<String, String> {
    let mut f = match File::open(path) {
        Ok(a) => a,
        Err(e) => return Err("No such file".to_owned())
    };
    let mut result = String::new();
    f.read_to_string(&mut result).unwrap();
    Ok(result)
}

pub fn write(path: PathBuf, content: &[u8]) {
    let mut f = File::open(path).unwrap();
    f.write_all(content);
}