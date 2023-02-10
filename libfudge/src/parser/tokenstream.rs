use crate::scanner::Token;
use crate::source;

pub struct TokenStream<'a> {
    tokens: &'a Vec<Token>,
    source: &'a source::Source,
    count: usize,
}

impl<'a> TokenStream<'a> {
    pub fn new(tokens: &'a Vec<Token>, source: &'a source::Source) -> TokenStream<'a> {
        return TokenStream {
            tokens,
            source,
            count: 0,
        };
    }

    pub fn get_source_name(&self) -> &str {
        return self.source.name();
    }

    pub fn read_token(&mut self) -> Option<&Token> {
        self.count += 1;
        return self.tokens.get(self.count - 1);
    }
    pub fn get_token_string(&self, token: &Token) -> &str {
        return std::str::from_utf8(self.source.get_span(&token.source_span)).unwrap();
    }
}
