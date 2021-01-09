use crate::scanner::token;
use crate::source;

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorId {
    NonAsciiChar,
    UnexpectedChar,
    NonUtf8Sequence,
    EnexpectedSequence,
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

pub fn new_non_utf8_sequence_error(pos: u64, len: u64) -> Error {
    Error::at_span(
        ErrorId::NonUtf8Sequence,
        source::SourceSpan {
            pos,
            len: len as usize,
        },
        "Found non-ut8 sequence".into(),
    )
}

pub fn new_unexpected_char_error(c: char, pos: u64) -> Error {
    Error::at_pos(
        ErrorId::UnexpectedChar,
        pos,
        format!("Found unexpected character '{}'", c),
    )
}

pub fn new_non_ascii_char_error(c: char, pos: u64) -> Error {
    Error::at_pos(
        ErrorId::NonAsciiChar,
        pos,
        format!(
            "Found non-ascii character '{}' outside of string literal",
            c
        ),
    )
}

pub fn new_unexpected_sequence_error(pos: u64, len: u64, message: String) -> Error {
    Error::at_span(
        ErrorId::EnexpectedSequence,
        source::SourceSpan {
            pos,
            len: len as usize,
        },
        message,
    )
}
