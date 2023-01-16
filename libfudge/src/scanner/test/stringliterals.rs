use super::*;

#[test]
fn test_empty() {
    verify_exact_scan("\"\"", &[Token::new(TokenType::StringLiteral, 0, 2)]);
}

#[test]
fn test_basic() {
    verify_exact_scan("\"Hello\"", &[Token::new(TokenType::StringLiteral, 0, 7)]);
}

#[test]
fn test_escape() {
    verify_exact_scan(
        "\"Hello \\0 \"",
        &[Token::new(TokenType::StringLiteral, 0, 11)],
    );
}
