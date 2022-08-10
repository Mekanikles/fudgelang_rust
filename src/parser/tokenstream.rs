use crate::scanner::Scanner;
use crate::scanner::Token;

pub trait TokenStream {
    fn read_token(&mut self) -> Option<Token>;
    fn get_token_string(&self, token: &Token) -> String;
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
}