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
fn test_non_ascii_1() {
    let errors = verify_exact_scan("Hallåj", &[Token::new(TokenType::Identifier, 0, 7)]);
    expect_error_ids(&errors, &[new_error_id(errors::NonAsciiIdentifier)]);
}

#[test]
fn test_non_ascii_2() {
    let errors = verify_exact_scan("Hallå", &[Token::new(TokenType::Identifier, 0, 6)]);
    expect_error_ids(&errors, &[new_error_id(errors::NonAsciiIdentifier)]); 
}

#[test]
fn test_non_ascii_3() {
    let errors = verify_exact_scan("Åland", &[Token::new(TokenType::Identifier, 0, 6)]);
    expect_error_ids(&errors, &[new_error_id(errors::NonAsciiIdentifier)]); 
}

#[test]
fn test_non_alphanumerical() {
    let errors = verify_exact_scan("Sh💩t", &[
        Token::new(TokenType::Identifier, 0, 7),
        ]);
    expect_error_ids(&errors, &[new_error_id(errors::NonAsciiIdentifier)]);
}