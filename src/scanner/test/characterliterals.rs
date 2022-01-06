use super::*;

#[test]
fn test_empty() {
    verify_exact_scan("\'\'", &[Token::new(TokenType::CharacterLiteral, 0, 2)]);
}

#[test]
fn test_basic() {
    verify_exact_scan("\'c\'", &[Token::new(TokenType::CharacterLiteral, 0, 3)]);
}

#[test]
fn test_escape() {
    verify_exact_scan("\'\\0\'", &[Token::new(TokenType::CharacterLiteral, 0, 4)]);
}
