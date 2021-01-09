use super::scanner::*;
use super::token::*;
use crate::error;
use crate::source::*;

mod basic;
mod comment;

fn expect_error_ids(errors: &Vec<error::Error>, expected_error_ids: &[error::ErrorId]) {
    for i in 0..expected_error_ids.len() {
        assert_eq!(errors[i].id, expected_error_ids[i]);
    }
    assert_eq!(
        errors.len(),
        expected_error_ids.len(),
        "Found more errors than expected!"
    );
}

fn expect_token(expected_tokens: &[Token], i: usize, scanned_token: &Token) {
    if i < expected_tokens.len() {
        assert_eq!(expected_tokens[i], *scanned_token);
    } else {
        panic!("Scanned more tokens than expected!");
    }
}

fn verify_scanner_tokens<S: Scanner>(scanner: &mut S, expected_tokens: &[Token]) {
    let mut count = 0;
    while let Some(t) = scanner.read_token() {
        expect_token(expected_tokens, count, &t);
        count += 1;
    }

    assert_eq!(
        count as usize,
        expected_tokens.len(),
        "Scanned less tokens than expected!"
    );
}

// Checks that the scanner produces an exact list of tokens
fn verify_exact_scan(source: &str, expected_tokens: &[Token]) -> Vec<error::Error> {
    let source = MemorySource::from_str(source);
    let mut scanner = ScannerImpl::new(&source);

    verify_scanner_tokens(&mut scanner, expected_tokens);
    scanner.errors.clone()
}

// Checks that scanner produces any tokens that matches the list
fn verify_sparse_scan(source: &str, expected_tokens: &[Token]) -> Vec<error::Error> {
    let source = MemorySource::from_str(source);
    let mut scanner = ScannerImpl::new(&source);

    let mut scanned_tokens = Vec::new();
    while let Some(t) = scanner.read_token() {
        scanned_tokens.push(t);
    }

    for t in expected_tokens {
        assert!(
            scanned_tokens.iter().position(|e| e == t) != None,
            "Expected token not found in scan!"
        );
    }
    scanner.errors.clone()
}
