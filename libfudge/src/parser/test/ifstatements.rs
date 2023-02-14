use super::utils::*;
use crate::error::*;
use crate::parser::ast::NodeId::*;

#[test]
fn test_statement_if_empty() {
    verify_ast(
        "if a then\n\
        end",
        &entrypoint_wrapper_tree(&[tree(
            IfStatement,
            &[leaf(SymbolReference), leaf(StatementBody)],
        )]),
    );
}

#[test]
fn test_statement_if_empty_2() {
    verify_ast(
        "if a then end",
        &entrypoint_wrapper_tree(&[tree(
            IfStatement,
            &[leaf(SymbolReference), leaf(StatementBody)],
        )]),
    );
}

#[test]
fn test_wrong_statement_if_empty() {
    let s = "\
        if a\n\
        then\n\
        end";

    let result = generate_ast_with_errors(s, false);
    expect_error_ids(&result.1, &[new_error_id(errors::MismatchedAlignment)]);
}

#[test]
fn test_statement_if_empty_multiline_conditional() {
    verify_ast(
        "\
        if call(a,\n\
            \tb)\n\
        then\n\
        end",
        &entrypoint_wrapper_tree(&[tree(
            IfStatement,
            &[
                tree(
                    CallOperation,
                    &[
                        leaf(SymbolReference),
                        tree(
                            ArgumentList,
                            &[leaf(SymbolReference), leaf(SymbolReference)],
                        ),
                    ],
                ),
                leaf(StatementBody),
            ],
        )]),
    );
}

#[test]
fn test_statement_if_empty_wrong_multiline_conditional() {
    let s = "\
        if call(a,\n\
            \tb, c) then\n\
        end";

    let result = generate_ast_with_errors(s, false);
    expect_error_ids(&result.1, &[new_error_id(errors::MismatchedAlignment)]);
}

#[test]
fn test_statement_if_then_else_empty() {
    verify_ast(
        "if a then\nelse\nend",
        &entrypoint_wrapper_tree(&[tree(
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
fn test_statement_if_else_if_empty() {
    verify_ast(
        "if a then\nelseif b then\nend",
        &entrypoint_wrapper_tree(&[tree(
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
fn test_statement_if_else_if_else_empty() {
    verify_ast(
        "if a then\nelseif b then\nelse\nend",
        &entrypoint_wrapper_tree(&[tree(
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
fn test_statement_if_chained_else_if_empty() {
    verify_ast(
        "if a then\nelseif b then\nelseif c then\nelseif d then\nend",
        &entrypoint_wrapper_tree(&[tree(
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
fn test_statement_if_chained_else_if_else_empty() {
    verify_ast(
        "if a then\nelseif b then\nelseif c then\nelseif d then\nelse\nend",
        &entrypoint_wrapper_tree(&[tree(
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
fn test_statement_if_non_empty() {
    let blockversion = "if a then\n\tb\nend";

    verify_ast(
        blockversion,
        &entrypoint_wrapper_tree(&[tree(
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
    let blockversion = "if a then\n\tb\nelse\n\tb\nend";

    verify_ast(
        blockversion,
        &entrypoint_wrapper_tree(&[tree(
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
        "if a then\n\te\nelseif b then\n\te\nelseif c then\n\te\nelseif d then\n\te\nelse\n\te\nend";

    verify_ast(
        blockversion,
        &entrypoint_wrapper_tree(&[tree(
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
