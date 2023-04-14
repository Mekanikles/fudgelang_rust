use crate::asg::Statement;

use super::*;

impl<'a> Grapher<'a> {
    pub fn parse_statement(
        &mut self,
        astkey: ast::AstKey,
        node: &ast::NodeRef,
    ) -> Option<Statement> {
        match self.context.get_ast(astkey).get_node(node) {
            ast::Node::ModuleSelfDeclaration(_) => {
                /* TODO: This should be pruned before any intepretation step */
                todo!()
            }
            ast::Node::Module(n) => {
                self.parse_module(astkey, n);
                None
            }
            ast::Node::StatementBody(_n) => todo!(), // TODO: Can this happen?
            ast::Node::SymbolDeclaration(n) => self.parse_symboldeclaration(astkey, n),
            ast::Node::IfStatement(n) => Some(self.parse_ifstatement(astkey, n)),
            ast::Node::ReturnStatement(n) => Some(self.parse_returnstatement(astkey, n)),
            ast::Node::AssignStatement(n) => Some(self.parse_assignstatement(astkey, n)),
            n => {
                panic!("{:?} is not a valid statement", n);
            }
        }
    }

    pub fn parse_symboldeclaration(
        &mut self,
        astkey: ast::AstKey,
        ast_symdecl: &ast::nodes::SymbolDeclaration,
    ) -> Option<Statement> {
        let ast = self.context.get_ast(astkey);

        let symbol_name: String = ast.get_symbol(&ast_symdecl.symbol).unwrap().into();

        // Blech, storing this a string copy
        let old_symdecl_name =
            std::mem::replace(&mut self.state.current_symdecl_name, symbol_name.clone());

        let type_expr = ast_symdecl
            .typeexpr
            .map(|e| self.parse_expression(astkey, &e));

        let init_expr = ast_symdecl
            .initexpr
            .map(|e| self.parse_expression(astkey, &e));

        self.state.current_symdecl_name = old_symdecl_name;

        let symbol_decl = asg::SymbolDeclaration::new(symbol_name, type_expr, init_expr);

        self.state
            .get_current_symbolscope()
            .declarations
            .add(symbol_decl);

        None
    }

    pub fn parse_ifstatement(
        &mut self,
        astkey: ast::AstKey,
        ast_if: &ast::nodes::IfStatement,
    ) -> Statement {
        let ast = self.context.get_ast(astkey);
        let branches = ast_if
            .branches
            .iter()
            .map(|x| {
                (
                    self.parse_expression(astkey, &x.0),
                    self.parse_statement_body(astkey, ast::as_node!(ast, StatementBody, &x.1)),
                )
            })
            .collect();

        let elsebranch = ast_if
            .elsebranch
            .as_ref()
            .map(|x| self.parse_statement_body(astkey, ast::as_node!(ast, StatementBody, &x)));

        let ifstmt = asg::statements::If {
            branches,
            elsebranch,
        };

        asg::Statement::If(ifstmt)
    }

    pub fn parse_returnstatement(
        &mut self,
        astkey: ast::AstKey,
        ast_return: &ast::nodes::ReturnStatement,
    ) -> Statement {
        let expr = ast_return.expr.map(|e| self.parse_expression(astkey, &e));

        let returnstmt = asg::statements::Return { expr };

        asg::Statement::Return(returnstmt)
    }

    pub fn parse_assignstatement(
        &mut self,
        astkey: ast::AstKey,
        ast_assign: &ast::nodes::AssignStatement,
    ) -> Statement {
        let lhs = self.parse_expression(astkey, &ast_assign.lhs);
        let rhs = self.parse_expression(astkey, &ast_assign.rhs);

        let assignstmt = asg::statements::Assign { lhs, rhs };

        asg::Statement::Assign(assignstmt)
    }
}
