use super::tokenstream::TokenStream;

pub struct Parser<'a, T: TokenStream>
{
    tokens: &'a mut T,
}

impl<'a, T: TokenStream> Parser<'a, T> {
    pub fn new(tokens: &'a mut T) -> Self {
        Parser {
            tokens: tokens
        }
    }

    pub fn parse(&mut self) {
        let mut tokencount = 0;
        while self.tokens.read_token().is_some() {
            tokencount += 1;
        }

        println!("Parsed {} tokens!", tokencount);
    }
}



