use super::utils::*;
use crate::parser::ast::NodeId::*;

fn wrap_in_simple_function_literal(body: &str) -> String {
    return format!("func() do\n{}\nend", body);
}

fn wrap_in_function_literal_with_outparams(returntype: &str, body: &str) -> String {
    return format!("func() -> {} do\n{}\nend", returntype, body);
}

fn simple_function_literal_wrapper_tree(body: &[NodeIdTree]) -> NodeIdTree {
    return module_fragment_wrapper_tree(&[tree(FunctionLiteral, &[tree(StatementBody, body)])]);
}

fn function_literal_with_outparams_wrapper_tree(
    outparams: &[NodeIdTree],
    body: &[NodeIdTree],
) -> NodeIdTree {
    let mut subtree: Vec<NodeIdTree> = Vec::new();

    for p in outparams {
        subtree.push(tree(OutputParameter, &[p.clone()]));
    }

    subtree.push(tree(StatementBody, body));

    return tree(
        Module,
        &[tree(StatementBody, &[tree(FunctionLiteral, &subtree[..])])],
    );
}

#[test]
fn test_empty_function() {
    let source = wrap_in_simple_function_literal("");
    let expected = simple_function_literal_wrapper_tree(&[]);

    verify_ast(source.as_str(), &expected);
}

#[test]
fn test_function_with_empty_return() {
    let source = wrap_in_simple_function_literal("\treturn");
    let expected = simple_function_literal_wrapper_tree(&[leaf(ReturnStatement)]);

    verify_ast(source.as_str(), &expected);
}

#[test]
fn test_function_with_primitive_return() {
    let source = wrap_in_function_literal_with_outparams("#primitives.u32", "\treturn 42");
    let expected = function_literal_with_outparams_wrapper_tree(
        &[leaf(BuiltInObjectReference)],
        &[tree(ReturnStatement, &[leaf(IntegerLiteral)])],
    );

    verify_ast(source.as_str(), &expected);
}
