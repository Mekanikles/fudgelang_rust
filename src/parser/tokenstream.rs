use crate::scanner::Token;
use crate::scanner::Scanner;

pub trait TokenStream {
    fn read_token(&mut self) -> Option<Token>; 
}

// Implemtation for Scanners
impl<T> TokenStream for T where T: Scanner {
    fn read_token(&mut self) -> Option<Token> {
        return Scanner::read_token(self);
    }
}


