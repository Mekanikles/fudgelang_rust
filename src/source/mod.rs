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
    lookahead : Option<u8>,
}

impl<R : Read> SourceReader<R> {
    fn readbyte(reader : &mut BufReader<R>) -> Option<u8>
    {
        let mut buf = [0; 1];
        match reader.read(&mut buf) {
            Ok(n) if n > 0 => {
                Some(buf[0])},
            _ => None
        }
    }
    pub fn new(read : R) -> SourceReader<R> {
        let mut reader = BufReader::new(read);
        let mut pos = 0;
        let mut lookahead = None;
        let current = Self::readbyte(&mut reader);
        if current != None {
            pos = 1;
            lookahead = Self::readbyte(&mut reader);
        }

        return SourceReader { 
            reader, 
            pos,
            current,
            lookahead };
    }
    pub fn advance(&mut self) {
        if self.current == None {
            return;
        }
        self.pos += 1;
        self.current = self.lookahead;
        self.lookahead = Self::readbyte(&mut self.reader);
    }
    pub fn pos(&self) -> u64 { self.pos }
    pub fn peek(&self) -> Option<u8> { self.current }
    pub fn lookahead(&self) -> Option<u8> { self.lookahead } 
}

pub trait Source<'a, R> {
    fn get_reader(&'a self) -> SourceReader<R>;
}


