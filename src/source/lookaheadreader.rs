use std::io::Read;

pub struct LookAheadReader<R> {
    readable: R,
    pos: u64,
    current: Option<u8>,
    lookahead: Option<u8>,
}

impl<R: Read> LookAheadReader<R> {
    pub fn new(mut readable: R) -> Self {
        let current = readbyte(&mut readable);
        let lookahead = current.and_then(|_| readbyte(&mut readable));
        Self {
            readable,
            pos: 0,
            current,
            lookahead,
        }
    }
    pub fn advance(&mut self) {
        if self.current == None {
            return;
        }
        self.pos += 1;
        self.current = self.lookahead;
        self.lookahead = readbyte(&mut self.readable);
    }
    pub fn pos(&self) -> u64 {
        self.pos
    }
    pub fn peek(&self) -> Option<u8> {
        self.current
    }
    pub fn lookahead(&self) -> Option<u8> {
        self.lookahead
    }
}

fn readbyte<R: Read>(reader: &mut R) -> Option<u8> {
    let mut buf = [0; 1];
    reader
        .read(&mut buf)
        .ok()
        .filter(|&n| n > 0)
        .map(|_| buf[0])
}
