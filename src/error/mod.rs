use crate::scanner::token;
use crate::source;

pub mod scannererrors;
pub use scannererrors::*;

pub mod errors {
    pub use FatalErrorType::*;
    pub use MajorErrorType::*;
    pub use MinorErrorType::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum FatalErrorType {
        PlaceHolder,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum MajorErrorType {
        InvalidSequece,
        NonUtf8Sequence,
        UnexpectedSequence,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum MinorErrorType {
        NonAsciiIdentifier,
        InvalidIndentation,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ErrorId {
    FatalError(errors::FatalErrorType),
    MajorError(errors::MajorErrorType),
    MinorError(errors::MinorErrorType),
}

pub trait ErrorIdConstructor {
    fn create_id(&self) -> ErrorId;
}

impl ErrorIdConstructor for errors::FatalErrorType {
    fn create_id(&self) -> ErrorId {
        ErrorId::FatalError(self.clone())
    }
}

impl ErrorIdConstructor for errors::MajorErrorType {
    fn create_id(&self) -> ErrorId {
        ErrorId::MajorError(self.clone())
    }
}

impl ErrorIdConstructor for errors::MinorErrorType {
    fn create_id(&self) -> ErrorId {
        ErrorId::MinorError(self.clone())
    }
}

pub fn new_error_id<T: ErrorIdConstructor>(t: T) -> ErrorId {
    t.create_id()
}

pub fn error_label(id: ErrorId) -> &'static str {
    match id {
        ErrorId::FatalError(_e) => {
            return "Error";
        }
        ErrorId::MajorError(_e) => {
            return "Error";
        }
        ErrorId::MinorError(_e) => {
            return "Error";
        }
    }
}

pub fn error_code(id: ErrorId) -> String {
    match id {
        ErrorId::FatalError(e) => {
            return format!("A{:03}", e as i32);
        }
        ErrorId::MajorError(e) => {
            return format!("B{:03}", e as i32);
        }
        ErrorId::MinorError(e) => {
            return format!("C{:03}", e as i32);
        }
    }
}

#[derive(Clone)]
pub struct Error {
    pub id: ErrorId,
    pub message: String,
    pub source_span: source::SourceSpan,
}

impl Error {
    pub fn at_span<T: ErrorIdConstructor>(
        t: T,
        source_span: source::SourceSpan,
        message: String,
    ) -> Error {
        Error {
            id: new_error_id(t),
            message,
            source_span,
        }
    }
    pub fn at_pos<T: ErrorIdConstructor>(t: T, pos: u64, message: String) -> Error {
        Self::at_span(t, source::SourceSpan { pos, len: 1 }, message)
    }
    pub fn at_token<T: ErrorIdConstructor>(t: T, token: &token::Token, message: String) -> Error {
        Self::at_span(t, token.source_span, message)
    }
}
