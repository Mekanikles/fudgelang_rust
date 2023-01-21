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
fn test_block_incomplete() {
    let errors = verify_exact_scan_with_errors("/*", &[Token::new(TokenType::Comment, 0, 2)]);
    expect_error_ids(&errors, &[new_error_id(errors::UnexpectedEOF)]);
}

#[test]
fn test_block_stray_close() {
    let errors = verify_exact_scan_with_errors("*/", &[]);
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
fn test_block_nested_incomplete() {
    let errors = verify_exact_scan_with_errors("/*/**/", &[Token::new(TokenType::Comment, 0, 6)]);
    expect_error_ids(&errors, &[new_error_id(errors::UnexpectedEOF)]);
}

#[test]
fn test_block_with_string() {
    verify_exact_scan("/*\"text\"*/", &[Token::new(TokenType::Comment, 0, 10)]);
}

#[test]
fn test_block_with_unmatched_string() {
    let errors = verify_exact_scan_with_errors("/*\"text", &[Token::new(TokenType::Comment, 0, 7)]);
    expect_error_ids(
        &errors,
        &[
            new_error_id(errors::UnexpectedEOF),
            new_error_id(errors::UnexpectedEOF),
        ],
    );
}

#[test]
fn test_block_with_block_end_inside_string() {
    verify_exact_scan("/*\"*/\"*/", &[Token::new(TokenType::Comment, 0, 8)]);
}

#[test]
fn test_block_with_block_end_inside_unmatched_string() {
    let errors =
        verify_exact_scan_with_errors("/*\"*/test", &[Token::new(TokenType::Comment, 0, 9)]);
    expect_error_ids(
        &errors,
        &[
            new_error_id(errors::UnexpectedEOF),
            new_error_id(errors::UnexpectedEOF),
        ],
    );
}

#[test]
fn test_block_with_unmatched_string_inside_line_comment() {
    verify_exact_scan("/*//\"text\n*/", &[Token::new(TokenType::Comment, 0, 12)]);
}

#[test]
fn test_block_with_block_end_inside_line_comment() {
    verify_exact_scan("/*\n//*/\n*/", &[Token::new(TokenType::Comment, 0, 10)]);
}
