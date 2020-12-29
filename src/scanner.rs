use std::io::Read;
use std::assert;
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

    fn produce_linebreak(&mut self) -> Token {
        let pos = self.reader.pos();
        self.reader.advance();

        // Eat all control characters
        while let Some(n) = self.reader.peek() {
            if !(n as char).is_ascii_control() {
                break;
            }
            self.reader.advance();
        }

        return Token::LineBreak(OCTokenData(pos));
    }

    fn produce_linecomment(&mut self) -> Token
    {
        let startpos = self.reader.pos();
        self.reader.advance();
        self.reader.advance();

        // Eat until line break
        while let Some(n) = self.reader.peek() {
            if n as char == '\n' {
                break;
            }
            self.reader.advance();
        }

        return Token::Comment(NCTokenData(startpos, self.reader.pos() - startpos));
    }

    fn produce_blockcomment(&mut self) -> Token
    {
        let startpos = self.reader.pos();
        self.reader.advance();
        self.reader.advance();

        // Eat block, including nested blocks
        let mut blocklevel = 1;
        while let Some(n) = self.reader.peek() {
            if n as char == '/' && self.reader.lookahead() == Some('*' as u8) {
                blocklevel += 1;
                self.reader.advance();
            }
            else if n as char == '*' && self.reader.lookahead() == Some('/' as u8) {
                blocklevel -= 1;
                self.reader.advance();
            }    

            self.reader.advance();

            if blocklevel == 0 {
                break;
            }           
        }

        assert!(blocklevel == 0, "Unexpected end of file inside block comment");

        return Token::Comment(NCTokenData(startpos, self.reader.pos() - startpos));
    }

    fn produce_spacing(&mut self) -> Token
    {
        let startpos = self.reader.pos();
        self.reader.advance();

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

    // Helper for producing 1-char token data, also advances reader
    fn produce_oc_tokendata(&mut self) -> OCTokenData
    {
        let tokendata = OCTokenData(self.reader.pos());
        self.reader.advance();
        return tokendata;   
    }
    
    pub fn read_token(&mut self) -> Option<Token>
    {
        while let Some(n) = self.reader.peek() {
            let c = n as char;

            // Non-ascii characters are not allowed outside of string literals
            if !(c).is_ascii() {
                panic!("Non-ascii character found!"); }

            // LL(1) tokens
            match c {
                '.' => return Some(Token::Dot(self.produce_oc_tokendata())),
                ',' => return Some(Token::Comma(self.produce_oc_tokendata())),
                '\t' => return Some(Token::Indent(self.produce_oc_tokendata())),
                '\n' => return Some(self.produce_linebreak()),
                ' ' => return Some(self.produce_spacing()),
                'a'..='z' | 'A'..='Z' => return Some(self.produce_identifier()),
                _ => ()
            }

            let l = match self.reader.lookahead() {
                Some(n) => n as char,
                _ => 0 as char
            };

            // LL(2) tokens
            match c {
                '/' => match l {
                    '/' => return Some(self.produce_linecomment()),
                    '*' => return Some(self.produce_blockcomment()),
                    _ => ()
                }
                _ => ()
            }

            println!("Error, found unrecognized character '{}' at pos: {}", n as char, self.reader.pos());
            break;
        }

        return None;
    }
}

