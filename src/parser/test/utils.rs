use crate::parser::ast;
use crate::parser::ast::NodeInfo;

use crate::parser::*;
use crate::scanner::*;
use crate::source::*;

use crate::output;

pub fn generate_ast(source: &str) -> ast::Ast {
    let source = MemorySource::from_str(source);

    let mut scanner = ScannerImpl::new(&source);
    let mut parser = parser::Parser::new(&mut scanner);
    parser.parse();

    output::print_errors(&parser.get_tokenstream_errors(), &source);
    output::print_errors(&parser.get_parser_errors(), &source);

    return parser.ast;
}

pub fn verify_ast(source: &str, expected: &NodeIdTree) -> ast::Ast {
    let ast = generate_ast(source);

    assert!(ast.get_root().is_some());
    let rootref = ast.get_root().unwrap();

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

    let tree = record_tree_recursively(&ast, &rootref);

    assert_eq!(tree, *expected);
    return ast;
}

#[derive(Clone, PartialEq, Debug)]
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

pub fn module_fragment_wrapper_tree(body: &[NodeIdTree]) -> NodeIdTree {
    use crate::parser::ast::NodeId::*;
    return tree(ModuleFragment, &[tree(StatementBody, body)]);
}
