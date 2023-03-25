use super::utils::*;
use crate::ast::NodeId::*;

#[test]
fn test_declare_empty_struct() {
    verify_ast(
        "\
        struct begin\n\
        end",
        &entrypoint_wrapper_tree(&[leaf(StructLiteral)]),
    );
}

#[test]
fn test_declare_simple_struct() {
    verify_ast(
        "\
        struct begin\n\
            \tvar b : u32\n\
        end",
        &entrypoint_wrapper_tree(&[tree(
            StructLiteral,
            &[tree(StructField, &[leaf(SymbolReference)])],
        )]),
    );
}
