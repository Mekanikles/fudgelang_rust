use super::scanner::*;
use super::token::*;
use crate::source;

mod utils;
use utils::*;

mod basic;
mod comments;
mod identifiers;
mod indentation;
mod keywords;

#[test]
fn test_get_line_info_trivial() {
    let source = source::MemorySource::from_str("");
    let scanner = ScannerImpl::new(&source);

    let lineinfo = scanner.get_line_info(0);
    assert!(lineinfo.is_none());
}

#[test]
fn test_get_line_info_simple() {
    let source = source::MemorySource::from_str("row1\nrow2\nrow3\nrow4");
    let scanner = ScannerImpl::new(&source);

    let lineinfo = scanner.get_line_info(12).unwrap();
    assert_eq!(lineinfo.text.trim(), "row3");
    assert_eq!(lineinfo.row, 3);
}

#[test]
fn test_get_line_info_complex() {
    let source = source::MemorySource::from_str("row1(ö)\x0d\nrow2(💩)\nrow3\nrow4");
    let scanner = ScannerImpl::new(&source);

    let lineinfo = scanner.get_line_info(21).unwrap();
    assert_eq!(lineinfo.text.trim(), "row3");
    assert_eq!(lineinfo.row, 3);
}

#[test]
fn test_file_with_comments() {
    let source = source::MemorySource::from_filepath("testdata/comments.txt");
    let mut scanner = ScannerImpl::new(&source);

    verify_scanner_tokens_snapshot(&mut scanner);
}

