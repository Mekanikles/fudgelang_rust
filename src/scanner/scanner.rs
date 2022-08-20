use super::*;
use crate::error;
use crate::source;
use phf::phf_map;
use std::debug_assert;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;

use std::io::BufRead;
use std::io::BufReader;

// Map with all scannable keywords
static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "if" => TokenType::If,
    "then" => TokenType::Then,
    "else" => TokenType::Else,
    "true" => TokenType::True,
    "false" => TokenType::False,
    "def" => TokenType::Def,
    "func" => TokenType::Func,
    "do" => TokenType::Do,
    "end" => TokenType::End,
    "return" => TokenType::Return,
};

// TODO: This is not a good place for this, has nothing to do with the scanner
pub struct LineInfo {
    pub text: String,
    pub line_start: usize,
    pub row: u32,
}

pub trait Scanner {
    fn get_errors(&self) -> &Vec<error::Error>;
    fn get_token_source_string(&self, token: &Token) -> String;
    fn get_line_info(&self, filepos: u64) -> Option<LineInfo>;
    fn read_token(&mut self) -> Option<Token>;
}

pub struct ScannerImpl<'a, R: Read, S: source::Source<'a, R>> {
    source: &'a S,
    reader: source::LookAheadReader<R>,
    allow_indentation: bool,
    errors: error::ErrorManager,
}

impl<'a, R: Read + Seek, S: source::Source<'a, R>> Scanner for ScannerImpl<'a, R, S> {
    fn get_errors(&self) -> &Vec<error::Error> {
        return self.errors.get_errors();
    }

    fn get_token_source_string(&self, token: &Token) -> String {
        return self.get_source_string(token.source_span.pos, token.source_span.len);
    }

    fn get_line_info(&self, filepos: u64) -> Option<LineInfo> {
        let mut reader = BufReader::new(self.source.get_readable());

        let mut seekpos = 0;
        let mut row = 0;
        let mut text = String::new();
        while let Ok(bytes_read) = reader.read_line(&mut text) {
            if bytes_read == 0 {
                return None;
            }
            let eol = seekpos + bytes_read;
            if eol > filepos as usize {
                return Some(LineInfo {
                    text,
                    line_start: seekpos,
                    row: row + 1,
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
            if self.errors.reached_error_limit() {
                return None;
            }

            // Indentation management
            if n == b'\t' {
                let indentation_token = self.produce_indentation();
                if !self.allow_indentation {
                    self.errors.log_error(error::new_invalid_indentation_error(
                        indentation_token.source_span.pos,
                        indentation_token.source_span.len as u64,
                    ));
                    continue;
                } else {
                    return Some(indentation_token);
                }
            } else if self.allow_indentation {
                self.allow_indentation = false;
            }

            // Consume unimportant whitespace
            if n == b' ' {
                self.consume_spacing();
                continue;
            }

            // Produce token
            match n {
                b'.' => return Some(self.produce_token_and_advance(TokenType::Dot)),
                b',' => return Some(self.produce_token_and_advance(TokenType::Comma)),
                b':' => return Some(self.produce_token_and_advance(TokenType::Colon)),
                b'=' => return Some(self.produce_token_and_advance(TokenType::Equals)),
                b'+' => return Some(self.produce_token_and_advance(TokenType::Plus)),
                b'-' => match self.reader.lookahead() {
                    Some(b'>') => {
                        return Some(self.produce_token_and_advance_n(TokenType::Arrow, 2))
                    }
                    _ => return Some(self.produce_token_and_advance(TokenType::Minus)),
                },
                b'(' => return Some(self.produce_token_and_advance(TokenType::OpeningParenthesis)),
                b')' => return Some(self.produce_token_and_advance(TokenType::ClosingParenthesis)),
                b'[' => {
                    return Some(self.produce_token_and_advance(TokenType::OpeningSquareBracket))
                }
                b']' => {
                    return Some(self.produce_token_and_advance(TokenType::ClosingSquareBracket))
                }
                b'{' => return Some(self.produce_token_and_advance(TokenType::OpeningCurlyBrace)),
                b'}' => return Some(self.produce_token_and_advance(TokenType::ClosingCurlyBrace)),
                b';' => return Some(self.produce_token_and_advance(TokenType::SemiColon)),
                b'#' => return Some(self.produce_token_and_advance(TokenType::Hash)),
                b'/' => match self.reader.lookahead() {
                    Some(b'/') => return Some(self.produce_linecomment()),
                    Some(b'*') => return Some(self.produce_blockcomment()),
                    _ => return Some(self.produce_token_and_advance(TokenType::Slash)),
                },
                b'*' => match self.reader.lookahead() {
                    Some(b'/') => {
                        self.errors.log_error(error::new_unexpected_sequence_error(
                            self.reader.pos(),
                            2,
                            "Found stray block comment end".into(),
                        ));
                        self.reader.advance();
                        self.reader.advance();
                        continue;
                    }
                    _ => return Some(self.produce_token_and_advance(TokenType::Star)),
                },
                b'\n' => {
                    // TODO: Eat windows-style line breaks as one token
                    self.allow_indentation = true;
                    return Some(self.produce_token_and_advance(TokenType::LineBreak));
                }
                b'\"' => return Some(self.produce_stringliteral()),
                b'\'' => return Some(self.produce_characterliteral()),
                b'0'..=b'9' => return Some(self.produce_numericliteral()),
                b'a'..=b'z' | b'A'..=b'Z' | b'_' => return Some(self.produce_identifier()),
                _ => (),
            }

            // If we reach this point, we did not know what to do with this char
            //  do error handling best we can
            let pos = self.reader.pos();
            if let Some(c) = self.read_utf8_char_with_error() {
                // If this is the start of an "invalid" alphabetic utf8 sequence,
                //  treat it as an identifier
                if !invalid_sequence_started && !c.is_ascii() && c.is_alphabetic() {
                    return Some(self.produce_non_ascii_identifier(pos));
                }

                if !invalid_sequence_started {
                    invalid_sequence_started = true;
                    self.errors.log_error(error::new_invalid_sequence_error(
                        pos,
                        self.reader.pos() - pos,
                    ));
                } else {
                    // TODO: Check that last error len matches current posision
                    //  there might be spacing in-between
                    self.errors.adjust_last_error_end(self.reader.pos());
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
            allow_indentation: true,
            errors: error::ErrorManager::new(),
        }
    }

    fn get_source_string(&self, pos: u64, len: usize) -> String {
        let mut reader = self.source.get_readable();
        match reader.seek(SeekFrom::Start(pos)) {
            Ok(_) => {
                let mut v: Vec<u8> = Vec::new();
                v.resize(len, 0);
                if let Ok(_) = reader.read(&mut v) {
                    return String::from_utf8_lossy(&v).to_string();
                }
            }
            Err(_) => (),
        }
        return "".into();
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
                self.errors.log_error(error::new_non_utf8_sequence_error(
                    pos,
                    self.reader.pos() - pos,
                ));
                None
            }
        };
    }

    fn produce_linecomment(&mut self) -> Token {
        let startpos = self.reader.pos();
        debug_assert!(self.reader.peek().unwrap() == b'/');
        self.reader.advance();
        debug_assert!(self.reader.peek().unwrap() == b'/');
        self.reader.advance();

        // Eat until line break
        while let Some(n) = self.reader.peek() {
            if n == b'\n' {
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
        debug_assert!(self.reader.peek().unwrap() == b'/');
        self.reader.advance();
        debug_assert!(self.reader.peek().unwrap() == b'*');
        self.reader.advance();

        // Eat block, including nested blocks
        let mut blocklevel = 1;
        while let Some(n) = self.reader.peek() {
            if n == b'/' && self.reader.lookahead() == Some('*' as u8) {
                blocklevel += 1;
                self.reader.advance();
            } else if n == b'*' && self.reader.lookahead() == Some('/' as u8) {
                blocklevel -= 1;
                self.reader.advance();
            }

            self.reader.advance();

            if blocklevel == 0 {
                break;
            }
        }

        if blocklevel != 0 {
            // TODO: Add error reference to start of comment
            self.errors.log_error(error::new_unexpected_eof_error(
                self.reader.pos(),
                "Unexpected end of file inside block comment".into(),
            ));
        }

        return Token::new(
            TokenType::Comment,
            startpos,
            (self.reader.pos() - startpos) as usize,
        );
    }

    fn consume_spacing(&mut self) {
        debug_assert!(self.reader.peek().unwrap() == b' ');
        self.reader.advance();

        while let Some(n) = self.reader.peek() {
            if n != b' ' {
                break;
            }
            self.reader.advance();
        }
    }

    fn produce_indentation(&mut self) -> Token {
        let startpos = self.reader.pos();
        debug_assert!(self.reader.peek().unwrap() == b'\t');
        self.reader.advance();

        while let Some(n) = self.reader.peek() {
            if n != b'\t' {
                break;
            }
            self.reader.advance();
        }
        return Token::new(
            TokenType::Indentation,
            startpos,
            (self.reader.pos() - startpos) as usize,
        );
    }

    // Produces an "invalid" identifier and logs an error
    fn produce_non_ascii_identifier(&mut self, sourcepos: u64) -> Token {
        // Error recovery: advance until end of non-ascii utf8 sequence
        while let Some(n) = self.reader.peek() {
            if (n as char).is_ascii() && !(n as char).is_ascii_alphanumeric() {
                break;
            }

            self.read_utf8_char_with_error();
        }

        self.errors.log_error(error::new_non_ascii_identifier_error(
            sourcepos,
            self.reader.pos() - sourcepos,
            self.get_source_string(sourcepos, (self.reader.pos() - sourcepos) as usize),
        ));

        return Token::new(
            TokenType::Identifier,
            sourcepos,
            (self.reader.pos() - sourcepos) as usize,
        );
    }

    // Produce identifier starting at supplied source pos, continuing at the reader pos
    fn produce_identifier_at_pos(&mut self, sourcepos: u64) -> Token {
        // TODO: This seems unnecessary
        let mut string: String = String::new();

        while let Some(n) = self.reader.peek() {
            if !(n as char).is_ascii_alphanumeric() && n != b'_' {
                // If we are still ascii, this marks the end of a valid identifier
                if (n as char).is_ascii() {
                    break;
                }

                // In order to not give cascading errors in the parser, we still produce a token here
                return self.produce_non_ascii_identifier(sourcepos);
            }
            string.push(n as char);
            self.reader.advance();
        }

        // Everything was fine, check if this was a keyword
        if let Some(tokentype) = KEYWORDS.get(&string) {
            return Token::new(
                *tokentype,
                sourcepos,
                (self.reader.pos() - sourcepos) as usize,
            );
        }

        // If not a keyword, it is an identifier
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

    fn produce_stringliteral(&mut self) -> Token {
        let startpos = self.reader.pos();
        debug_assert!(self.reader.peek().unwrap() == b'\"');
        self.reader.advance();

        while self.reader.peek().filter(|c| *c != b'\"').is_some() {
            self.reader.advance();
        }

        // TODO: Add error reference to start of literal
        if self.reader.peek().is_none() {
            self.errors.log_error(error::new_unexpected_eof_error(
                self.reader.pos(),
                "Unexpected end of file inside string literal".into(),
            ));
        } else {
            self.reader.advance();
        }

        return Token::new(
            TokenType::StringLiteral,
            startpos,
            (self.reader.pos() - startpos) as usize,
        );
    }

    fn produce_characterliteral(&mut self) -> Token {
        let startpos = self.reader.pos();
        debug_assert!(self.reader.peek().unwrap() == b'\'');
        self.reader.advance();

        while self.reader.peek().filter(|c| *c != b'\'').is_some() {
            self.reader.advance();
        }

        // TODO: Add error reference to start of literal
        if self.reader.peek().is_none() {
            self.errors.log_error(error::new_unexpected_eof_error(
                self.reader.pos(),
                "Unexpected end of file inside character literal".into(),
            ));
        } else {
            self.reader.advance();
        }

        return Token::new(
            TokenType::CharacterLiteral,
            startpos,
            (self.reader.pos() - startpos) as usize,
        );
    }

    fn produce_numericliteral(&mut self) -> Token {
        let startpos = self.reader.pos();
        debug_assert!(self.reader.peek().unwrap().is_ascii_digit());
        self.reader.advance();

        // Note: we eat all trailing alphanumericals in this function, parsing of the
        //  actual number and error reporting happens when constructing AST

        while self
            .reader
            .peek()
            .filter(|c| c.is_ascii_alphanumeric() || *c == b'.')
            .is_some()
        {
            self.reader.advance();
        }

        return Token::new(
            TokenType::NumericLiteral,
            startpos,
            (self.reader.pos() - startpos) as usize,
        );
    }

    fn produce_token_and_advance(&mut self, tokentype: TokenType) -> Token {
        let token = Token::new(tokentype, self.reader.pos(), 1);
        self.reader.advance();
        return token;
    }

    fn produce_token_and_advance_n(&mut self, tokentype: TokenType, len: usize) -> Token {
        let token = Token::new(tokentype, self.reader.pos(), len);
        for _ in 0..len {
            self.reader.advance();
        }
        return token;
    }
}
