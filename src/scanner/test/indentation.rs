use super::*;
use crate::error::*;

#[test]
fn test_simple() {
    verify_exact_scan("\t", &[Token::new(TokenType::Indentation, 0, 1)]);
}

#[test]
fn test_merged() {
    verify_exact_scan("\t\t", &[Token::new(TokenType::Indentation, 0, 2)]);
}

#[test]
fn test_multiline() {
    verify_sparse_scan("\t \n\t \n\t", &[
        Token::new(TokenType::Indentation, 0, 1),
        Token::new(TokenType::Indentation, 3, 1),
        Token::new(TokenType::Indentation, 6, 1),
        ]);
}

#[test]
fn test_invalid_1() {
    let errors = verify_exact_scan_with_errors(" \t", &[]);
    expect_error_ids(&errors, &[new_error_id(errors::InvalidIndentation)]);
}

#[test]
fn test_invalid_2() {
    let errors = verify_exact_scan_with_errors("\t \t", &[Token::new(TokenType::Indentation, 0, 1)]);
    expect_error_ids(&errors, &[new_error_id(errors::InvalidIndentation)]);
}

