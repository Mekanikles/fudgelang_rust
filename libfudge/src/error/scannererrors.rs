use super::*;

pub fn new_unexpected_eof_error(pos: u64, message: String) -> Error {
    Error::at_span(
        errors::UnexpectedEOF,
        source::SourceSpan { pos, len: 1 },
        message,
    )
}

pub fn new_non_utf8_sequence_error(pos: u64, len: u64) -> Error {
    Error::at_span(
        errors::NonUtf8Sequence,
        source::SourceSpan {
            pos,
            len: len as usize,
        },
        "Found non-ut8 sequence".into(),
    )
}

pub fn new_invalid_sequence_error(pos: u64, len: u64) -> Error {
    Error::at_span(
        errors::InvalidSequece,
        source::SourceSpan {
            pos,
            len: len as usize,
        },
        "Found invalid sequence".into(),
    )
}

pub fn new_unexpected_sequence_error(pos: u64, len: u64, message: String) -> Error {
    Error::at_span(
        errors::UnexpectedSequence,
        source::SourceSpan {
            pos,
            len: len as usize,
        },
        message,
    )
}

pub fn new_non_ascii_identifier_error(pos: u64, len: u64, identifier: String) -> Error {
    Error::at_span(
        errors::NonAsciiIdentifier,
        source::SourceSpan {
            pos,
            len: len as usize,
        },
        format!("Non-ascii identifier: '{}'", identifier),
    )
}

pub fn new_unexpected_indentation_error(pos: u64, len: u64) -> Error {
    Error::at_span(
        errors::UnexpectedIndentation,
        source::SourceSpan {
            pos,
            len: len as usize,
        },
        "Indentations are only allowed at the start of a line or immediately after other indentations".into(),
    )
}

pub fn new_padding_not_supported_error(pos: u64, len: u64) -> Error {
    Error::at_span(
        errors::PaddingNotSupported,
        source::SourceSpan {
            pos,
            len: len as usize,
        },
        "Indentation/Padding with spaces is not allowed".into(),
    )
}

pub fn new_trailing_whitespace_error(pos: u64, len: u64) -> Error {
    Error::at_span(
        errors::TrailingWhitespace,
        source::SourceSpan {
            pos,
            len: len as usize,
        },
        "Trailing whitespace is not allowed".into(),
    )
}
