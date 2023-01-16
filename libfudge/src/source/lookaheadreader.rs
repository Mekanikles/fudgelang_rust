use crate::source::Source;

pub struct LookAheadSourceReader<'a> {
    source: &'a Source,
    pos: u64,
    current: Option<u8>,
    lookahead: Option<u8>,
}

impl<'a> LookAheadSourceReader<'a> {
    pub fn new(source: &'a Source) -> Self {
        let data = source.data();
        let current = data.get(0);
        let lookahead = current.and_then(|_| data.get(1));
        Self {
            source,
            pos: 0,
            current: current.cloned(),
            lookahead: lookahead.cloned(),
        }
    }
    pub fn advance(&mut self) {
        if self.current == None {
            return;
        }
        self.pos += 1;
        self.current = self.lookahead;
        self.lookahead = self.source.data().get(self.pos as usize + 1).cloned();
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

    pub fn source(&self) -> &'a Source {
        return self.source;
    }
}
