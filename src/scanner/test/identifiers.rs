use super::*;
use crate::error::*;

#[test]
fn test_simple() {
    verify_exact_scan("Hello", &[Token::new(TokenType::Identifier, 0, 5)]);
}

#[test]
fn test_alphanumeric() {
    verify_exact_scan("Hello23", &[Token::new(TokenType::Identifier, 0, 7)]);
}

#[test]
fn test_underscore_1() {
    verify_exact_scan("_hello", &[Token::new(TokenType::Identifier, 0, 6)]);
}

#[test]
fn test_underscore_2() {
    verify_exact_scan("hello_", &[Token::new(TokenType::Identifier, 0, 6)]);
}

#[test]
fn test_underscore_3() {
    verify_exact_scan("_hello_", &[Token::new(TokenType::Identifier, 0, 7)]);
}

#[test]
fn test_underscore_4() {
    verify_exact_scan("_hel_lo_", &[Token::new(TokenType::Identifier, 0, 8)]);
}

#[test]
fn test_underscore_5() {
    verify_exact_scan(
        "hel_ lo",
        &[
            Token::new(TokenType::Identifier, 0, 4),
            Token::new(TokenType::Identifier, 5, 2),
        ],
    );
}

#[test]
fn test_non_ascii_1() {
    let errors =
        verify_exact_scan_with_errors("Hallåj", &[Token::new(TokenType::Identifier, 0, 7)]);
    expect_error_ids(&errors, &[new_error_id(errors::NonAsciiIdentifier)]);
}

#[test]
fn test_non_ascii_2() {
    let errors = verify_exact_scan_with_errors("Hallå", &[Token::new(TokenType::Identifier, 0, 6)]);
    expect_error_ids(&errors, &[new_error_id(errors::NonAsciiIdentifier)]);
}

#[test]
fn test_non_ascii_3() {
    let errors = verify_exact_scan_with_errors("Åland", &[Token::new(TokenType::Identifier, 0, 6)]);
    expect_error_ids(&errors, &[new_error_id(errors::NonAsciiIdentifier)]);
}

#[test]
fn test_non_alphanumerical() {
    let errors = verify_exact_scan_with_errors("Sh💩t", &[Token::new(TokenType::Identifier, 0, 7)]);
    expect_error_ids(&errors, &[new_error_id(errors::NonAsciiIdentifier)]);
}
