use super::*;
use crate::error;
use crate::scanner::*;
use crate::source::*;

pub fn expect_error_ids(errors: &Vec<error::Error>, expected_error_ids: &[error::ErrorId]) {
    for i in 0..expected_error_ids.len() {
        assert_eq!(errors[i].id, expected_error_ids[i]);
    }
    assert_eq!(
        errors.len(),
        expected_error_ids.len(),
        "Found more errors than expected!"
    );
}

pub fn expect_token(expected_tokens: &[Token], i: usize, scanned_token: &Token) {
    if i < expected_tokens.len() {
        assert_eq!(expected_tokens[i], *scanned_token);
    } else {
        panic!("Scanned more tokens than expected!");
    }
}

pub fn verify_exact_scanner_tokens(scanner_result: &ScannerResult, expected_tokens: &[Token]) {
    let mut count = 0;
    for t in &scanner_result.tokens {
        expect_token(expected_tokens, count, &t);
        count += 1;
    }

    assert_eq!(
        count as usize,
        expected_tokens.len(),
        "Scanned less tokens than expected!"
    );
}

pub fn verify_sparse_scanner_tokens(scanner_result: &ScannerResult, expected_tokens: &[Token]) {
    for t in expected_tokens {
        assert!(
            scanner_result.tokens.iter().position(|e| e == t) != None,
            "Expected token not found in scan!"
        );
    }
}

pub fn verify_scanner_tokens_snapshot(scanner_result: &ScannerResult) {
    insta::assert_debug_snapshot!(scanner_result.tokens);
}

// Checks that the scanner produces an exact list of tokens
pub fn verify_exact_scan_with_errors(source: &str, expected_tokens: &[Token]) -> Vec<error::Error> {
    let source = Source::from_str(source);
    let scanner_result = scanner::tokenize(&source);
    verify_exact_scanner_tokens(&scanner_result, expected_tokens);
    return scanner_result.errors;
}

pub fn verify_exact_scan(source: &str, expected_tokens: &[Token]) {
    let errors = verify_exact_scan_with_errors(source, expected_tokens);
    assert!(errors.is_empty());
}

// Checks that scanner produces any tokens that matches the list
pub fn verify_sparse_scan_with_errors(
    source: &str,
    expected_tokens: &[Token],
) -> Vec<error::Error> {
    let source = Source::from_str(source);
    let scanner_result = scanner::tokenize(&source);

    verify_sparse_scanner_tokens(&scanner_result, expected_tokens);
    return scanner_result.errors;
}

pub fn verify_sparse_scan(source: &str, expected_tokens: &[Token]) {
    let errors = verify_sparse_scan_with_errors(source, expected_tokens);
    assert!(errors.is_empty());
}

pub fn do_scan_with_errors(source: &str) -> Vec<error::Error> {
    let source = Source::from_str(source);
    let scanner_result = scanner::tokenize(&source);
    return scanner_result.errors;
}

pub fn get_scanner_result_from_file(file: &str) -> ScannerResult {
    let source = Source::from_file(file);
    let scanner_result = scanner::tokenize(&source);
    return scanner_result;
}

pub fn get_scanner_result_from_bytes(bytes: &[u8]) -> ScannerResult {
    let source = Source::from_bytes(bytes);
    let scanner_result = scanner::tokenize(&source);
    return scanner_result;
}
