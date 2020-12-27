use std::io::Read;
use std::io::BufReader;
use std::option::Option;
use std::fs::File;

pub struct SourceReader<R> {
    reader : BufReader<R>,
    pos : u64,
    current : Option<u8>,
}

impl<R : Read> SourceReader<R> {
    pub fn new(read : R) -> SourceReader<R> {
        let mut reader = SourceReader { 
            reader: BufReader::new(read), 
            pos: 0,
            current: None };
        reader.advance();
        return reader;
    }
    pub fn advance(&mut self) {
        let mut buf = [0; 1];
        self.current = match self.reader.read(&mut buf) {
            Ok(n) if n > 0 => {
                self.pos += 1;
                Some(buf[0])},
            _ => None };
    }
    pub fn pos(&self) -> u64 { self.pos }
    pub fn peek(&self) -> Option<u8> { self.current }    
}

pub trait Source<'a, R> {
    fn get_reader(&'a self) -> SourceReader<R>;
}

pub struct FileSource {
    filename : String,
}

impl FileSource {
    pub fn new(filename : &str) -> FileSource { FileSource { filename: filename.to_string() } }
}

impl Source<'_, File> for FileSource {
    fn get_reader(&self) -> SourceReader<File> { 
        let file = File::open(self.filename.clone()).expect("File not found!");
        return SourceReader::new(file); 
    }
}

pub struct MemorySource {
    bytes : Vec<u8>,
}

impl MemorySource {
    pub fn from_bytes(bytes : &[u8]) -> MemorySource { MemorySource { bytes: bytes.to_vec() } }
    pub fn from_str(string : &str) -> MemorySource { MemorySource { bytes: string.as_bytes().to_vec() } }
    pub fn from_file(filename : &str) -> MemorySource { 
        let mut file = File::open(filename).expect("File not found!");
        let mut data = Vec::new();
        match file.read_to_end(&mut data) {
            Ok(n) if n > 0 => MemorySource { bytes: data },
            _ => MemorySource { bytes: Vec::new() }
        }
    }     
}

impl<'a> Source<'a, &'a[u8]> for MemorySource {
    fn get_reader(&'a self) -> SourceReader<&'a[u8]> { 
        return SourceReader::new(&self.bytes); 
    }
}