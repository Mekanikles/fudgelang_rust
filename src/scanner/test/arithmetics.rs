use super::*;

#[test]
fn test_single_character_operators() {
    verify_exact_scan("+", &[Token::new(TokenType::Plus, 0, 1)]);
    verify_exact_scan("-", &[Token::new(TokenType::Minus, 0, 1)]);
    verify_exact_scan("/", &[Token::new(TokenType::Slash, 0, 1)]);
    verify_exact_scan("*", &[Token::new(TokenType::Star, 0, 1)]);
    verify_exact_scan("=", &[Token::new(TokenType::Equals, 0, 1)]);
}
