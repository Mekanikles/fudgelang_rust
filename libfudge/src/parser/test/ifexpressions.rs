use super::utils::*;
use crate::error::*;
use crate::parser::ast::NodeId::*;

#[test]
fn test_expression_if_simple() {
    verify_ast(
        "def x = if a => b",
        &module_fragment_wrapper_tree(&[tree(
            SymbolDeclaration,
            &[tree(
                IfExpression,
                &[leaf(SymbolReference), leaf(SymbolReference)],
            )],
        )]),
    );
}

#[test]
fn test_expression_if_else_oneline() {
    verify_ast(
        "def x = if a => b else c",
        &module_fragment_wrapper_tree(&[tree(
            SymbolDeclaration,
            &[tree(
                IfExpression,
                &[
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                ],
            )],
        )]),
    );
}

#[test]
fn test_expression_if_elseif_else_oneline_1() {
    verify_ast(
        "def x = if a => b else if c => d else e",
        &module_fragment_wrapper_tree(&[tree(
            SymbolDeclaration,
            &[tree(
                IfExpression,
                &[
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    tree(
                        IfExpression,
                        &[
                            leaf(SymbolReference),
                            leaf(SymbolReference),
                            leaf(SymbolReference),
                        ],
                    ),
                ],
            )],
        )]),
    );
}

#[test]
fn test_expression_if_elseif_else_oneline_2() {
    verify_ast(
        "def x = if a => b elseif c => d else e",
        &module_fragment_wrapper_tree(&[tree(
            SymbolDeclaration,
            &[tree(
                IfExpression,
                &[
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                ],
            )],
        )]),
    );
}

#[test]
fn test_expression_if_simple_multiline_1() {
    verify_ast(
        "def x = if a =>\n\
            \tb",
        &module_fragment_wrapper_tree(&[tree(
            SymbolDeclaration,
            &[tree(
                IfExpression,
                &[leaf(SymbolReference), leaf(SymbolReference)],
            )],
        )]),
    );
}

#[test]
fn test_expression_if_simple_multiline_2() {
    verify_ast(
        "def x =\n\
            \tif a =>\n\
                \t\tb",
        &module_fragment_wrapper_tree(&[tree(
            SymbolDeclaration,
            &[tree(
                IfExpression,
                &[leaf(SymbolReference), leaf(SymbolReference)],
            )],
        )]),
    );
}

#[test]
fn test_expression_if_wrong_multiline_1() {
    let s = "\
        def x = if a =>\n\
        b";

    let result = generate_ast_with_errors(s, false);
    expect_error_ids(&result.1, &[new_error_id(errors::MismatchedIndentation)]);
}

#[test]
fn test_expression_if_wrong_multiline_2() {
    let s = "\
        def x =\n\
        if a => b";

    let result = generate_ast_with_errors(s, false);
    expect_error_ids(&result.1, &[new_error_id(errors::MismatchedIndentation)]);
}

#[test]
fn test_expression_if_wrong_multiline_3() {
    let s = "\
        def x = if a\n\
            \t=> b";

    let result = generate_ast_with_errors(s, false);
    expect_error_ids(&result.1, &[new_error_id(errors::MismatchedAlignment)]);
}

#[test]
fn test_expression_if_wrong_multiline_4() {
    let s = "\
        def x =\n\
            \tif a\n\
            \t=> b";

    let result = generate_ast_with_errors(s, false);
    expect_error_ids(&result.1, &[new_error_id(errors::MismatchedAlignment)]);
}

#[test]
fn test_expression_if_else_multiline_1() {
    verify_ast(
        "def x =\n\
            \tif a => b\n\
            \telse c",
        &module_fragment_wrapper_tree(&[tree(
            SymbolDeclaration,
            &[tree(
                IfExpression,
                &[
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                ],
            )],
        )]),
    );
}

#[test]
fn test_expression_if_else_multiline_2() {
    verify_ast(
        "def x =\n\
            \tif a =>\n\
                \t\tb\n\
            \telse\n\
                \t\tc",
        &module_fragment_wrapper_tree(&[tree(
            SymbolDeclaration,
            &[tree(
                IfExpression,
                &[
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                ],
            )],
        )]),
    );
}

#[test]
fn test_expression_wrong_if_else_multiline_1() {
    let s = "\
        def x = if a => b\n\
            \telse c";

    let result = generate_ast_with_errors(s, false);
    expect_error_ids(&result.1, &[new_error_id(errors::MismatchedAlignment)]);
}

#[test]
fn test_expression_wrong_if_else_multiline_2() {
    let s = "\
        def x = if a => b\n\
        else c";
    let result = generate_ast_with_errors(s, false);
    expect_error_ids(&result.1, &[new_error_id(errors::MismatchedAlignment)]);
}

#[test]
fn test_expression_if_else_if_else_multiline_1() {
    verify_ast(
        "def x =\n\
            \tif a => b\n\
            \telse\n\
            \t\tif c => d\n\
            \t\telse e",
        &module_fragment_wrapper_tree(&[tree(
            SymbolDeclaration,
            &[tree(
                IfExpression,
                &[
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    tree(
                        IfExpression,
                        &[
                            leaf(SymbolReference),
                            leaf(SymbolReference),
                            leaf(SymbolReference),
                        ],
                    ),
                ],
            )],
        )]),
    );
}

#[test]
fn test_expression_if_else_if_else_multiline_2() {
    verify_ast(
        "def x =\n\
            \tif a =>\n\
                \t\tb\n\
            \telse\n\
                \t\tif c =>\n\
                    \t\t\td\n\
                \t\telse\n\
                    \t\t\te",
        &module_fragment_wrapper_tree(&[tree(
            SymbolDeclaration,
            &[tree(
                IfExpression,
                &[
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    tree(
                        IfExpression,
                        &[
                            leaf(SymbolReference),
                            leaf(SymbolReference),
                            leaf(SymbolReference),
                        ],
                    ),
                ],
            )],
        )]),
    );
}

#[test]
fn test_expression_if_elseif_else_multiline_1() {
    verify_ast(
        "def x =\n\
            \tif a => b\n\
            \telseif c => d\n\
            \telse e",
        &module_fragment_wrapper_tree(&[tree(
            SymbolDeclaration,
            &[tree(
                IfExpression,
                &[
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                ],
            )],
        )]),
    );
}

#[test]
fn test_expression_if_elseif_else_multiline_2() {
    verify_ast(
        "def x =\n\
            \tif a =>\n\
                \t\tb\n\
            \telseif c =>\n\
                \t\td\n\
            \telse\n\
                \t\te",
        &module_fragment_wrapper_tree(&[tree(
            SymbolDeclaration,
            &[tree(
                IfExpression,
                &[
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                    leaf(SymbolReference),
                ],
            )],
        )]),
    );
}
