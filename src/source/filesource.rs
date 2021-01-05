use super::*;
use std::fs::File;
use std::path::*;

pub struct FileSource {
    filename : PathBuf,
}

impl FileSource {
    pub fn new<P : AsRef<Path>>(filename : P) -> FileSource { FileSource { filename: PathBuf::from(filename.as_ref()) } }
}

impl Source<'_, File> for FileSource {
    fn get_readable(&self) -> File {
        File::open(self.filename.clone()).expect("File not found!")
    }
    fn get_reader(&self) -> SourceReader<File> { 
        let file = File::open(self.filename.clone()).expect("File not found!");
        return SourceReader::new(file); 
    }
}
