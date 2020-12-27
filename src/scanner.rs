use std::io::Read;
use crate::source;
use std::marker::PhantomData;

pub struct Scanner<'a, R : Read, S : source::Source<'a, R>> {
    source : S,
    goddamnitrust : PhantomData<&'a R>,
}

impl<'a, R : Read, S : source::Source<'a, R>> Scanner<'a, R, S> {
    pub fn new(source : S) -> Self {
        return Scanner::<R, S> { source, goddamnitrust: PhantomData };
    }

    pub fn test(&'a self) {
        let mut reader = self.source.get_reader();
        while let Some(n) = reader.peek() {
            println!("Pos: {}, val: {}", reader.pos(), n);
            reader.advance();
        }
    }
}

