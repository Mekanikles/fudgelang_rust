use super::utils::*;

use crate::parser::ast;
use crate::parser::ast::NodeId::*;

use phf::phf_map;

// Map with all declaration types
static DECLTYPES: phf::Map<&'static str, ast::SymbolDeclarationType> = phf_map! {
    "def" => ast::SymbolDeclarationType::Def,
    "var" => ast::SymbolDeclarationType::Var,
};

#[test]
fn test_all_simple_declarations() {
    fn test_simple_declaration(declstr: &str, decltype: ast::SymbolDeclarationType) {
        let source = format!("{} a = 0", declstr);
        let expected =
            module_fragment_wrapper_tree(&[tree(SymbolDeclaration, &[leaf(IntegerLiteral)])]);
        let ast = verify_ast(source.as_str(), &expected);

        if let Some(noderef) = ast.find_first_node(SymbolDeclaration) {
            if let ast::Node::SymbolDeclaration(n) = ast.get_node(&noderef) {
                assert_eq!(decltype, n.decltype);
            }
        }
    }

    for decl in DECLTYPES.keys() {
        test_simple_declaration(decl, DECLTYPES[decl]);
    }
}
