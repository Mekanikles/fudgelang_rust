use super::*;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;

pub struct MemorySource {
    bytes: Vec<u8>,
}

#[allow(dead_code)]
impl MemorySource {
    pub fn from_bytes(bytes: &[u8]) -> MemorySource {
        MemorySource {
            bytes: bytes.to_vec(),
        }
    }
    pub fn from_str(string: &str) -> MemorySource {
        MemorySource {
            bytes: string.as_bytes().to_vec(),
        }
    }
    pub fn from_filepath<P: AsRef<Path>>(filename: P) -> MemorySource {
        let mut file = File::open(filename).expect("File not found!");
        let mut data = Vec::new();
        match file.read_to_end(&mut data) {
            Ok(n) if n > 0 => MemorySource { bytes: data },
            _ => MemorySource { bytes: Vec::new() },
        }
    }
}

impl<'a> Source<'a, Cursor<&'a [u8]>> for MemorySource {
    fn get_readable(&'a self) -> Cursor<&'a [u8]> {
        Cursor::new(&self.bytes)
    }
}
