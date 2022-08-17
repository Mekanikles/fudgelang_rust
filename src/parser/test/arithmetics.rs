use super::utils::*;

use crate::parser::ast;
use crate::parser::ast::NodeId::*;

use phf::phf_map;

// Map with all adressable binary operators
static BINOPS: phf::Map<&'static str, ast::BinaryOperationType> = phf_map! {
    "+" => ast::BinaryOperationType::Add,
    "-" => ast::BinaryOperationType::Sub,
    "*" => ast::BinaryOperationType::Mul,
    "/" => ast::BinaryOperationType::Div,
};

#[test]
fn test_all_simple_binary_operations() {
    fn test_simple_binary_operation(opstr: &str, optype: ast::BinaryOperationType) {
        let source = format!("a {} b", opstr);
        let expected = module_fragment_wrapper_tree(&[tree(
            BinaryOperation,
            &[leaf(SymbolReference), leaf(SymbolReference)],
        )]);
        let ast = verify_ast(source.as_str(), &expected);

        if let Some(noderef) = ast.find_first_node(BinaryOperation) {
            if let ast::Node::BinaryOperation(n) = ast.get_node(&noderef) {
                assert_eq!(optype, n.optype);
            }
        }
    }

    for op in BINOPS.keys() {
        test_simple_binary_operation(op, BINOPS[op]);
    }
}

#[test]
fn test_empty_parenthesis() {
    verify_ast("()", &module_fragment_wrapper_tree(&[]));
}

#[test]
fn test_nested_empty_parenthesis() {
    verify_ast("((()))", &module_fragment_wrapper_tree(&[]));
}

#[test]
fn test_simple_parenthesis() {
    let ast1 = generate_ast("a");
    let ast2 = generate_ast("(a)");

    assert_eq!(generate_nodeid_tree(&ast1), generate_nodeid_tree(&ast2));
}

#[test]
fn test_binary_operation_order_ltr() {
    fn test_ltr_for_binary_op(opstr: &str) {
        let ast1 = generate_ast(&format!("a {} b {} c {} d", opstr, opstr, opstr));
        let ast2 = generate_ast(&format!("((a {} b) {} c) {} d", opstr, opstr, opstr));

        assert_eq!(generate_nodeid_tree(&ast1), generate_nodeid_tree(&ast2));
    }

    for op in BINOPS.keys() {
        test_ltr_for_binary_op(op);
    }
}

#[test]
fn test_operation_order_precedence() {
    let ast1 = generate_ast("a + b - c * d / c");
    let ast2 = generate_ast("(((a + b) - c) * d) / c)");

    assert_eq!(generate_nodeid_tree(&ast1), generate_nodeid_tree(&ast2));
}
