use super::*;
use std::fs::File;
use std::io::Cursor;
use std::io::Read;
use std::path::Path;

pub struct MemorySource {
    name: String,
    bytes: Vec<u8>,
}

#[allow(dead_code)]
impl MemorySource {
    pub fn from_bytes(bytes: &[u8]) -> MemorySource {
        MemorySource {
            name: "bytesource".into(),
            bytes: bytes.to_vec(),
        }
    }
    pub fn from_str(string: &str) -> MemorySource {
        MemorySource {
            name: "memorysource".into(),
            bytes: string.as_bytes().to_vec(),
        }
    }
    pub fn from_filepath<P: AsRef<Path>>(filename: P) -> MemorySource {
        let path = filename.as_ref();
        let name : String = path.to_string_lossy().into();
        let mut file = File::open(filename).expect("File not found!");
        let mut data = Vec::new();
        match file.read_to_end(&mut data) {
            Ok(n) if n > 0 => MemorySource { 
                name: name,
                bytes: data 
            },
            _ => MemorySource {
                name: name,
                bytes: Vec::new(),
            },
        }
    }
}

impl<'a> Source<'a, Cursor<&'a [u8]>> for MemorySource {
    fn get_name(&'a self) -> &'a str {
        return &*self.name;
    }

    fn get_readable(&'a self) -> Cursor<&'a [u8]> {
        Cursor::new(&self.bytes)
    }
}
