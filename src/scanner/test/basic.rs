use super::*;
use crate::error;

#[test]
fn test_simple_error() {
    let errors = verify_exact_scan("€", &[]);
    assert_eq!(errors.len(), 1);
}

#[test]
fn test_errors_and_tokens() {
    let errors = verify_exact_scan(
        ".€€€€.",
        &[
            Token::new(TokenType::Dot, 0, 1),
            Token::new(TokenType::Dot, 13, 1),
        ],
    );
    assert_eq!(errors.len(), 4);
}

#[test]
fn test_error_threshold() {
    let errors = verify_exact_scan(".€€€€€.", &[Token::new(TokenType::Dot, 0, 1)]);
    assert_eq!(errors.len(), 5);
}

#[test]
fn test_non_ascii_chars() {
    let errors = verify_exact_scan(
        ".ö.",
        &[
            Token::new(TokenType::Dot, 0, 1),
            Token::new(TokenType::Dot, 3, 1),
        ],
    );
    expect_error_ids(&errors, &[error::ErrorId::NonAsciiChar]);
}

#[test]
fn test_unexpected_chars() {
    let errors = verify_exact_scan(
        ".\0.",
        &[
            Token::new(TokenType::Dot, 0, 1),
            Token::new(TokenType::Dot, 2, 1),
        ],
    );
    expect_error_ids(&errors, &[error::ErrorId::UnexpectedChar]);
}

#[test]
fn test_non_utf8_sequence() {
    const ILLEGAL_CTRL_BYTE: u8 = 0b10000000;
    const CTRL_2_BYTE: u8 = 0b11000000;
    const CTRL_3_BYTE: u8 = 0b11100000;
    const CTRL_4_BYTE: u8 = 0b11110000;
    const SEQ_BYTE: u8 = 0b10000000;
    const ILLEGAL_SEQ_BYTE: u8 = 0b01000000;

    let source = MemorySource::from_bytes(&[
        b'.',
        ILLEGAL_CTRL_BYTE,
        b'.',
        CTRL_2_BYTE,
        ILLEGAL_SEQ_BYTE,
        b'.',
        CTRL_3_BYTE,
        SEQ_BYTE,
        ILLEGAL_SEQ_BYTE,
        b'.',
        CTRL_4_BYTE,
        SEQ_BYTE,
        SEQ_BYTE,
        ILLEGAL_SEQ_BYTE,
        b'.',
    ]);
    let mut scanner = ScannerImpl::new(&source);

    verify_scanner_tokens(
        &mut scanner,
        &[
            Token::new(TokenType::Dot, 0, 1),
            Token::new(TokenType::Dot, 2, 1),
            Token::new(TokenType::Dot, 5, 1),
            Token::new(TokenType::Dot, 9, 1),
            Token::new(TokenType::Dot, 14, 1),
        ],
    );
    expect_error_ids(
        &scanner.errors,
        &[
            error::ErrorId::NonUtf8Sequence,
            error::ErrorId::NonUtf8Sequence,
            error::ErrorId::NonUtf8Sequence,
            error::ErrorId::NonUtf8Sequence,
        ],
    );
}
