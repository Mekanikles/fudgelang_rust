use super::*;

#[test]
fn test_single_character_operators() {
    verify_exact_scan("+", &[Token::new(TokenType::Plus, 0, 1)]);
    verify_exact_scan("-", &[Token::new(TokenType::Minus, 0, 1)]);
    verify_exact_scan("/", &[Token::new(TokenType::Slash, 0, 1)]);
    verify_exact_scan("*", &[Token::new(TokenType::Star, 0, 1)]);
    verify_exact_scan("=", &[Token::new(TokenType::Equals, 0, 1)]);
    verify_exact_scan(">", &[Token::new(TokenType::GreaterThan, 0, 1)]);
    verify_exact_scan("<", &[Token::new(TokenType::LessThan, 0, 1)]);
}

#[test]
fn test_multi_character_operators() {
    verify_exact_scan("==", &[Token::new(TokenType::CompareEq, 0, 2)]);
    verify_exact_scan(">=", &[Token::new(TokenType::GreaterThanOrEq, 0, 2)]);
    verify_exact_scan("<=", &[Token::new(TokenType::LessThanOrEq, 0, 2)]);
}
