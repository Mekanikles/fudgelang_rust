use super::*;
use crate::error;
use crate::source;
use std::assert;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

use std::io::BufReader;
use std::io::BufRead;

const ERROR_THRESHOLD: usize = 5;

// TODO: This is not a good place for this, has nothing to do with the scanner
pub struct LineInfo {
    pub text : String,
    pub row : u32,
    pub column : u32,
}

pub trait Scanner {
    fn get_token_source_string(&self, token: &Token) -> String;
    fn get_line_info(&self, filepos : u64) -> Option<LineInfo>;
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

    fn get_line_info(&self, filepos : u64) -> Option<LineInfo> {
        let mut reader = BufReader::new(self.source.get_readable());
 
        let mut seekpos = 0;
        let mut row = 0;
        let mut text = String::new();
        while let Ok(bytes_read) = reader.read_line(&mut text) {
            if bytes_read == 0 {
                return None;
            }
            let eol = seekpos + bytes_read;
            if eol > filepos as usize
            {
                let column = text[..(filepos as usize - seekpos)].chars().count() as u32;
                return Some(LineInfo {
                    text,
                    row : row + 1,
                    column: column + 1,
                });
            }
            seekpos = eol;
            row += 1;
            text.clear();
        }
        return None;
    }

    fn read_token(&mut self) -> Option<Token> {
        let mut invalid_sequence_started = false;
        while let Some(n) = self.reader.peek() {
            // If we have reached the maximum allowed errors before producing a token
            //  consider the scan ended
            if self.errors.len() >= ERROR_THRESHOLD {
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
                _ => ()
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
                    continue;
                }
                _ => (),
            }

            // If we reach this point, we did not know what to do with this char
            //  do error handling best we can
            let pos = self.reader.pos();
            if let Some(c) = self.read_utf8_char_with_error() {
                // If this is the start of an "invalid" alphabetic utf8 sequence,
                //  treat it as an identifier
                if !invalid_sequence_started && !c.is_ascii() && c.is_alphabetic() {
                    return Some(self.produce_identifier_at_pos(pos));
                }

                if !invalid_sequence_started {
                    invalid_sequence_started = true;
                    self.log_error(error::new_invalid_sequence_error(
                        pos,
                        self.reader.pos() - pos,
                    ));
                }
                else
                {
                    self.adjust_last_error_end(self.reader.pos());
                }
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

    fn adjust_last_error_end(&mut self, end : u64) {
        let pos = self.errors.last_mut().unwrap().source_span.pos;
        self.errors.last_mut().unwrap().source_span.len = (end - pos) as usize;
    }

    fn read_utf8_char(&mut self) -> Option<char> {
        let controlbyte = self.reader.peek().unwrap();

        self.reader.advance();

        if controlbyte <= 127 {
            return Some(controlbyte as char);
        }

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

    fn read_utf8_char_with_error(&mut self) -> Option<char> {
        let pos = self.reader.pos();
        return match self.read_utf8_char() {
            Some(n) => Some(n),
            None => {
                self.log_error(error::new_non_utf8_sequence_error(pos, self.reader.pos() - pos));
                None
            }
        };
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

    // Produce identifier starting at supplied source pos, continuing at the readder pos
    fn produce_identifier_at_pos(&mut self, sourcepos : u64) -> Token {
        while let Some(n) = self.reader.peek() {
            if !(n as char).is_ascii_alphanumeric() {
                // If we are still ascii, this marks the end of a valid identifier
                if (n as char).is_ascii() {
                    break;
                }

                // Error recovery: advance until end of non-ascii utf8 sequence
                while let Some(n) = self.reader.peek() {
                    if (n as char).is_ascii() && !(n as char).is_ascii_alphanumeric() {
                        break;
                    }
                    
                    self.read_utf8_char_with_error();
                }

                self.log_error(error::new_non_ascii_identifier_error(sourcepos, self.reader.pos() - sourcepos));

                // In order to not give cascading errors in the parser, we still produce a token here
                return Token::new(
                    TokenType::Identifier,
                    sourcepos,
                    (self.reader.pos() - sourcepos) as usize,
                );
            }
            self.reader.advance();
        }

        // Everything was fine
        return Token::new(
            TokenType::Identifier,
            sourcepos,
            (self.reader.pos() - sourcepos) as usize,
        );
    }

    // Produce identifier at reader pos
    fn produce_identifier(&mut self) -> Token {
        return self.produce_identifier_at_pos(self.reader.pos());
    }  

    fn produce_token_and_advance(&mut self, tokentype: TokenType) -> Token {
        let token = Token::new(tokentype, self.reader.pos(), 1);
        self.reader.advance();
        return token;
    }
}
