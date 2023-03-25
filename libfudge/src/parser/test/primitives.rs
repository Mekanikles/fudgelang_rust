use super::utils::*;
use crate::typesystem::*;

use crate::ast;
use crate::ast::NodeId::*;

fn wrap_in_simple_declaration(typename: &str) -> String {
    return format!("def _ = {}", typename);
}

fn simple_declaration_wrapper_tree(body: &[NodeIdTree]) -> NodeIdTree {
    return entrypoint_wrapper_tree(&[tree(SymbolDeclaration, body)]);
}

#[test]
fn test_all_builtin_primitive_declarations() {
    fn test_builtin_primitive_declaration(name: &str, ptype: &PrimitiveType) {
        let p = format!("#primitives.{}", name);
        let source = wrap_in_simple_declaration(&p);
        let expected = simple_declaration_wrapper_tree(&[leaf(BuiltInObjectReference)]);
        let ast = verify_ast(source.as_str(), &expected);

        if let Some(noderef) = ast.find_first_node(BuiltInObjectReference) {
            if let ast::Node::BuiltInObjectReference(n) = ast.get_node(&noderef) {
                if let ast::BuiltInObject::PrimitiveType(pt) = n.object {
                    assert_eq!(&pt, ptype);
                }
            }
        }
    }

    for key in PRIMITIVES.keys() {
        test_builtin_primitive_declaration(key, &PRIMITIVES[key]);
    }
}

#[test]
fn test_boolean_literals() {
    verify_ast(
        "true\nfalse",
        &entrypoint_wrapper_tree(&[leaf(BooleanLiteral), leaf(BooleanLiteral)]),
    );
}
