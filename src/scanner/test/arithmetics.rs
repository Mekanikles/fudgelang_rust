use super::*;

#[test]
fn test_single_character_operators() {
    verify_exact_scan("+", &[Token::new(TokenType::Plus, 0, 1)]);
    verify_exact_scan("-", &[Token::new(TokenType::Minus, 0, 1)]);
    verify_exact_scan("/", &[Token::new(TokenType::Slash, 0, 1)]);
    verify_exact_scan("*", &[Token::new(TokenType::Star, 0, 1)]);
    verify_exact_scan("=", &[Token::new(TokenType::Equals, 0, 1)]);
}

#[test]
fn test_parentheses_1() {
    verify_exact_scan("()", &[
        Token::new(TokenType::LeftParenthesis, 0, 1),
        Token::new(TokenType::RightParenthesis, 1, 1)]);
}

#[test]
fn test_parentheses_2() {
    verify_exact_scan("((()))", &[
        Token::new(TokenType::LeftParenthesis, 0, 1),
        Token::new(TokenType::LeftParenthesis, 1, 1),
        Token::new(TokenType::LeftParenthesis, 2, 1),
        Token::new(TokenType::RightParenthesis, 3, 1),
        Token::new(TokenType::RightParenthesis, 4, 1),
        Token::new(TokenType::RightParenthesis, 5, 1)]);
}

#[test]
fn test_parentheses_3() {
    verify_sparse_scan("_(_)_", &[
        Token::new(TokenType::LeftParenthesis, 1, 1),
        Token::new(TokenType::RightParenthesis, 3, 1)]);
}