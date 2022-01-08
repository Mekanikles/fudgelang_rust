use crate::scanner::Token;
use crate::scanner::Scanner;

pub trait TokenStream {
    fn read_token(&mut self) -> Option<Token>;
    fn get_token_string(&self, token: &Token) -> String;
}

// Implemtation for Scanners
impl<T> TokenStream for T where T: Scanner {
    fn read_token(&mut self) -> Option<Token> {
        return Scanner::read_token(self);
    }

    fn get_token_string(&self, token: &Token) -> String {
        return Scanner::get_token_source_string(self, token);
    }
}


