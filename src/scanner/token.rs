use super::*;
use std::io::Read;
use std::fmt;

#[derive(PartialEq)]
#[derive(Debug)]
pub struct OCTokenData(pub u64);

#[derive(PartialEq)]
#[derive(Debug)]
pub struct NCTokenData(pub u64, pub u64);

#[derive(PartialEq)]
#[derive(Debug)]
pub struct IdentifierTokenData(pub u64, pub usize, pub usize);

#[derive(PartialEq)]
#[derive(Debug)]
pub enum Token
{
    // 1-char tokens, has pos
    Comma(OCTokenData), 
    Dot(OCTokenData),
    SemiColon(OCTokenData),
    Indent(OCTokenData), 
    LineBreak(OCTokenData),

    // n-char tokens, has pos + length
    Spacing(NCTokenData), Comment(NCTokenData),

    // Identifier tokens, has pos + identifier
    Identifier(IdentifierTokenData)
}

pub struct TokenDisplay<'a, R : Read>
{
    pub token : &'a Token,
    pub scanner : &'a Scanner<R>,
}

impl<'a, R : Read> fmt::Debug for TokenDisplay<'a, R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.token {
            Token::Identifier(n) => {
                f.debug_tuple("Identifier")
                 .field(&self.scanner.resolve_identifier(n.1, n.2))
                 .finish()
            },
            _ => self.token.fmt(f)
        }
    }
}