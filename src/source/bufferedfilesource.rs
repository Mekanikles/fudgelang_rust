use super::*;
use std::fs::File;
use std::io::BufReader;
use std::path::*;

pub struct BufferedFileSource {
    filename: PathBuf,
}

impl BufferedFileSource {
    pub fn from_filepath<P: AsRef<Path>>(filename: P) -> BufferedFileSource { BufferedFileSource { filename: PathBuf::from(filename.as_ref()) } }
}

impl Source<'_, BufReader<File>> for BufferedFileSource {
    fn get_readable(&self) -> BufReader<File> {
        BufReader::new(File::open(self.filename.clone()).expect("File not found!"))
    }
}
