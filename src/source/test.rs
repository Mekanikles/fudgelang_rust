use super::*;
use std::io::Read;
use std::io::Seek;

fn expect_byte(expected_bytes: &[u8], i: usize, read_byte: u8) {
    if i < expected_bytes.len() {
        assert_eq!(expected_bytes[i], read_byte);
    } else {
        panic!("Source was longer than expected!");
    }
}

fn verify_source<'a, R: Read + Seek, S: Source<'a, R>>(source: &'a S, expected_bytes: &[u8]) {
    let mut reader = LookAheadReader::new(source.get_readable());
    let mut count = 0;
    while let Some(n) = reader.peek() {
        assert_eq!(count, reader.pos());
        expect_byte(expected_bytes, count as usize, n);
        if let Some(n) = reader.lookahead() {
            expect_byte(expected_bytes, count as usize + 1, n);
        }
        reader.advance();
        count += 1;
    }
    assert!(
        count as usize == expected_bytes.len(),
        "Source was shorter than expected!"
    );
}

#[test]
fn test_bufferedfilesource() {
    let source = BufferedFileSource::from_filepath("testdata/sourcetest.txt");
    verify_source(&source, "HejHoppFastFile".as_bytes());
}

#[test]
fn test_memorysource() {
    let source = MemorySource::from_bytes(&[0, 1, 2, 3, 4]);
    verify_source(&source, &[0, 1, 2, 3, 4]);

    let source = MemorySource::from_str("HejHoppFastStr");
    verify_source(&source, "HejHoppFastStr".as_bytes());

    let source = MemorySource::from_filepath("testdata/sourcetest.txt");
    verify_source(&source, "HejHoppFastFile".as_bytes());
}
