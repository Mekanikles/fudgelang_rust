use std::io::Read;
use std::io::BufReader;
use std::option::Option;

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
            Ok(n) if n > 0 => Some(buf[0]),
            _ => None };
    }
    pub fn pos(&self) -> u64 { self.pos }
    pub fn peek(&self) -> Option<u8> { self.current }    
}

pub trait Source<'a, R> {
    fn get_reader(&'a self) -> SourceReader<R>;
}

pub struct StringSource {
    string : String,
}

impl StringSource {
    pub fn new(string : String) -> StringSource { StringSource { string } }
}

impl<'a> Source<'a, &'a[u8]> for StringSource {
    fn get_reader(&'a self) -> SourceReader<&'a[u8]> { 
        return SourceReader::new(self.string.as_bytes()); 
    }
}

pub struct ByteArraySource {
    bytes : Vec<u8>,
}

impl ByteArraySource {
    pub fn new(bytes : &[u8]) -> ByteArraySource { ByteArraySource { bytes: bytes.to_vec() } }
}

impl<'a> Source<'a, &'a[u8]> for ByteArraySource {
    fn get_reader(&'a self) -> SourceReader<&'a[u8]> { 
        return SourceReader::new(&self.bytes); 
    }
}