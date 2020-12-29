use super::*;

#[test]
fn test_line_trivial() {
    verify_exact_scan("//", &[
        Token::Comment(NCTokenData(1, 2)) ]);
}

#[test]
fn test_line_simple() {
    verify_exact_scan("// Comment", &[
        Token::Comment(NCTokenData(1, 10)) ]);
}

#[test]
fn test_line_edges() {
    verify_exact_scan(".// Comment\n", &[
        Token::Dot(OCTokenData(1)),
        Token::Comment(NCTokenData(2, 10)),
        Token::LineBreak(OCTokenData(12)) ]);
}

#[test]
fn test_line_partial() {
    verify_sparse_scan("text// Comment\ntext", &[
        Token::Comment(NCTokenData(5, 10)) ]);
}

#[test]
fn test_line_nested() {
    verify_sparse_scan("text// Comment // Comment\ntext", &[
        Token::Comment(NCTokenData(5, 21)) ]);
}

#[test]
fn test_line_multiple() {
    verify_sparse_scan("text// Comment\n// Comment\ntext", &[
        Token::Comment(NCTokenData(5, 10)),
        Token::Comment(NCTokenData(16, 10)) ]);
}

#[test]
fn test_block_trivial() {
    verify_exact_scan("/**/", &[
        Token::Comment(NCTokenData(1, 4)) ]);
}

#[test]
#[should_panic]
fn test_block_incomplete() {
    verify_exact_scan("/*", &[]);
}

#[test]
#[should_panic]
fn test_block_strayopen() {
    verify_exact_scan("*/", &[]);
}

#[test]
fn test_block_simple() {
    verify_exact_scan("/* Comment */", &[
        Token::Comment(NCTokenData(1, 13)) ]);
}

#[test]
fn test_block_edges() {
    verify_exact_scan("./* Comment */.", &[
        Token::Dot(OCTokenData(1)),
        Token::Comment(NCTokenData(2, 13)),
        Token::Dot(OCTokenData(15)) ]);
}

#[test]
fn test_block_partial() {
    verify_sparse_scan("text/* Comment */text", &[
        Token::Comment(NCTokenData(5, 13)) ]);
}

#[test]
fn test_block_nested_trivial() {
    verify_exact_scan("/*/**/*/", &[
        Token::Comment(NCTokenData(1, 8)) ]);
}

#[test]
#[should_panic]
fn test_block_nested_incomplete() {
    verify_exact_scan("/*/**/", &[
        Token::Comment(NCTokenData(1, 8)) ]);
}


