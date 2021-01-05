use super::*;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::assert;
use crate::source;

pub trait Scanner
{
    fn get_token_source_string(&self, token: &Token) -> String;
    fn read_token(&mut self) -> Option<Token>;
}

pub struct ScannerImpl<'a, R: Read, S: source::Source<'a, R>> {
    source: &'a S,
    reader: source::LookAheadReader<R>,
}

impl<'a, R: Read + Seek, S: source::Source<'a, R>> Scanner for ScannerImpl<'a, R, S> {
    fn get_token_source_string(&self, token: &Token) -> String
    {
        let mut reader = self.source.get_readable();
        match reader.seek(SeekFrom::Start(token.source_pos)) {
            Ok(_) => {
                let mut v: Vec<u8> = Vec::new();
                v.resize(token.source_len, 0);
                if let Ok(_) = reader.read(&mut v) {
                    return String::from_utf8_lossy(&v).to_string();
                }
            },
            Err(_) => ()
        }
        return "".into();
    }

    fn read_token(&mut self) -> Option<Token>
    {
        while let Some(n) = self.reader.peek() {
            let c = n as char;

            // Non-ascii characters are not allowed outside of string literals
            if !(c).is_ascii() {
                panic!("Non-ascii character found!"); }

            // LL(1) tokens
            match c {
                '.' => return Some(self.produce_token_and_advance(TokenType::Dot)),
                ',' => return Some(self.produce_token_and_advance(TokenType::Comma)),
                ';' => return Some(self.produce_token_and_advance(TokenType::SemiColon)),
                '\t' => return Some(self.produce_token_and_advance(TokenType::Indent)),
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
                '*' if l == '/' => panic!("Stray closing comment found!"),
                _ => ()
            }

            panic!("Error, found unrecognized character '{}' at pos: {}", n as char, self.reader.pos());
        }

        return None;
    }
}

impl<'a, R: Read + Seek, S: source::Source<'a, R>> ScannerImpl<'a, R, S> {
    pub fn new(source: &'a S) -> Self {
        ScannerImpl { 
            source: source,
            reader: source::LookAheadReader::new(source.get_readable()),
        }
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

        return Token::new(TokenType::LineBreak, pos, 1);
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

        return Token::new(TokenType::Comment, startpos, (self.reader.pos() - startpos) as usize);
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

        return Token::new(TokenType::Comment, startpos, (self.reader.pos() - startpos) as usize);
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
        return Token::new(TokenType::Spacing, startpos, (self.reader.pos() - startpos) as usize);
    }

    fn produce_identifier(&mut self) -> Token
    {
        let sourcepos = self.reader.pos();

        while let Some(n) = self.reader.peek() {
            if !(n as char).is_ascii_alphanumeric() {
                break; 
            }
            self.reader.advance();
        }

        return Token::new(TokenType::Identifier, sourcepos, (self.reader.pos() - sourcepos) as usize)
    }

    fn produce_token_and_advance(&mut self, tokentype: TokenType) -> Token
    {
        let token = Token::new(tokentype, self.reader.pos(), 1); 
        self.reader.advance();
        return token;   
    }
}
