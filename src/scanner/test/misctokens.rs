use super::*;

#[test]
fn test_single_character_tokens() {
    verify_exact_scan(":", &[Token::new(TokenType::Colon, 0, 1)]);
    verify_exact_scan(";", &[Token::new(TokenType::SemiColon, 0, 1)]);
}

#[test]
fn test_n_character_tokens() {
    verify_exact_scan("->", &[Token::new(TokenType::Arrow, 0, 2)]);
}