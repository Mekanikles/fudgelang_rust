use crate::scanner::Scanner;
use crate::scanner::Token;

use crate::error;

pub trait TokenStream {
    fn read_token(&mut self) -> Option<Token>;
    fn get_token_string(&self, token: &Token) -> String;

    fn get_errors(&self) -> &Vec<error::Error>;
}

// Implementation for Scanners
impl<T> TokenStream for T
where
    T: Scanner,
{
    fn read_token(&mut self) -> Option<Token> {
        return Scanner::read_token(self);
    }

    fn get_token_string(&self, token: &Token) -> String {
        return Scanner::get_token_source_string(self, token);
    }

    fn get_errors(&self) -> &Vec<error::Error> {
        return Scanner::get_errors(self);
    }
}
