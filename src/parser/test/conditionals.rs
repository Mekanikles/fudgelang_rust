use super::utils::*;
use crate::parser::ast::NodeId::*;

#[test]
fn test_statement_if_empty() {
    verify_ast(
        "if a then\nend",
        &module_fragment_wrapper_tree(&[tree(
            IfStatement,
            &[leaf(SymbolReference), leaf(StatementBody)],
        )]),
    );
}

#[test]
fn test_if_then_empty() {
    verify_ast(
        "if a then\nend",
        &module_fragment_wrapper_tree(&[tree(
            IfStatement,
            &[leaf(SymbolReference), leaf(StatementBody)],
        )]),
    );
}

#[test]
fn test_if_then_else_empty() {
    verify_ast(
        "if a then\nelse\nend",
        &module_fragment_wrapper_tree(&[tree(
            IfStatement,
            &[
                leaf(SymbolReference),
                leaf(StatementBody),
                leaf(StatementBody),
            ],
        )]),
    );
}

#[test]
fn test_if_expr() {
    verify_ast(
        "if a b",
        &module_fragment_wrapper_tree(&[tree(
            IfStatement,
            &[leaf(SymbolReference), leaf(SymbolReference)],
        )]),
    );
}

#[test]
fn test_if_expr_else() {
    verify_ast(
        "if a b else c",
        &module_fragment_wrapper_tree(&[tree(
            IfStatement,
            &[
                leaf(SymbolReference),
                leaf(SymbolReference),
                leaf(SymbolReference),
            ],
        )]),
    );
}
