use super::*;

#[test]
fn test_parentheses_1() {
    verify_exact_scan("()", &[
        Token::new(TokenType::OpeningParenthesis, 0, 1),
        Token::new(TokenType::ClosingParenthesis, 1, 1)]);
}

#[test]
fn test_parentheses_2() {
    verify_sparse_scan("_(_)_", &[
        Token::new(TokenType::OpeningParenthesis, 1, 1),
        Token::new(TokenType::ClosingParenthesis, 3, 1)]);
}

#[test]
fn test_square_brackets_1() {
    verify_exact_scan("[]", &[
        Token::new(TokenType::OpeningSquareBracket, 0, 1),
        Token::new(TokenType::ClosingSquareBracket, 1, 1)]);
}

#[test]
fn test_square_brackets_2() {
    verify_sparse_scan("_[_]_", &[
        Token::new(TokenType::OpeningSquareBracket, 1, 1),
        Token::new(TokenType::ClosingSquareBracket, 3, 1)]);
}

#[test]
fn test_curly_braces_1() {
    verify_exact_scan("{}", &[
        Token::new(TokenType::OpeningCurlyBrace, 0, 1),
        Token::new(TokenType::ClosingCurlyBrace, 1, 1)]);
}

#[test]
fn test_curly_braces_2() {
    verify_sparse_scan("_{_}_", &[
        Token::new(TokenType::OpeningCurlyBrace, 1, 1),
        Token::new(TokenType::ClosingCurlyBrace, 3, 1)]);
}

#[test]
fn test_mix_brackets() {
    verify_exact_scan("([({()})])", &[
        Token::new(TokenType::OpeningParenthesis, 0, 1),
        Token::new(TokenType::OpeningSquareBracket, 1, 1),
        Token::new(TokenType::OpeningParenthesis, 2, 1),
        Token::new(TokenType::OpeningCurlyBrace, 3, 1),
        Token::new(TokenType::OpeningParenthesis, 4, 1),
        Token::new(TokenType::ClosingParenthesis, 5, 1),
        Token::new(TokenType::ClosingCurlyBrace, 6, 1),
        Token::new(TokenType::ClosingParenthesis, 7, 1),
        Token::new(TokenType::ClosingSquareBracket, 8, 1),
        Token::new(TokenType::ClosingParenthesis, 9, 1)]);
}
