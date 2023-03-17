use super::utils::*;
use crate::parser::ast::NodeId::*;

#[test]
fn test_symbol_subscript() {
    verify_ast(
        "a.b",
        &entrypoint_wrapper_tree(&[tree(SubScript, &[leaf(SymbolReference)])]),
    );
}

#[test]
fn test_nested_subscript() {
    verify_ast(
        "a.b.c",
        &entrypoint_wrapper_tree(&[tree(
            SubScript,
            &[tree(SubScript, &[leaf(SymbolReference)])],
        )]),
    );
}

#[test]
fn test_expression_subscript() {
    verify_ast(
        "(a+b).c",
        &entrypoint_wrapper_tree(&[tree(
            SubScript,
            &[tree(
                BinaryOperation,
                &[leaf(SymbolReference), leaf(SymbolReference)],
            )],
        )]),
    );
}

#[test]
fn test_subscript_call() {
    verify_ast(
        "a.b(c)",
        &entrypoint_wrapper_tree(&[tree(
            CallOperation,
            &[
                tree(SubScript, &[leaf(SymbolReference)]),
                tree(ArgumentList, &[leaf(SymbolReference)]),
            ],
        )]),
    );
}
