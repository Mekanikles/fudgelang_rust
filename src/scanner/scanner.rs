use super::*;
use crate::error;
use crate::source;
use phf::phf_map;
use std::debug_assert;

use crate::source::LookAheadSourceReader;
use crate::source::SourceSpan;

// Map with all scannable keywords
pub static KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "if" => TokenType::If,
    "then" => TokenType::Then,
    "else" => TokenType::Else,
    "elseif" => TokenType::ElseIf,
    "true" => TokenType::True,
    "false" => TokenType::False,
    "def" => TokenType::Def,
    "var" => TokenType::Var,
    "func" => TokenType::Func,
    "do" => TokenType::Do,
    "end" => TokenType::End,
    "return" => TokenType::Return,
};

pub struct ScannerResult {
    pub tokens: Vec<Token>,
    pub errors: Vec<error::Error>,
}

struct Scanner<'a> {
    reader: source::LookAheadSourceReader<'a>,
    allow_indentation: bool,
    output_padding: bool,
    errors: error::ErrorManager,
}

fn guess_token_count(source: &source::Source) -> usize {
    // Guess based on the average length of a token, then double that for good measure
    return (source.data().len() / 3) * 2;
}

// Tokenize entire source
pub fn tokenize(source: &source::Source) -> ScannerResult {
    let mut tokens: Vec<Token> = Vec::new();
    tokens.reserve(guess_token_count(&source));

    let mut scanner = Scanner::new(&source);

    while let Some(n) = scanner.read_token() {
        tokens.push(n);
    }

    return ScannerResult {
        tokens: tokens,
        errors: scanner.errors.error_data.errors,
    };
}

impl<'a> Scanner<'a> {
    fn new(source: &'a source::Source) -> Scanner<'a> {
        Scanner {
            reader: LookAheadSourceReader::new(&source),
            allow_indentation: false,
            output_padding: false,
            errors: error::ErrorManager::new(),
        }
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
                    self.errors
                        .log_error(error::new_unexpected_indentation_error(
                            indentation_token.source_span.pos,
                            indentation_token.source_span.len as u64,
                        ));
                    continue;
                } else {
                    return Some(indentation_token);
                }
            } else if self.allow_indentation {
                self.allow_indentation = false;
                self.output_padding = true;
            }

            // Handle padding and whitespace
            if n == b' ' {
                if self.output_padding {
                    // Padding is only allowed directly after indendation
                    let padding_token = self.produce_padding();
                    self.output_padding = false;
                    return Some(padding_token);
                } else {
                    // Consume unimportant whitespace
                    self.consume_spacing();
                    continue;
                }
            } else {
                self.output_padding = false;
            }

            // Produce token
            match n {
                b'.' => return Some(self.produce_token_and_advance(TokenType::Dot)),
                b',' => return Some(self.produce_token_and_advance(TokenType::Comma)),
                b':' => return Some(self.produce_token_and_advance(TokenType::Colon)),
                b'=' => match self.reader.lookahead() {
                    Some(b'=') => {
                        return Some(self.produce_token_and_advance_n(TokenType::CompareEq, 2));
                    }
                    Some(b'>') => {
                        return Some(self.produce_token_and_advance_n(TokenType::FatArrow, 2));
                    }
                    _ => return Some(self.produce_token_and_advance(TokenType::Equals)),
                },
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
                b'>' => match self.reader.lookahead() {
                    Some(b'=') => {
                        return Some(
                            self.produce_token_and_advance_n(TokenType::GreaterThanOrEq, 2),
                        )
                    }
                    _ => return Some(self.produce_token_and_advance(TokenType::GreaterThan)),
                },
                b'<' => match self.reader.lookahead() {
                    Some(b'=') => {
                        return Some(self.produce_token_and_advance_n(TokenType::LessThanOrEq, 2))
                    }
                    _ => return Some(self.produce_token_and_advance(TokenType::LessThan)),
                },
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

    fn produce_padding(&mut self) -> Token {
        debug_assert!(self.reader.peek().unwrap() == b' ');
        let startpos = self.reader.pos();
        self.consume_spacing();

        return Token::new(
            TokenType::Padding,
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
            self.reader
                .source()
                .get_source_string(&SourceSpan {
                    pos: sourcepos,
                    len: (self.reader.pos() - sourcepos) as usize,
                })
                .to_string(),
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
