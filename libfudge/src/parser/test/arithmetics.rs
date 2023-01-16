use super::utils::*;

use crate::parser::ast;
use crate::parser::ast::NodeId::*;

use phf::phf_map;

// Map with all addressable binary operators
static BINOPS: phf::Map<&'static str, ast::BinaryOperationType> = phf_map! {
    "+" => ast::BinaryOperationType::Add,
    "-" => ast::BinaryOperationType::Sub,
    "*" => ast::BinaryOperationType::Mul,
    "/" => ast::BinaryOperationType::Div,
    "==" => ast::BinaryOperationType::Equals,
    ">" => ast::BinaryOperationType::GreaterThan,
    ">=" => ast::BinaryOperationType::GreaterThanOrEq,
    "<" => ast::BinaryOperationType::LessThan,
    "<=" => ast::BinaryOperationType::LessThanOrEq,
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
fn test_parenthesis_precedence() {
    verify_ast(
        "a + (b + c)",
        &module_fragment_wrapper_tree(&[tree(
            BinaryOperation,
            &[
                leaf(SymbolReference),
                tree(
                    BinaryOperation,
                    &[leaf(SymbolReference), leaf(SymbolReference)],
                ),
            ],
        )]),
    );
}

#[test]
fn test_all_binary_operations_order_ltr() {
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
fn test_binary_operation_order_same_precedence1() {
    let ast1 = generate_ast("a + b - c + d");
    let ast2 = generate_ast("((a + b) - c) + d");

    assert_eq!(generate_nodeid_tree(&ast1), generate_nodeid_tree(&ast2));
}

#[test]
fn test_binary_operation_order_same_precedence2() {
    let ast1 = generate_ast("a * b / c * d");
    let ast2 = generate_ast("((a * b) / c) * d");

    assert_eq!(generate_nodeid_tree(&ast1), generate_nodeid_tree(&ast2));
}

#[test]
fn test_binary_operation_order_precedence_mixed1() {
    let ast1 = generate_ast("a + b * c");
    let ast2 = generate_ast("a + (b * c)");

    assert_eq!(generate_nodeid_tree(&ast1), generate_nodeid_tree(&ast2));
}

#[test]
fn test_binary_operation_order_precedence_mixed2() {
    let ast1 = generate_ast("a / b - c");
    let ast2 = generate_ast("(a / b) - c");

    assert_eq!(generate_nodeid_tree(&ast1), generate_nodeid_tree(&ast2));
}

#[test]
fn test_binary_operation_order_precedence_mixed3() {
    let ast1 = generate_ast("a + b - c * d / e");
    let ast2 = generate_ast("(a + b) - ((c * d) / e)");

    assert_eq!(generate_nodeid_tree(&ast1), generate_nodeid_tree(&ast2));
}

#[test]
fn test_binary_operation_order_precedence_mixed4() {
    let ast1 = generate_ast("a - b * c + d / e");
    let ast2 = generate_ast("a - (b * c) + (d / e)");

    assert_eq!(generate_nodeid_tree(&ast1), generate_nodeid_tree(&ast2));
}

#[test]
fn test_binary_operation_order_comparison() {
    let ast1 = generate_ast("a < b + c");
    let ast2 = generate_ast("a < (b + c)");

    assert_eq!(generate_nodeid_tree(&ast1), generate_nodeid_tree(&ast2));
}
