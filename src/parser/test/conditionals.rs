use super::utils::*;
use crate::parser::ast::NodeId::*;

#[test]
fn test_statement_empty_if() {
    verify_ast(
        "if a then\nend",
        &module_fragment_wrapper_tree(&[tree(
            IfStatement,
            &[leaf(SymbolReference), leaf(StatementBody)],
        )]),
    );
}

#[test]
fn test_statement_empty_if_then_else_empty() {
    verify_ast(
        "if a then\nelse then\nend",
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
fn test_statement_empty_if_else_if() {
    verify_ast(
        "if a then\nelse if b then\nend",
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
fn test_statement_empty_if_else_if_else() {
    verify_ast(
        "if a then\nelse if b then\nelse then\nend",
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
fn test_statement_empty_if_chained_else_if() {
    verify_ast(
        "if a then\nelse if b then\nelse if c then\nelse if d then\nend",
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
fn test_statement_empty_if_chained_else_if_else() {
    verify_ast(
        "if a then\nelse if b then\nelse if c then\nelse if d then\nelse then\nend",
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
fn test_statement_non_empty_if() {
    let blockversion = "if a then\n\tb\nend";

    verify_ast(
        blockversion,
        &module_fragment_wrapper_tree(&[tree(
            IfStatement,
            &[
                leaf(SymbolReference),
                tree(StatementBody, &[leaf(SymbolReference)]),
            ],
        )]),
    );
}

#[test]
fn test_statement_non_empty_if_else() {
    let blockversion = "if a then\n\tb\nelse then\n\tb\nend";

    verify_ast(
        blockversion,
        &module_fragment_wrapper_tree(&[tree(
            IfStatement,
            &[
                leaf(SymbolReference),
                tree(StatementBody, &[leaf(SymbolReference)]),
                tree(StatementBody, &[leaf(SymbolReference)]),
            ],
        )]),
    );
}

#[test]
fn test_statement_non_empty_if_chained_else_if_else() {
    let blockversion =
        "if a then\n\te\nelse if b then\n\te\nelse if c then\n\te\nelse if d then\n\te\nelse then\n\te\nend";

    verify_ast(
        blockversion,
        &module_fragment_wrapper_tree(&[tree(
            IfStatement,
            &[
                leaf(SymbolReference),
                tree(StatementBody, &[leaf(SymbolReference)]),
                leaf(SymbolReference),
                tree(StatementBody, &[leaf(SymbolReference)]),
                leaf(SymbolReference),
                tree(StatementBody, &[leaf(SymbolReference)]),
                leaf(SymbolReference),
                tree(StatementBody, &[leaf(SymbolReference)]),
                tree(StatementBody, &[leaf(SymbolReference)]),
            ],
        )]),
    );
}
