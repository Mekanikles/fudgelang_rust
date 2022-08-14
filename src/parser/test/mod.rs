use super::ast;
use super::ast::NodeInfo;

use crate::parser::*;
use crate::scanner::*;
use crate::source::*;

use crate::output;

fn verify_ast(source: &str, expected: &NodeIdTree) {
    let source = MemorySource::from_str(source);

    let mut scanner = ScannerImpl::new(&source);
    let mut parser = parser::Parser::new(&mut scanner);
    parser.parse();

    let ast = &parser.ast;

    assert!(ast.get_root().is_some());
    let rootref = ast.get_root().unwrap();
  
    // Rust cannot do recursive lambdas, booh
    fn record_tree_recursively(ast: &ast::Ast, noderef: &ast::NodeRef) -> NodeIdTree
    { 
        let mut this = NodeIdTree { id: ast.get_node(noderef).id(), children: Vec::new() };
        ast::visit_children(ast.get_node(noderef), | noderef | {
            this.children.push(record_tree_recursively(ast, noderef));
        });
        return this;
    }

    let tree = record_tree_recursively(ast, &rootref);

    // TODO: Have to print in this order, since parse borrows scanner
    output::print_errors(parser.get_errors(), &source);
    output::print_errors(scanner.get_errors(), &source);

    assert_eq!(tree, *expected);
}

#[derive(Clone, PartialEq, Debug)]
struct NodeIdTree
{
    id: ast::NodeId,
    children: Vec<NodeIdTree>,
}

fn tree(id: ast::NodeId, children: &[NodeIdTree]) -> NodeIdTree {
    NodeIdTree { id, children: children.to_vec() }
}

fn leaf(id: ast::NodeId) -> NodeIdTree {
    NodeIdTree { id, children: Vec::new() }
}

fn wrap_in_simple_function_literal(body: &str) -> String {
    return format!("func() do\n{}\nend", body);
}

fn simple_function_literal_wrapper_tree(body: &[NodeIdTree]) -> NodeIdTree {
    return tree(ModuleFragment, &[tree(StatementBody, &[
        tree(FunctionLiteral, &[
            tree(StatementBody, body)
        ])
    ])]);
}

use ast::NodeId::*;

#[test]
fn test_empty_function() {
    let source = wrap_in_simple_function_literal("");
    let expected = simple_function_literal_wrapper_tree(&[]);

    verify_ast(source.as_str(), &expected);
}

#[test]
fn test_function_with_empty_return() {
    let source = wrap_in_simple_function_literal(
        "\treturn");
    let expected = simple_function_literal_wrapper_tree(&[leaf(ReturnStatement)]);

    verify_ast(source.as_str(), &expected);
}
