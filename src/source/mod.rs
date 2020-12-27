use std::io::Read;
use std::io::BufReader;
use std::option::Option;

mod filesource; pub use filesource::*;
mod memorysource; pub use memorysource::*;

#[cfg(test)]
mod test;

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


