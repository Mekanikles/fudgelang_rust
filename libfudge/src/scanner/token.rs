use crate::source;
use std::fmt;
use std::str;

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
    FatArrow,
    Plus,
    Minus,
    Slash,
    Star,
    GreaterThan,
    LessThan,

    // n-char tokens
    LineBreak,
    Indentation,
    Comment,
    CompareEq,
    GreaterThanOrEq,
    LessThanOrEq,

    // Keywords
    Module,
    If,
    Then,
    Else,
    ElseIf,
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

pub struct TokenDisplay<'a> {
    pub token: &'a Token,
    pub source: &'a source::Source,
}

// Debug formatter
impl<'a> fmt::Debug for TokenDisplay<'a> {
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
                    .field(&str::from_utf8(&self.source.get_span(&self.token.source_span)).unwrap())
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
