use super::utils::*;
use crate::error::*;

pub fn verify_exact_errors(source: &str, expected_errors: &[ErrorId]) {
    let (_, errors) = generate_ast_with_errors(source, expected_errors.len() == 0);

    let error_ids = errors.iter().map(|x| x.id).collect::<Vec<_>>();
    assert_eq!(error_ids, expected_errors);
}

pub fn verify_no_errors(source: &str) {
    verify_exact_errors(source, &[]);
}

#[test]
fn test_indentation_first_line() {
    verify_exact_errors("\ta", &[new_error_id(errors::MismatchedIndentation)]);
}

#[test]
fn test_single_line() {
    let s = "\
        call(1 + 2 + 4 + 5)";
    verify_no_errors(s);
}

#[test]
fn test_multi_line_1() {
    let s = "\
        call(1 + 2 +\n\
            \t3 + 4)";
    verify_no_errors(s);
}

#[test]
fn test_multi_line_2() {
    let s = "\
        call(1 +\n\
            \t2 +\n\
            \t3 +\n\
            \t4)";
    verify_no_errors(s);
}

#[test]
fn test_wrong_multi_line_1() {
    let s = "\
        call(1 + 2 +\n\
        3 + 4)";
    verify_exact_errors(s, &[new_error_id(errors::MismatchedIndentation)]);
}

#[test]
fn test_wrong_multi_line_2() {
    let s = "\
        call(1 + 2 +\n\
                \t\t3 + 4)";
    verify_exact_errors(s, &[new_error_id(errors::MismatchedIndentation)]);
}

#[test]
fn test_wrong_multi_line_3() {
    let s = "\
        call(1 +\n\
            \t2 +\n\
            \t3 +\n\
        4)";
    verify_exact_errors(s, &[new_error_id(errors::MismatchedIndentation)]);
}

#[test]
fn test_multiple_statements() {
    verify_no_errors(
        "\
        a\n\
        b\n",
    );
}

#[test]
fn test_multiple_statements_same_line() {
    let s = "\
        a b\n";
    verify_exact_errors(s, &[new_error_id(errors::ExpectedNewLine)]);
}

#[test]
fn test_statement_after_expression() {
    verify_no_errors(
        "\
        def a = b\n\
        c\n",
    );
}

#[test]
fn test_statement_after_expression_same_line() {
    let s = "\
        def a = b c\n";
    verify_exact_errors(s, &[new_error_id(errors::ExpectedNewLine)]);
}

#[test]
fn test_empty_vertical_block() {
    let s = "\
        if a then\n\
        end";
    verify_no_errors(s);
}

#[test]
fn test_wrong_empty_vertical_block() {
    let s = "\
        if a then end";
    verify_exact_errors(s, &[new_error_id(errors::ExpectedNewLine)]);
}

#[test]
fn test_wrong_vertical_block() {
    let s = "\
        if a then b end";
    verify_exact_errors(
        s,
        &[
            new_error_id(errors::ExpectedNewLine),
            new_error_id(errors::ExpectedNewLine),
        ],
    );
}

#[test]
fn test_vertical_block_indentation() {
    let s = "\
        if a then\n\
            \ta\n\
        end";
    verify_no_errors(s);
}

#[test]
fn test_vertical_block_wrong_indentation_1() {
    let s = "\
        if a then\n\
        a\n\
        end";
    verify_exact_errors(s, &[new_error_id(errors::MismatchedIndentation)]);
}

#[test]
fn test_vertical_block_wrong_indentation_2() {
    verify_exact_errors(
        "if a then\n\
                \t\ta\n\
        end",
        &[new_error_id(errors::MismatchedIndentation)],
    );
}

#[test]
fn test_vertical_block_indentation_multiple_statements() {
    verify_no_errors(
        "if a then\n\
            \tb\n\
            \tc\n\
	    end",
    );
}

#[test]
fn test_vertical_block_padding() {
    verify_exact_errors(
        "if a then\n\
            \t  a\n\
        end",
        &[
            new_error_id(errors::PaddingNotSupported),
            new_error_id(errors::MismatchedIndentation),
        ],
    );
}
