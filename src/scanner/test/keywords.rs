use super::*;

#[test]
fn test_simple() {
    verify_exact_scan("if", &[Token::new(TokenType::If, 0, 2)]);
}

#[test]
fn test_case() {
    verify_exact_scan("IF", &[Token::new(TokenType::Identifier, 0, 2)]);
}

#[test]
fn test_substring_keyword() {
    verify_exact_scan("iff", &[Token::new(TokenType::Identifier, 0, 3)]);
}

#[test]
fn test_all_keywords() {
    verify_exact_scan("if", &[Token::new(TokenType::If, 0, 2)]);
}
