use crate::source::*;
use super::token::*;
use super::scanner::*;

mod comment;

fn expect_token(expected_tokens : &[Token], i : usize, scanned_token : &Token) {
    if i < expected_tokens.len() {
        assert_eq!(expected_tokens[i], *scanned_token);
    }
    else {
        panic!("Scanned more tokens than expected!");
    }
}

fn verify_scanner_tokens<S : Scanner>(scanner : &mut S, expected_tokens : &[Token])
{
    let mut count = 0;
    while let Some(t) = scanner.read_token() {
        expect_token(expected_tokens, count, &t);
        count += 1;
    }

    assert!(count as usize == expected_tokens.len(), "Scanned less tokens than expected!");    
}

// Checks that the scanner produces an exact list of tokens
fn verify_exact_scan(source : &str, expected_tokens : &[Token]) {
    let source = MemorySource::from_str(source);
    let mut scanner = ScannerImpl::new(&source);

    verify_scanner_tokens(&mut scanner, expected_tokens);
}

// Checks that scanner produces any tokens that matches the list
fn verify_sparse_scan(source : &str, expected_tokens : &[Token]) {
    let source = MemorySource::from_str(source);
    let mut scanner = ScannerImpl::new(&source);
    
    let mut scanned_tokens = Vec::new();
    while let Some(t) = scanner.read_token() {
        scanned_tokens.push(t);
    }

    for t in expected_tokens {
        assert!(scanned_tokens.iter().position(|e| e == t) != None, 
            "Expected token not found in scan!");
    }
}
