use super::*;
use std::fmt;
use std::convert::Into;

#[derive(PartialEq)]
#[derive(Debug)]
pub enum TokenType
{
    // 1-char tokens
    Comma, Dot, SemiColon, Indent, LineBreak,

    // n-char tokens
    Spacing, Comment,

    // Tokens with significant data
    Identifier
}

#[derive(PartialEq)]
pub struct Token
{
    pub tokentype: TokenType,
    pub source_pos: u64,
    pub source_len: usize,
}

impl Token {
    pub fn new(tokentype: TokenType, pos: u64, len: usize) -> Token {
        Token {
            tokentype: tokentype, 
            source_pos: pos,
            source_len: len 
        }
    }
}

pub struct TokenDisplay<'a, S: Scanner>
{
    pub token: &'a Token,
    pub scanner: &'a S,
}

impl<'a, S: Scanner> fmt::Debug for TokenDisplay<'a, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.token.tokentype {
            TokenType::Identifier => {
                f.debug_tuple("Identifier")
                 .field(&self.token.source_pos)
                 .field(&self.token.source_len)
                 .field(&self.scanner.get_token_source_string(self.token))
                 .finish()
            },
            _ => { 
                self.token.tokentype.fmt(f).unwrap();
                f.debug_tuple("")
                 .field(&self.token.source_pos)
                 .field(&self.token.source_pos)
                 .finish()
            }
        }
    }
}