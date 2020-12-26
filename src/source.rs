use std::io::Read;
use std::io::BufReader;
use std::option::Option;

pub trait SourceReader {
    fn advance(&mut self);
    fn pos(&self) -> u64;
    fn peek(&self) -> Option<u8>;
}

pub struct BufferedSourceReader<R> {
    reader : BufReader<R>,
    pos : u64,
    current : Option<u8>,
}

impl<R : Read> BufferedSourceReader<R> {
    pub fn new(read : R) -> BufferedSourceReader<R> {
        let mut reader = BufferedSourceReader { 
            reader: BufReader::new(read), 
            pos: 0,
            current: None };
        reader.advance();
        return reader;
    }
}

impl<R : Read> SourceReader for BufferedSourceReader<R> {
    fn advance(&mut self) {
        let mut buf = [0; 1];
        self.current = match self.reader.read(&mut buf) {
            Ok(n) if n > 0 => Some(buf[0]),
            _ => None };
    }
    fn pos(&self) -> u64 { self.pos }
    fn peek(&self) -> Option<u8> { self.current }
}

pub trait Source {
    type SourceReaderT : SourceReader;
    fn get_reader(&self) -> Self::SourceReaderT;
}

pub struct ByteArraySource<'a> {
    bytes : &'a[u8],
}

impl<'a> ByteArraySource<'a> {
    pub fn new(bytes : &'a[u8]) -> ByteArraySource { ByteArraySource { bytes } }
}

impl<'a> Source for ByteArraySource<'a> {
    type SourceReaderT = BufferedSourceReader<&'a[u8]>;
    fn get_reader(&self) -> Self::SourceReaderT { 
        return BufferedSourceReader::new(self.bytes); 
    }
}