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
fn test_simple_padding_1() {
    let errors = verify_exact_scan_with_errors(" ", &[Token::new(TokenType::Indentation, 0, 1)]);
    expect_error_ids(&errors, &[new_error_id(errors::PaddingNotSupported)]);
}

#[test]
fn test_simple_padding_2() {
    let errors = verify_exact_scan_with_errors(
        "   a",
        &[
            Token::new(TokenType::Indentation, 0, 3),
            Token::new(TokenType::Identifier, 3, 1),
        ],
    );
    expect_error_ids(&errors, &[new_error_id(errors::PaddingNotSupported)]);
}

#[test]
fn test_mixed_indent_padding() {
    let errors =
        verify_exact_scan_with_errors("\t\t    ", &[Token::new(TokenType::Indentation, 0, 6)]);
    expect_error_ids(&errors, &[new_error_id(errors::PaddingNotSupported)]);
}

#[test]
fn test_trailing_whitespace() {
    let errors =
        verify_exact_scan_with_errors("hello  ", &[Token::new(TokenType::Identifier, 0, 5)]);
    expect_error_ids(&errors, &[new_error_id(errors::TrailingWhitespace)]);
}

#[test]
fn test_trailing_whitespace_with_indent() {
    let errors = verify_exact_scan_with_errors(
        "\thello  ",
        &[
            Token::new(TokenType::Indentation, 0, 1),
            Token::new(TokenType::Identifier, 1, 5),
        ],
    );
    expect_error_ids(&errors, &[new_error_id(errors::TrailingWhitespace)]);
}

#[test]
fn test_trailing_whitespace_with_indent_and_padding() {
    let errors = verify_exact_scan_with_errors(
        "\t  hello  ",
        &[
            Token::new(TokenType::Indentation, 0, 3),
            Token::new(TokenType::Identifier, 3, 5),
        ],
    );
    expect_error_ids(
        &errors,
        &[
            new_error_id(errors::PaddingNotSupported),
            new_error_id(errors::TrailingWhitespace),
        ],
    );
}

#[test]
fn test_multiline_indentation() {
    verify_sparse_scan(
        "\ta\n\ta\n\t",
        &[
            Token::new(TokenType::Indentation, 0, 1),
            Token::new(TokenType::Indentation, 3, 1),
            Token::new(TokenType::Indentation, 6, 1),
        ],
    );
}

#[test]
fn test_invalid_1() {
    let errors = verify_exact_scan_with_errors(" \t", &[Token::new(TokenType::Indentation, 0, 2)]);
    expect_error_ids(&errors, &[new_error_id(errors::PaddingNotSupported)]);
}

#[test]
fn test_invalid_2() {
    let errors =
        verify_exact_scan_with_errors("\t \t", &[Token::new(TokenType::Indentation, 0, 3)]);
    expect_error_ids(&errors, &[new_error_id(errors::PaddingNotSupported)]);
}
