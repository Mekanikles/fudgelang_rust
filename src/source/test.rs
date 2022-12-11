use super::*;

fn expect_byte(expected_bytes: &[u8], i: usize, read_byte: u8) {
    if i < expected_bytes.len() {
        assert_eq!(expected_bytes[i], read_byte);
    } else {
        panic!("Source was longer than expected!");
    }
}

fn verify_source<'a>(source: &Source, expected_bytes: &[u8]) {
    let mut reader = LookAheadSourceReader::new(source);
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
fn test_filesouce() {
    let source = Source::from_file("testdata/singletoken.txt");
    verify_source(&source, "HejHoppFastFile".as_bytes());
}

#[test]
fn test_bytesource() {
    let source = Source::from_bytes(&[0, 1, 2, 3, 4]);
    verify_source(&source, &[0, 1, 2, 3, 4]);
}

#[test]
fn test_stringsource() {
    let source = Source::from_str("HejHoppFastStr");
    verify_source(&source, "HejHoppFastStr".as_bytes());

    let source = Source::from_file("testdata/singletoken.txt");
    verify_source(&source, "HejHoppFastFile".as_bytes());
}

#[test]
fn test_get_line_info_trivial() {
    let source = Source::from_str("");

    let lineinfo = source.get_line_info(0);
    assert!(lineinfo.is_none());
}

#[test]
fn test_get_line_info_simple() {
    let source = Source::from_str("row1\nrow2\nrow3\nrow4");

    let lineinfo = source.get_line_info(12).unwrap();
    assert_eq!(lineinfo.text.trim(), "row3");
    assert_eq!(lineinfo.row, 3);
}

#[test]
fn test_get_line_info_complex() {
    let source = Source::from_str("row1(ö)\x0d\nrow2(💩)\nrow3\nrow4");

    let lineinfo = source.get_line_info(21).unwrap();
    assert_eq!(lineinfo.text.trim(), "row3");
    assert_eq!(lineinfo.row, 3);
}
