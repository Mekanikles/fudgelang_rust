use super::*;
use std::fs::File;

pub struct FileSource {
    filename : String,
}

impl FileSource {
    pub fn new(filename : &str) -> FileSource { FileSource { filename: filename.to_string() } }
}

impl Source<'_, File> for FileSource {
    fn get_reader(&self) -> SourceReader<File> { 
        let file = File::open(self.filename.clone()).expect("File not found!");
        return SourceReader::new(file); 
    }
}
