use super::*;

impl<'a> Grapher<'a> {
    pub fn parse_statement(&mut self, astkey: ast::AstKey, node: &ast::NodeRef) {
        return match self.context.get_ast(astkey).get_node(node) {
            ast::Node::ModuleSelfDeclaration(_) => {
                /* TODO: This should be pruned before any intepretation step */
            }
            ast::Node::Module(n) => self.parse_module(astkey, n),
            ast::Node::StatementBody(_n) => todo!(), // TODO: Can this happen?
            ast::Node::SymbolDeclaration(n) => self.parse_symbol_declaration(astkey, n),
            ast::Node::IfStatement(n) => todo!(),
            ast::Node::ReturnStatement(n) => todo!(),
            ast::Node::AssignStatement(n) => todo!(),
            n => {
                panic!("{:?} is not a valid statement", n);
            }
        };
    }

    pub fn parse_symbol_declaration(
        &mut self,
        astkey: ast::AstKey,
        ast_symdecl: &ast::nodes::SymbolDeclaration,
    ) {
        let ast = self.context.get_ast(astkey);

        let symbol_name: String = ast.get_symbol(&ast_symdecl.symbol).unwrap().into();

        // Blech
        self.state.current_symdecl_name = symbol_name.clone();

        let type_expr = if let Some(e) = ast_symdecl.typeexpr {
            Some(self.parse_expression(astkey, &e))
        } else {
            None
        };

        let init_expr = if let Some(e) = ast_symdecl.initexpr {
            Some(self.parse_expression(astkey, &e))
        } else {
            None
        };

        // Bluch
        self.state.current_symdecl_name = "".into();

        self.state
            .get_current_symbolscope()
            .declarations
            .add(asg::SymbolDeclaration::new(
                symbol_name,
                type_expr,
                init_expr,
            ));
    }
}
