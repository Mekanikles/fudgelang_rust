use crate::scanner::token;
use crate::source;

pub mod scannererrors;
pub use scannererrors::*;

pub mod errors {
    // TODO: We might want to attribute "severity" to errors more dynamically
    //  (say, via some map), so that users can elevate warnings to errors etc
    // The static division here might be better oriented towards recoverability, 
    //  or category, rather than user-facing severity
    pub use FatalErrorType::*;
    pub use MajorErrorType::*;
    pub use MinorErrorType::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum FatalErrorType {
        UnexpectedEOF,
        ErrorLimitExceeded,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum MajorErrorType {
        InvalidSequece,
        NonUtf8Sequence,
        UnexpectedSequence,
        UnexpectedToken,
        ExpectedExpression,
        UnknownCompilerDirective,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum MinorErrorType {
        NonAsciiIdentifier,
        InvalidIndentation,
        ExpectedInputParameterDeclaration,
        ExpectedOutputParameterDeclaration,
    }
}

const FATAL_ERROR_THRESHOLD: usize = 1;
const MAJOR_ERROR_THRESHOLD: usize = 5;
const MINOR_ERROR_THRESHOLD: usize = 20;

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
        ErrorId::FatalError(*self)
    }
}

impl ErrorIdConstructor for errors::MajorErrorType {
    fn create_id(&self) -> ErrorId {
        ErrorId::MajorError(*self)
    }
}

impl ErrorIdConstructor for errors::MinorErrorType {
    fn create_id(&self) -> ErrorId {
        ErrorId::MinorError(*self)
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

pub struct ErrorData {
    fatal_error_count: usize,
    major_error_count: usize,
    minor_error_count: usize,
    errors: Vec<Error>,
}

pub struct ErrorManager {
    reached_error_limit : bool,
    error_data : ErrorData,
}

impl ErrorManager {
    pub fn new() -> Self {
        ErrorManager {
            reached_error_limit: false,
            error_data: ErrorData {
                fatal_error_count: 0,
                major_error_count: 0,
                minor_error_count: 0,
                errors: Vec::new(),
            },
        }
    }
    
    pub fn reached_error_limit(&self) -> bool {
        return self.reached_error_limit;
    }

    pub fn get_errors(&self) -> &Vec<Error> {
        return &self.error_data.errors;
    }

    pub fn log_error(&mut self, error: Error) -> ErrorId {
        let id = error.id;
        self.error_data.errors.push(error);

        match id {
            ErrorId::FatalError(_e) => {
                self.error_data.fatal_error_count += 1;
                if self.error_data.fatal_error_count >= FATAL_ERROR_THRESHOLD {
                    self.reached_error_limit = true;
                }
            }
            ErrorId::MajorError(_e) => {
                self.error_data.major_error_count += 1;
                if self.error_data.major_error_count >= MAJOR_ERROR_THRESHOLD {
                    self.reached_error_limit = true;
                }
            }
            ErrorId::MinorError(_e) => {
                self.error_data.minor_error_count += 1;
                if self.error_data.minor_error_count >= MINOR_ERROR_THRESHOLD {
                    self.reached_error_limit = true;
                }
            }
        }
        return id;
    }

    pub fn adjust_last_error_end(&mut self, end : u64) {
        let pos = self.error_data.errors.last_mut().unwrap().source_span.pos;
        self.error_data.errors.last_mut().unwrap().source_span.len = (end - pos) as usize;
    }
}