use super::*;

#[test]
fn test_basic() {
    verify_exact_scan("0", &[Token::new(TokenType::NumericLiteral, 0, 1)]);
}

#[test]
fn test_hexadecimal() {
    verify_exact_scan("0xFF", &[Token::new(TokenType::NumericLiteral, 0, 4)]);
}

#[test]
fn test_binary() {
    verify_exact_scan("0b10", &[Token::new(TokenType::NumericLiteral, 0, 4)]);
}

#[test]
fn test_float() {
    verify_exact_scan("0.0", &[Token::new(TokenType::NumericLiteral, 0, 3)]);
}
