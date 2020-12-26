use crate::source;
use crate::source::SourceReader;

pub struct Scanner<S : source::Source> {
    source : S,
}

impl<S : source::Source> Scanner<S> {
    pub fn new(source : S) -> Self {
        return Scanner::<S> { source };
    }

    pub fn test(&self) {
        let mut reader = self.source.get_reader();
        while let Some(n) = reader.peek() {
            println!("Pos: {}, val: {}", reader.pos(), n);
            reader.advance();
        }
    }
}

