use crate::parser::ast;
use crate::parser::ast::NodeInfo;

use crate::error;
use crate::parser::*;
use crate::scanner::*;
use crate::source::*;

use crate::output;

use crate::parser::tokenstream::TokenStream;
use std::fmt;

// TODO: Copy of function in scanner test utils, move to common utils file
pub fn expect_error_ids(errors: &Vec<error::Error>, expected_error_ids: &[error::ErrorId]) {
    assert_eq!(
        errors.len(),
        expected_error_ids.len(),
        "Found a different error count than expected!"
    );

    for i in 0..expected_error_ids.len() {
        assert_eq!(errors[i].id, expected_error_ids[i]);
    }
}

pub fn generate_ast_with_errors(source: &str, print_errors: bool) -> (ast::Ast, Vec<error::Error>) {
    let source = Source::from_str(source);

    let scanner_result = scanner::tokenize(&source);
    let parser_result = parser::parse(&mut TokenStream::new(&scanner_result.tokens, &source));

    if print_errors {
        output::print_errors(&scanner_result.errors, &source);
        output::print_errors(&parser_result.errors, &source);
    }

    let mut errors: Vec<error::Error> = Vec::new();

    errors.append(&mut scanner_result.errors.to_owned());
    errors.append(&mut parser_result.errors.to_owned());

    return (parser_result.ast, errors);
}

pub fn generate_ast(source: &str) -> ast::Ast {
    let (ast, errors) = generate_ast_with_errors(source, true);
    let error_ids = errors.iter().map(|x| x.id).collect::<Vec<_>>();
    assert_eq!(error_ids, &[]);
    return ast;
}

pub fn generate_nodeid_tree(ast: &ast::Ast) -> NodeIdTree {
    // Rust cannot do recursive lambdas, booh
    fn record_tree_recursively(ast: &ast::Ast, noderef: &ast::NodeRef) -> NodeIdTree {
        let mut this = NodeIdTree {
            id: ast.get_node(noderef).id(),
            children: Vec::new(),
        };
        ast::visit_children(ast.get_node(noderef), |noderef| {
            this.children.push(record_tree_recursively(ast, noderef));
            return true;
        });
        return this;
    }

    assert!(ast.get_root().is_some());
    let rootref = ast.get_root().unwrap();
    return record_tree_recursively(&ast, &rootref);
}

pub fn verify_ast(source: &str, expected: &NodeIdTree) -> ast::Ast {
    let ast = generate_ast(source);
    assert_eq!(generate_nodeid_tree(&ast), *expected);
    return ast;
}

#[derive(Clone, PartialEq)]
pub struct NodeIdTree {
    pub id: ast::NodeId,
    pub children: Vec<NodeIdTree>,
}

pub fn tree(id: ast::NodeId, children: &[NodeIdTree]) -> NodeIdTree {
    NodeIdTree {
        id,
        children: children.to_vec(),
    }
}

pub fn leaf(id: ast::NodeId) -> NodeIdTree {
    NodeIdTree {
        id,
        children: Vec::new(),
    }
}

impl<'a> fmt::Debug for NodeIdTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn recurse(node: &NodeIdTree, f: &mut fmt::Formatter<'_>, indent: u32) -> fmt::Result {
            f.write_fmt(format_args!(
                "\n{}|{:?}",
                " ".repeat(indent as usize),
                node.id,
            ))?;
            for child in &node.children {
                recurse(child, f, indent + 1)?;
            }

            return Ok(());
        }

        return recurse(self, f, 0);
    }
}

pub fn module_fragment_wrapper_tree(body: &[NodeIdTree]) -> NodeIdTree {
    use crate::parser::ast::NodeId::*;
    return tree(ModuleFragment, &[tree(StatementBody, body)]);
}
