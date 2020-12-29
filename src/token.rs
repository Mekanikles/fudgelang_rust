use std::fmt;

#[derive(Debug)]
pub struct OCTokenData(pub u64);

#[derive(Debug)]
pub struct NCTokenData(pub u64, pub u64);

pub struct IdentifierTokenData(pub u64, pub Vec<u8>);
impl fmt::Debug for IdentifierTokenData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "'{}'", &String::from_utf8(self.1.clone()).unwrap())
    }
}

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
