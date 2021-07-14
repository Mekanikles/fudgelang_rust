use crate::scanner::token;
use crate::source;

pub mod scannererrors;
pub use scannererrors::*;

const fn scanner_error_code(index : u32) -> isize {
    return 0xA000 + index as isize;
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorId {
    // Scanner errors
    InvalidSequece = scanner_error_code(1),
    NonUtf8Sequence = scanner_error_code(2),
    UnexpectedSequence = scanner_error_code(3),
    NonAsciiIdentifier = scanner_error_code(4),
}

#[derive(Clone)]
pub struct Error {
    pub id: ErrorId,
    pub message: String,
    pub source_span: source::SourceSpan,
}

impl Error {
    pub fn at_span(id: ErrorId, source_span: source::SourceSpan, message: String) -> Error {
        Error {
            id,
            message,
            source_span,
        }
    }
    pub fn at_pos(id: ErrorId, pos: u64, message: String) -> Error {
        Self::at_span(id, source::SourceSpan { pos, len: 1 }, message)
    }
    pub fn at_token(id: ErrorId, token: &token::Token, message: String) -> Error {
        Self::at_span(id, token.source_span, message)
    }
}
