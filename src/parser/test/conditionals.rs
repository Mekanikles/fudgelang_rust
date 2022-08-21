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
fn test_statement_if_then_else_empty() {
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
fn test_statement_if_else_if() {
    verify_ast(
        "if a then\nelse if b\nend",
        &module_fragment_wrapper_tree(&[tree(
            IfStatement,
            &[
                leaf(SymbolReference),
                leaf(StatementBody),
                leaf(SymbolReference),
                leaf(StatementBody),
            ],
        )]),
    );
}

#[test]
fn test_statement_if_else_if_else() {
    verify_ast(
        "if a then\nelse if b\nelse\nend",
        &module_fragment_wrapper_tree(&[tree(
            IfStatement,
            &[
                leaf(SymbolReference),
                leaf(StatementBody),
                leaf(SymbolReference),
                leaf(StatementBody),
                leaf(StatementBody),
            ],
        )]),
    );
}

#[test]
fn test_statement_if_chained_else_if() {
    verify_ast(
        "if a then\nelse if b\nelse if c\nelse if d\nend",
        &module_fragment_wrapper_tree(&[tree(
            IfStatement,
            &[
                leaf(SymbolReference),
                leaf(StatementBody),
                leaf(SymbolReference),
                leaf(StatementBody),
                leaf(SymbolReference),
                leaf(StatementBody),
                leaf(SymbolReference),
                leaf(StatementBody),
            ],
        )]),
    );
}

#[test]
fn test_statement_if_chained_else_if_else() {
    verify_ast(
        "if a then\nelse if b\nelse if c\nelse if d\nelse\nend",
        &module_fragment_wrapper_tree(&[tree(
            IfStatement,
            &[
                leaf(SymbolReference),
                leaf(StatementBody),
                leaf(SymbolReference),
                leaf(StatementBody),
                leaf(SymbolReference),
                leaf(StatementBody),
                leaf(SymbolReference),
                leaf(StatementBody),
                leaf(StatementBody),
            ],
        )]),
    );
}

#[test]
fn test_expression_if_simple() {
    verify_ast(
        "if a 0 else 2",
        &module_fragment_wrapper_tree(&[tree(
            IfExpression,
            &[
                leaf(SymbolReference),
                leaf(IntegerLiteral),
                leaf(IntegerLiteral),
            ],
        )]),
    );
}

#[test]
fn test_expression_if_else_if_else() {
    verify_ast(
        "if a 0 else if b 1 else 2",
        &module_fragment_wrapper_tree(&[tree(
            IfExpression,
            &[
                leaf(SymbolReference),
                leaf(IntegerLiteral),
                leaf(SymbolReference),
                leaf(IntegerLiteral),
                leaf(IntegerLiteral),
            ],
        )]),
    );
}

#[test]
fn test_expression_if_chained_else_if_else() {
    verify_ast(
        "if a 0 else if b 1 else if c 2 else 3",
        &module_fragment_wrapper_tree(&[tree(
            IfExpression,
            &[
                leaf(SymbolReference),
                leaf(IntegerLiteral),
                leaf(SymbolReference),
                leaf(IntegerLiteral),
                leaf(SymbolReference),
                leaf(IntegerLiteral),
                leaf(IntegerLiteral),
            ],
        )]),
    );
}
