use super::utils::*;
use crate::error::*;

pub fn verify_exact_errors(source: &str, expected_errors: &[ErrorId]) {
    let (_, errors) = generate_ast_with_errors(source, false);

    let error_ids = errors.iter().map(|x| x.id).collect::<Vec<_>>();
    assert_eq!(error_ids, expected_errors);
}

#[test]
fn test_indentation_first_line() {
    verify_exact_errors("\ta", &[new_error_id(errors::MismatchedIndentation)]);
}

#[test]
fn test_padding_first_line() {
    verify_exact_errors("  a", &[new_error_id(errors::MismatchedPadding)]);
}

#[test]
fn test_expression_single_line() {
    let s = "\
        call(1 + 2 + 4 + 5)";
    verify_exact_errors(s, &[]);
}

#[test]
fn test_expression_multi_line_1() {
    let s = "\
        call(1 + 2 +\n\
            \t3 + 4)";
    verify_exact_errors(s, &[]);
}

#[test]
fn test_expression_multi_line_2() {
    let s = "\
        call(1 +\n\
            \t2 +\n\
            \t3 +\n\
            \t4)";
    verify_exact_errors(s, &[]);
}

#[test]
fn test_vertical_block_indentation() {
    let s = "\
        if a then\n\
            \ta\n\
        end";
    verify_exact_errors(s, &[]);
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
        "if a then\n\t\ta\nend",
        &[new_error_id(errors::MismatchedIndentation)],
    );
}

#[test]
fn test_vertical_block_wrong_padding() {
    verify_exact_errors(
        "if a then\n\t  a\nend",
        &[new_error_id(errors::MismatchedPadding)],
    );
}
