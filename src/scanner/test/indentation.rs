use super::*;
use crate::error::*;

#[test]
fn test_simple_indent() {
    verify_exact_scan("\t", &[Token::new(TokenType::Indentation, 0, 1)]);
}

#[test]
fn test_merged_indent() {
    verify_exact_scan("\t\t", &[Token::new(TokenType::Indentation, 0, 2)]);
}

#[test]
fn test_simple_padding() {
    verify_exact_scan(" ", &[Token::new(TokenType::Padding, 0, 1)]);
}

#[test]
fn test_merged_padding() {
    verify_exact_scan("    ", &[Token::new(TokenType::Padding, 0, 4)]);
}

#[test]
fn test_mixed_indent_padding() {
    verify_exact_scan(
        "\t\t    ",
        &[
            Token::new(TokenType::Indentation, 0, 2),
            Token::new(TokenType::Padding, 2, 4),
        ],
    );
}

#[test]
fn test_trailing_whitespace() {
    verify_exact_scan("hello  ", &[Token::new(TokenType::Identifier, 0, 5)]);
}

#[test]
fn test_trailing_whitespace_with_indent() {
    verify_exact_scan(
        "\thello  ",
        &[
            Token::new(TokenType::Indentation, 0, 1),
            Token::new(TokenType::Identifier, 1, 5),
        ],
    );
}

#[test]
fn test_trailing_whitespace_with_indent_and_padding() {
    verify_exact_scan(
        "\t  hello  ",
        &[
            Token::new(TokenType::Indentation, 0, 1),
            Token::new(TokenType::Padding, 1, 2),
            Token::new(TokenType::Identifier, 3, 5),
        ],
    );
}

#[test]
fn test_multiline() {
    verify_sparse_scan(
        "\t \n\t \n\t",
        &[
            Token::new(TokenType::Indentation, 0, 1),
            Token::new(TokenType::Indentation, 3, 1),
            Token::new(TokenType::Indentation, 6, 1),
        ],
    );
}

#[test]
fn test_invalid_1() {
    let errors = verify_exact_scan_with_errors(" \t", &[Token::new(TokenType::Padding, 0, 1)]);
    expect_error_ids(&errors, &[new_error_id(errors::InvalidIndentation)]);
}

#[test]
fn test_invalid_2() {
    let errors = verify_exact_scan_with_errors(
        "\t \t",
        &[
            Token::new(TokenType::Indentation, 0, 1),
            Token::new(TokenType::Padding, 1, 1),
        ],
    );
    expect_error_ids(&errors, &[new_error_id(errors::InvalidIndentation)]);
}
