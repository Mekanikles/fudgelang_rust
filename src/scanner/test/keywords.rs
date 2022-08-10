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
fn test_nondelimited_keywords() {
    verify_exact_scan("ifdef", &[Token::new(TokenType::Identifier, 0, 5)]);
}

#[test]
fn test_delimited_keywords() {
    verify_exact_scan(
        "if def",
        &[
            Token::new(TokenType::If, 0, 2),
            Token::new(TokenType::Def, 3, 3),
        ],
    );
}

#[test]
fn test_all_keywords() {
    verify_exact_scan(
        "if def func do end",
        &[
            Token::new(TokenType::If, 0, 2),
            Token::new(TokenType::Def, 3, 3),
            Token::new(TokenType::Func, 7, 4),
            Token::new(TokenType::Do, 12, 2),
            Token::new(TokenType::End, 15, 3),
        ],
    );
}
