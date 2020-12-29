use super::*;

fn readsourcebyte(bytes : &[u8], i : usize) -> u8{
    if i < bytes.len() {
        return bytes[i];
    }
    panic!("Source was shorter than expected!");
}

fn verify_source<'a, R : Read, S : Source<'a, R>>(source : &'a S, bytes : &[u8]) {
    let mut reader = source.get_reader();
    while let Some(n) = reader.peek() {
        assert_eq!(readsourcebyte(bytes, reader.pos() as usize - 1), n);
        if let Some(n) = reader.lookahead() {
            assert_eq!(readsourcebyte(bytes, reader.pos() as usize), n); 
        }
        reader.advance();
    }
}

#[test]
fn test_filesource() {
    let source = FileSource::new("testdata/sourcetest.txt");
    verify_source(&source, "HejHoppFastFile".as_bytes());
}

#[test]
fn test_memorysource() {
    let source = MemorySource::from_bytes(&[0, 1, 2, 3, 4]);
    verify_source(&source, &[0, 1, 2, 3, 4]);

    let source = MemorySource::from_str("HejHoppFastStr");
    verify_source(&source, "HejHoppFastStr".as_bytes());

    let source = MemorySource::from_file("testdata/sourcetest.txt");
    verify_source(&source, "HejHoppFastFile".as_bytes());
}

