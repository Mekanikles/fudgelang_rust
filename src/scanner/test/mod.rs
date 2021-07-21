use super::scanner::*;
use super::token::*;
use crate::source;

mod utils;
use utils::*;

mod basic;
mod comments;
mod identifiers;
mod indentation;

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

    verify_sparse_scanner_tokens(
        &mut scanner,
        &[
            Token::new(TokenType::Identifier, 0, 5),
            Token::new(TokenType::Comma, 5, 1),
            Token::new(TokenType::Identifier, 7, 7),
            Token::new(TokenType::Dot, 14, 1),
            Token::new(TokenType::Comment, 16, 10),
            Token::new(TokenType::LineBreak, 26, 1),
            Token::new(TokenType::Identifier, 27, 5),
            Token::new(TokenType::Comment, 32, 18),
            Token::new(TokenType::Identifier, 50, 5),
            Token::new(TokenType::Comma, 55, 1),
            Token::new(TokenType::LineBreak, 56, 1),
            Token::new(TokenType::Comment, 58, 94),
            Token::new(TokenType::LineBreak, 152, 1),
            Token::new(TokenType::Identifier, 153, 7),
            Token::new(TokenType::Comma, 160, 1),
            Token::new(TokenType::Identifier, 164, 5),
            Token::new(TokenType::Dot, 169, 1),
            Token::new(TokenType::LineBreak, 170, 1),
            Token::new(TokenType::Comment, 171, 16),
        ],
    );    
}

