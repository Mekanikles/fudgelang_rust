use super::*;
use crate::error;
use crate::source;
use std::assert;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

const ERROR_THRESHOLD: u32 = 5;

pub trait Scanner {
    fn get_token_source_string(&self, token: &Token) -> String;
    fn read_token(&mut self) -> Option<Token>;
}

pub struct ScannerImpl<'a, R: Read, S: source::Source<'a, R>> {
    source: &'a S,
    reader: source::LookAheadReader<R>,
    pub errors: Vec<error::Error>,
}

impl<'a, R: Read + Seek, S: source::Source<'a, R>> Scanner for ScannerImpl<'a, R, S> {
    fn get_token_source_string(&self, token: &Token) -> String {
        let mut reader = self.source.get_readable();
        match reader.seek(SeekFrom::Start(token.source_span.pos)) {
            Ok(_) => {
                let mut v: Vec<u8> = Vec::new();
                v.resize(token.source_span.len, 0);
                if let Ok(_) = reader.read(&mut v) {
                    return String::from_utf8_lossy(&v).to_string();
                }
            }
            Err(_) => (),
        }
        return "".into();
    }

    fn read_token(&mut self) -> Option<Token> {
        let mut error_count = 0;

        while let Some(n) = self.reader.peek() {
            // If we have reached the maximum allowed errors before producing a token
            //  consider the scan ended
            if error_count >= ERROR_THRESHOLD {
                return None;
            }

            // LL(1) tokens
            match n {
                b'.' => return Some(self.produce_token_and_advance(TokenType::Dot)),
                b',' => return Some(self.produce_token_and_advance(TokenType::Comma)),
                b';' => return Some(self.produce_token_and_advance(TokenType::SemiColon)),
                b'\t' => return Some(self.produce_token_and_advance(TokenType::Indent)),
                b'\n' => return Some(self.produce_linebreak()),
                b' ' => return Some(self.produce_spacing()),
                b'a'..=b'z' | b'A'..=b'Z' => return Some(self.produce_identifier()),
                _ => (),
            }

            let l = match self.reader.lookahead() {
                Some(n) => n,
                _ => 0,
            };

            // LL(2) tokens
            match n {
                b'/' => match l {
                    b'/' => return Some(self.produce_linecomment()),
                    b'*' => return Some(self.produce_blockcomment()),
                    _ => (),
                },
                b'*' if l == b'/' => {
                    self.log_error(error::new_unexpected_sequence_error(
                        self.reader.pos(),
                        2,
                        "Found stray block comment end".into(),
                    ));
                    self.reader.advance();
                    self.reader.advance();
                    error_count += 1;
                    continue;
                }
                _ => (),
            }

            // Non-ascii characters are not allowed outside of string literals
            if !(n).is_ascii() {
                let pos = self.reader.pos();
                let c = self.read_utf8_char();
                if c.is_none() {
                    self.log_error(error::new_non_utf8_sequence_error(
                        pos,
                        self.reader.pos() - pos,
                    ));
                } else {
                    self.log_error(error::new_non_ascii_char_error(c.unwrap(), pos));
                }
                error_count += 1;
                continue;
            } else {
                self.log_error(error::new_unexpected_char_error(
                    n as char,
                    self.reader.pos(),
                ));
                self.reader.advance();
                error_count += 1;
                continue;
            }
        }

        return None;
    }
}

impl<'a, R: Read + Seek, S: source::Source<'a, R>> ScannerImpl<'a, R, S> {
    pub fn new(source: &'a S) -> Self {
        ScannerImpl {
            source: source,
            reader: source::LookAheadReader::new(source.get_readable()),
            errors: Vec::new(),
        }
    }

    fn log_error(&mut self, error: error::Error) {
        self.errors.push(error);
    }

    fn read_utf8_char(&mut self) -> Option<char> {
        let controlbyte = self.reader.peek().unwrap();

        self.reader.advance();

        // Utf-8 is at max 4 bytes
        let mut buf: [u8; 4] = [controlbyte, 0, 0, 0];

        const CTRL_2_BYTE_MASK: u8 = 0b11100000;
        const CTRL_3_BYTE_MASK: u8 = 0b11110000;
        const CTRL_4_BYTE_MASK: u8 = 0b11111000;
        const CTRL_2_BYTE_VALUE: u8 = 0b11000000;
        const CTRL_3_BYTE_VALUE: u8 = 0b11100000;
        const CTRL_4_BYTE_VALUE: u8 = 0b11110000;

        // Check how long the sequence should be
        let bytecount;
        match controlbyte {
            n if n & CTRL_2_BYTE_MASK == CTRL_2_BYTE_VALUE => bytecount = 2,
            n if n & CTRL_3_BYTE_MASK == CTRL_3_BYTE_VALUE => bytecount = 3,
            n if n & CTRL_4_BYTE_MASK == CTRL_4_BYTE_VALUE => bytecount = 4,
            _ => return None,
        }

        for i in 1..bytecount {
            let n = self.reader.peek();
            self.reader.advance();
            match n {
                Some(n) => buf[i] = n,
                _ => return None,
            }
        }

        let res = std::str::from_utf8(&buf[0..bytecount]);
        if res.is_ok() {
            return res.unwrap().chars().nth(0);
        }

        return None;
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

    fn produce_linecomment(&mut self) -> Token {
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

        return Token::new(
            TokenType::Comment,
            startpos,
            (self.reader.pos() - startpos) as usize,
        );
    }

    fn produce_blockcomment(&mut self) -> Token {
        let startpos = self.reader.pos();
        self.reader.advance();
        self.reader.advance();

        // Eat block, including nested blocks
        let mut blocklevel = 1;
        while let Some(n) = self.reader.peek() {
            if n as char == '/' && self.reader.lookahead() == Some('*' as u8) {
                blocklevel += 1;
                self.reader.advance();
            } else if n as char == '*' && self.reader.lookahead() == Some('/' as u8) {
                blocklevel -= 1;
                self.reader.advance();
            }

            self.reader.advance();

            if blocklevel == 0 {
                break;
            }
        }

        assert!(
            blocklevel == 0,
            "Unexpected end of file inside block comment"
        );

        return Token::new(
            TokenType::Comment,
            startpos,
            (self.reader.pos() - startpos) as usize,
        );
    }

    fn produce_spacing(&mut self) -> Token {
        let startpos = self.reader.pos();
        self.reader.advance();

        while let Some(n) = self.reader.peek() {
            if n as char != ' ' {
                break;
            }
            self.reader.advance();
        }
        return Token::new(
            TokenType::Spacing,
            startpos,
            (self.reader.pos() - startpos) as usize,
        );
    }

    fn produce_identifier(&mut self) -> Token {
        let sourcepos = self.reader.pos();

        while let Some(n) = self.reader.peek() {
            if !(n as char).is_ascii_alphanumeric() {
                break;
            }
            self.reader.advance();
        }

        return Token::new(
            TokenType::Identifier,
            sourcepos,
            (self.reader.pos() - sourcepos) as usize,
        );
    }

    fn produce_token_and_advance(&mut self, tokentype: TokenType) -> Token {
        let token = Token::new(tokentype, self.reader.pos(), 1);
        self.reader.advance();
        return token;
    }
}
