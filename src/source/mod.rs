use std::io::Read;

mod bufferedfilesource;
pub use bufferedfilesource::*;
mod memorysource;
pub use memorysource::*;

pub use MemorySource as FileSource;

mod lookaheadreader;
pub use lookaheadreader::*;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SourceSpan {
    pub pos: u64,
    pub len: usize,
}

pub trait Source<'a, R : Read> {
    fn get_readable(&'a self) -> R;
}

#[cfg(test)]
mod test;
