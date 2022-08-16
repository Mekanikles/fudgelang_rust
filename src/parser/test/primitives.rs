use crate::typesystem::*;
use super::utils::*;

use crate::parser::ast;
use crate::parser::ast::NodeId::*;

fn wrap_in_simple_declaration(typename: &str) -> String {
    return format!("def _ = {}", typename);
}

fn simple_simple_declaration_tree(body: &[NodeIdTree]) -> NodeIdTree {
    return module_fragment_wrapper_tree(&[
        tree(SymbolDeclaration, body)
        ]);
}

#[test]
fn test_all_builtin_primitive_declarations() {
    fn test_builtin_primitive_declaration(name: &str, ptype: &PrimitiveType) {
        let p = format!("#primitives.{}", name);
        let source = wrap_in_simple_declaration(&p);
        let expected = simple_simple_declaration_tree(&[leaf(BuiltInObjectReference)]);
        let ast = verify_ast(source.as_str(), &expected);

        if let Some(noderef) = ast.find_node(BuiltInObjectReference) {
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