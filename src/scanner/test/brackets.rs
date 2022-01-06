use super::*;

#[test]
fn test_parentheses_1() {
    verify_exact_scan("()", &[
        Token::new(TokenType::LeftParenthesis, 0, 1),
        Token::new(TokenType::RightParenthesis, 1, 1)]);
}

#[test]
fn test_parentheses_2() {
    verify_sparse_scan("_(_)_", &[
        Token::new(TokenType::LeftParenthesis, 1, 1),
        Token::new(TokenType::RightParenthesis, 3, 1)]);
}

#[test]
fn test_square_brackets_1() {
    verify_exact_scan("[]", &[
        Token::new(TokenType::LeftSquareBracket, 0, 1),
        Token::new(TokenType::RightSquareBracket, 1, 1)]);
}

#[test]
fn test_square_brackets_2() {
    verify_sparse_scan("_[_]_", &[
        Token::new(TokenType::LeftSquareBracket, 1, 1),
        Token::new(TokenType::RightSquareBracket, 3, 1)]);
}

#[test]
fn test_curly_braces_1() {
    verify_exact_scan("{}", &[
        Token::new(TokenType::LeftCurlyBrace, 0, 1),
        Token::new(TokenType::RightCurlyBrace, 1, 1)]);
}

#[test]
fn test_curly_braces_2() {
    verify_sparse_scan("_{_}_", &[
        Token::new(TokenType::LeftCurlyBrace, 1, 1),
        Token::new(TokenType::RightCurlyBrace, 3, 1)]);
}

#[test]
fn test_mix_brackets() {
    verify_exact_scan("([({()})])", &[
        Token::new(TokenType::LeftParenthesis, 0, 1),
        Token::new(TokenType::LeftSquareBracket, 1, 1),
        Token::new(TokenType::LeftParenthesis, 2, 1),
        Token::new(TokenType::LeftCurlyBrace, 3, 1),
        Token::new(TokenType::LeftParenthesis, 4, 1),
        Token::new(TokenType::RightParenthesis, 5, 1),
        Token::new(TokenType::RightCurlyBrace, 6, 1),
        Token::new(TokenType::RightParenthesis, 7, 1),
        Token::new(TokenType::RightSquareBracket, 8, 1),
        Token::new(TokenType::RightParenthesis, 9, 1)]);
}
