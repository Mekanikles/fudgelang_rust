use super::*;
use crate::source;
use std::fmt;

#[derive(PartialEq, Debug)]
pub enum TokenType {
    // 1-char tokens
    Comma,
    Dot,
    SemiColon,
    Indent,
    LineBreak,

    // n-char tokens
    Spacing,
    Comment,

    // Tokens with significant data
    Identifier,
}

#[derive(PartialEq, Debug)]
pub struct Token {
    pub tokentype: TokenType,
    pub source_span: source::SourceSpan,
}

impl Token {
    pub fn new(tokentype: TokenType, pos: u64, len: usize) -> Token {
        Token {
            tokentype: tokentype,
            source_span: source::SourceSpan { pos, len },
        }
    }
}

pub struct TokenDisplay<'a, S: Scanner> {
    pub token: &'a Token,
    pub scanner: &'a S,
}

impl<'a, S: Scanner> fmt::Debug for TokenDisplay<'a, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.token.tokentype {
            TokenType::Identifier => f
                .debug_tuple("Identifier")
                .field(&self.token.source_span.pos)
                .field(&self.token.source_span.len)
                .field(&self.scanner.get_token_source_string(self.token))
                .finish(),
            _ => {
                self.token.tokentype.fmt(f).unwrap();
                f.debug_tuple("")
                    .field(&self.token.source_span.pos)
                    .field(&self.token.source_span.len)
                    .finish()
            }
        }
    }
}
