use super::*;
use crate::error::*;

#[test]
fn test_line_trivial() {
    verify_exact_scan("//", &[Token::new(TokenType::Comment, 0, 2)]);
}

#[test]
fn test_line_simple() {
    verify_exact_scan("// Comment", &[Token::new(TokenType::Comment, 0, 10)]);
}

#[test]
fn test_line_edges() {
    verify_exact_scan(
        ".// Comment\n",
        &[
            Token::new(TokenType::Dot, 0, 1),
            Token::new(TokenType::Comment, 1, 10),
            Token::new(TokenType::LineBreak, 11, 1),
        ],
    );
}

#[test]
fn test_line_partial() {
    verify_sparse_scan(
        "text// Comment\ntext",
        &[Token::new(TokenType::Comment, 4, 10)],
    );
}

#[test]
fn test_line_nested() {
    verify_sparse_scan(
        "text// Comment // Comment\ntext",
        &[Token::new(TokenType::Comment, 4, 21)],
    );
}

#[test]
fn test_line_multiple() {
    verify_sparse_scan(
        "text// Comment\n// Comment\ntext",
        &[
            Token::new(TokenType::Comment, 4, 10),
            Token::new(TokenType::Comment, 15, 10),
        ],
    );
}

#[test]
fn test_block_trivial() {
    verify_exact_scan("/**/", &[Token::new(TokenType::Comment, 0, 4)]);
}

#[test]
#[should_panic]
fn test_block_incomplete() {
    verify_exact_scan("/*", &[]);
}

#[test]
fn test_block_stray_close() {
    let errors = verify_exact_scan("*/", &[]);
    expect_error_ids(&errors, &[new_error_id(errors::UnexpectedSequence)]);
}

#[test]
fn test_block_simple() {
    verify_exact_scan("/* Comment */", &[Token::new(TokenType::Comment, 0, 13)]);
}

#[test]
fn test_block_edges() {
    verify_exact_scan(
        "./* Comment */.",
        &[
            Token::new(TokenType::Dot, 0, 1),
            Token::new(TokenType::Comment, 1, 13),
            Token::new(TokenType::Dot, 14, 1),
        ],
    );
}

#[test]
fn test_block_partial() {
    verify_sparse_scan(
        "text/* Comment */text",
        &[Token::new(TokenType::Comment, 4, 13)],
    );
}

#[test]
fn test_block_nested_trivial() {
    verify_exact_scan("/*/**/*/", &[Token::new(TokenType::Comment, 0, 8)]);
}

#[test]
#[should_panic]
fn test_block_nested_incomplete() {
    verify_exact_scan("/*/**/", &[Token::new(TokenType::Comment, 0, 8)]);
}
