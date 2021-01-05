use std::io::Read;
use std::io::Seek;
use std::option::Option;

pub struct LookAheadReader<R> {
    readable: R,
    pos: u64,
    current: Option<u8>,
    lookahead: Option<u8>,
}

impl<R: Read + Seek> LookAheadReader<R> {
    pub fn new(readable: R) -> LookAheadReader<R> {
        let mut readable = readable;
        let mut lookahead = None;
        let current = Self::readbyte(&mut readable);
        if current != None {
            lookahead = Self::readbyte(&mut readable);
        }

        return LookAheadReader { 
            readable, 
            pos: 0,
            current,
            lookahead };
    }
    pub fn advance(&mut self) {
        if self.current == None {
            return;
        }
        self.pos += 1;
        self.current = self.lookahead;
        self.lookahead = Self::readbyte(&mut self.readable);
    }
    pub fn pos(&self) -> u64 { self.pos }
    pub fn peek(&self) -> Option<u8> { self.current }
    pub fn lookahead(&self) -> Option<u8> { self.lookahead }
}

impl<R: Read + Seek> LookAheadReader<R> {
    fn readbyte(reader: &mut R) -> Option<u8>
    {
        let mut buf = [0; 1];
        match reader.read(&mut buf) {
            Ok(n) if n > 0 => {
                Some(buf[0])},
            _ => None
        }
    }
}


