use super::*;

#[test]
fn test_basic() {
    verify_exact_scan("0", &[Token::new(TokenType::NumericLiteral, 0, 1)]);
}

#[test]
fn test_decimal() {
    verify_exact_scan("10", &[Token::new(TokenType::NumericLiteral, 0, 2)]);
}

#[test]
fn test_decimal_with_underscore() {
    verify_exact_scan("1_0", &[Token::new(TokenType::NumericLiteral, 0, 3)]);
}

#[test]
fn test_hexadecimal() {
    verify_exact_scan("0xFF", &[Token::new(TokenType::NumericLiteral, 0, 4)]);
}

#[test]
fn test_hexadecimal_with_underscore() {
    verify_exact_scan("0xF_F", &[Token::new(TokenType::NumericLiteral, 0, 5)]);
}

#[test]
fn test_binary() {
    verify_exact_scan("0b10", &[Token::new(TokenType::NumericLiteral, 0, 4)]);
}

#[test]
fn test_binary_with_underscore() {
    verify_exact_scan("0b1_0", &[Token::new(TokenType::NumericLiteral, 0, 5)]);
}

#[test]
fn test_octal() {
    verify_exact_scan("0o10", &[Token::new(TokenType::NumericLiteral, 0, 4)]);
}

#[test]
fn test_octal_with_underscore() {
    verify_exact_scan("0o1_0", &[Token::new(TokenType::NumericLiteral, 0, 5)]);
}

#[test]
fn test_float() {
    verify_exact_scan("0.0", &[Token::new(TokenType::NumericLiteral, 0, 3)]);
}

#[test]
fn test_float_with_undercore() {
    verify_exact_scan("0.0_0", &[Token::new(TokenType::NumericLiteral, 0, 5)]);
}

#[test]
fn test_float_with_exponent() {
    verify_exact_scan("0.0e10", &[Token::new(TokenType::NumericLiteral, 0, 6)]);
    verify_exact_scan("0.0E10", &[Token::new(TokenType::NumericLiteral, 0, 6)]);
}

#[test]
fn test_float_with_exponent_and_sign() {
    verify_exact_scan("0.0e+10", &[Token::new(TokenType::NumericLiteral, 0, 7)]);
    verify_exact_scan("0.0e-10", &[Token::new(TokenType::NumericLiteral, 0, 7)]);
    verify_exact_scan("0.0E+10", &[Token::new(TokenType::NumericLiteral, 0, 7)]);
    verify_exact_scan("0.0E-10", &[Token::new(TokenType::NumericLiteral, 0, 7)]);
}

#[test]
fn test_float_with_signed_exponent_and_operator() {
    verify_exact_scan(
        "0.0e+10+5",
        &[
            Token::new(TokenType::NumericLiteral, 0, 7),
            Token::new(TokenType::Plus, 7, 1),
            Token::new(TokenType::NumericLiteral, 8, 1),
        ],
    );
}

#[test]
fn test_float_and_operator() {
    verify_exact_scan(
        "0.0+5",
        &[
            Token::new(TokenType::NumericLiteral, 0, 3),
            Token::new(TokenType::Plus, 3, 1),
            Token::new(TokenType::NumericLiteral, 4, 1),
        ],
    );
}
