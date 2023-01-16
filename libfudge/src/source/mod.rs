use std::fs;
use std::path::*;

mod lookaheadreader;
pub use lookaheadreader::*;

use std::io::BufRead;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SourceSpan {
    pub pos: u64,
    pub len: usize,
}

pub struct LineInfo {
    pub text: String,
    pub line_start: usize,
    pub row: u32,
}

pub struct Source {
    name: String,
    data: Vec<u8>,
}

impl Source {
    pub fn from_file<P: AsRef<Path>>(file: P) -> Source {
        Source {
            name: file
                .as_ref()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            data: fs::read(file).unwrap(),
        }
    }

    pub fn from_str(data: &str) -> Source {
        Source {
            name: "stringsource".into(),
            data: data.as_bytes().to_vec(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Source {
        Source {
            name: "bytesource".into(),
            data: bytes.to_vec(),
        }
    }

    pub fn name(&self) -> &str {
        return self.name.as_str();
    }

    pub fn data(&self) -> &[u8] {
        return &self.data[..];
    }

    pub fn to_str(&self) -> &str {
        return std::str::from_utf8(&self.data[..]).unwrap();
    }

    pub fn get_span(&self, span: &SourceSpan) -> &[u8] {
        return &self.data[span.pos as usize..(span.pos as usize + span.len)];
    }

    pub fn get_source_string(&self, span: &SourceSpan) -> &str {
        return std::str::from_utf8(self.get_span(&span)).unwrap();
    }

    pub fn get_line_info(&self, filepos: u64) -> Option<LineInfo> {
        let mut seekpos = 0;
        let mut row = 0;
        let mut text = String::new();

        while let Ok(bytes_read) = (&self.data[seekpos..]).read_line(&mut text) {
            if bytes_read == 0 {
                return None;
            }

            let eol = seekpos + bytes_read;
            if eol > filepos as usize {
                return Some(LineInfo {
                    text,
                    line_start: seekpos,
                    row: row + 1,
                });
            }
            seekpos = eol;
            row += 1;
            text.clear();
        }
        return None;
    }
}

#[cfg(test)]
mod test;
