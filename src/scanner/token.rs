use super::*;
use crate::source;
use std::fmt;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum TokenType {
    // 1-char tokens
    Comma,
    Dot,
    Colon,
    SemiColon,
    Equals,
    Hash,
    OpeningParenthesis,
    ClosingParenthesis,
    OpeningSquareBracket,
    ClosingSquareBracket,
    OpeningCurlyBrace,
    ClosingCurlyBrace,
    Arrow,
    Plus,
    Minus,
    Slash,
    Star,

    // n-char tokens
    LineBreak,
    Indentation,
    Comment,

    // Keywords
    If,
    Then,
    Else,
    True,
    False,
    Def,
    Var,
    Func,
    Do,
    End,
    Return,

    // Tokens with significant data
    Identifier,
    StringLiteral,
    CharacterLiteral,
    NumericLiteral,
}

#[derive(PartialEq, Debug, Clone, Copy)]
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

// Debug formatter
impl<'a, S: Scanner> fmt::Debug for TokenDisplay<'a, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.token.tokentype {
            TokenType::Identifier
            | TokenType::NumericLiteral
            | TokenType::StringLiteral
            | TokenType::CharacterLiteral => {
                self.token.tokentype.fmt(f).unwrap();
                f.debug_tuple("")
                    .field(&self.token.source_span.pos)
                    .field(&self.token.source_span.len)
                    .field(&self.scanner.get_token_source_string(self.token))
                    .finish()
            }
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
