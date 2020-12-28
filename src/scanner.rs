use std::io::Read;
use crate::source;
use crate::token::*;

pub struct Scanner<R : Read> {
    reader : source::SourceReader<R>,
}

impl<R : Read> Scanner<R> {
    pub fn new<'a>(source : &'a impl source::Source<'a, R>) -> Self {
        Scanner::<R> { 
            reader: source.get_reader()}
    }

     // Helper for producing 1-char token data, also advances reader
    fn produce_octokendata(&mut self) -> OCTokenData
    {
        let tokendata = OCTokenData(self.reader.pos());
        self.reader.advance();
        return tokendata;   
    }

    fn produce_spacing(&mut self) -> Token
    {
        let startpos = self.reader.pos();

        while let Some(n) = self.reader.peek() {
            if n as char != ' ' {
                break; 
            }
            self.reader.advance();
        }
        return Token::Spacing(NCTokenData(startpos, self.reader.pos() - startpos));
    }

    fn produce_identifier(&mut self) -> Token
    {
        let pos = self.reader.pos();
        let mut data = Vec::<u8>::new();

        while let Some(n) = self.reader.peek() {
            if !(n as char).is_ascii_alphanumeric() {
                break; 
            }
            data.push(n);
            self.reader.advance();
        }
 
        return Token::Identifier(IdentifierTokenData(pos, data));
    }

    pub fn read_token(&mut self) -> Option<Token>
    {
        while let Some(n) = self.reader.peek() {
            let c = n as char;

            // Non-ascii characters are not allowed outside of string literals
            if !(c).is_ascii() {
                panic!("Non-ascii character found!"); }

            // Control characters
            if (c).is_ascii_control() {
                let mut waslb = false;
                // Eat all control characters
                while let Some(n) = self.reader.peek() {
                    if !(n as char).is_ascii_control() {
                        break;
                    }
                    waslb |= (n as char) == '\n';
                    self.reader.advance();
                }
                
                // Treat all control character sequences containing a line break
                //  as a single line break
                if waslb {
                    return Some(Token::LineBreak(OCTokenData(self.reader.pos())));
                }

                break;
            }

            match c {
                '.' => return Some(Token::Dot(self.produce_octokendata())),
                ',' => return Some(Token::Comma(self.produce_octokendata())),
                '\t' => return Some(Token::Indent(self.produce_octokendata())),
                ' ' => return Some(self.produce_spacing()),
                'a'..='z' | 'A'..='Z' => return Some(self.produce_identifier()),
                _ => ()
            }

            println!("Error, found invalid byte '{}' at pos: {}", n, self.reader.pos());
            break;
        }

        return None;
    }
}

