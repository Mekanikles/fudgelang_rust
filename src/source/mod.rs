mod bufferedfilesource; pub use bufferedfilesource::*;
mod memorysource; pub use memorysource::*;

pub use MemorySource as FileSource;

mod lookaheadreader; pub use lookaheadreader::*;

pub trait Source<'a, R> {
    fn get_readable(&'a self) -> R;
}

#[cfg(test)]
mod test;