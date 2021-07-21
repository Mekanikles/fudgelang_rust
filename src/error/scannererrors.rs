use super::*;

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

pub fn new_invalid_sequence_error(pos: u64, len: u64) -> Error {
    Error::at_span(
        ErrorId::InvalidSequece,
        source::SourceSpan {
            pos,
            len: len as usize,
        },
        "Found invalid sequence".into(),
    )
}

pub fn new_unexpected_sequence_error(pos: u64, len: u64, message: String) -> Error {
    Error::at_span(
        ErrorId::UnexpectedSequence,
        source::SourceSpan {
            pos,
            len: len as usize,
        },
        message,
    )
}

pub fn new_non_ascii_identifier_error(pos: u64, len: u64, identifier: String) -> Error {
    Error::at_span(
        ErrorId::NonAsciiIdentifier,
        source::SourceSpan {
            pos,
            len: len as usize,
        },
        format!("Non-ascii identifier: '{}'", identifier),
    )
}

pub fn new_invalid_indentation_error(pos: u64, len: u64) -> Error {
    Error::at_span(
        ErrorId::InvalidIndentation,
        source::SourceSpan {
            pos,
            len: len as usize,
        },
        "Indentations are only allowed at the start of a line or immediately after other indentations".into(),
    )
}