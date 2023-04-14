use crate::ast::as_node;

use super::*;

impl<'a> Grapher<'a> {
    pub fn parse_expression(
        &mut self,
        astkey: ast::AstKey,
        node: &ast::NodeRef,
    ) -> asg::ExpressionKey {
        match self.context.get_ast(astkey).get_node(node) {
            ast::Node::StructLiteral(n) => self.parse_structliteral(astkey, n),
            ast::Node::StringLiteral(n) => self.parse_stringliteral(astkey, n),
            ast::Node::FunctionLiteral(n) => self.parse_functionliteral(astkey, n),
            ast::Node::BuiltInObjectReference(n) => self.parse_builtinobjectreference(astkey, n),
            ast::Node::SymbolReference(n) => self.parse_symbolreference(astkey, n),
            ast::Node::IfExpression(n) => self.parse_ifexpression(astkey, n),
            ast::Node::CallOperation(n) => self.parse_calloperation(astkey, n),
            ast::Node::BinaryOperation(n) => self.parse_binaryoperation(astkey, n),
            ast::Node::SubScript(n) => self.parse_subscript(astkey, n),
            n => {
                panic!("{:?} is not a valid expression!", n);
            }
        }
    }

    pub fn parse_structliteral(
        &mut self,
        astkey: ast::AstKey,
        ast_lit: &ast::nodes::StructLiteral,
    ) -> asg::ExpressionKey {
        let mut fields = Vec::new();

        let ast = self.context.get_ast(astkey);
        for f in &ast_lit.fields {
            let sf = ast::as_node!(ast, StructField, &f);
            fields.push(asg::misc::StructField {
                name: ast.get_symbol(&sf.symbol).unwrap().clone(),
                typeexpr: self.parse_expression(astkey, &sf.typeexpr),
            });
        }

        let literal = asg::expressions::literals::StructLiteral { fields };

        self.state
            .asg
            .store
            .expressions
            .add(asg::Expression::Literal(
                asg::expressions::Literal::StructLiteral(literal),
            ))
    }

    pub fn parse_stringliteral(
        &mut self,
        _astkey: ast::AstKey,
        ast_lit: &ast::nodes::StringLiteral,
    ) -> asg::ExpressionKey {
        let literal = asg::expressions::literals::StringLiteral {
            string: ast_lit.text.clone(),
        };
        self.state
            .asg
            .store
            .expressions
            .add(asg::Expression::Literal(
                asg::expressions::Literal::StringLiteral(literal),
            ))
    }

    pub fn parse_functionliteral(
        &mut self,
        astkey: ast::AstKey,
        ast_lit: &ast::nodes::FunctionLiteral,
    ) -> asg::ExpressionKey {
        fn create_function_name(state: &State) -> String {
            if state.current_symdecl_name.is_empty() {
                format!("{}", state.get_current_module().name)
            } else {
                format!(
                    "{}.{}",
                    state.get_current_module().name,
                    state.current_symdecl_name
                )
            }
        }

        let ast = self.context.get_ast(astkey);

        // TODO: This is a bit wonky, we want to name functions, but we don't really
        //  know what symbol this will be bound to, try to figure something out
        let name = create_function_name(&self.state);

        let mut function = asg::Function::new(name);

        // Populate in-params
        for inparam in &ast_lit.inputparams {
            let inparam = as_node!(ast, InputParameter, inparam);

            let typeexpr = self.parse_expression(astkey, &inparam.typeexpr);

            function.inparams.push(asg::FunctionParameter {
                name: ast.get_symbol(&inparam.symbol).unwrap().clone(),
                typeexpr: typeexpr,
            });
        }

        let functionkey = self.state.asg.store.functions.add(function);

        // Parse body
        let old_func = self.state.current_function;
        self.state.current_function = Some(functionkey);
        self.parse_statement_body(astkey, as_node!(ast, StatementBody, &ast_lit.body));
        self.state.current_function = old_func;

        // Create literal expression
        let literal = asg::expressions::literals::FunctionLiteral { functionkey };
        self.state
            .asg
            .store
            .expressions
            .add(asg::Expression::Literal(
                asg::expressions::Literal::FunctionLiteral(literal),
            ))
    }

    pub fn parse_builtinobjectreference(
        &mut self,
        _astkey: ast::AstKey,
        ast_obj: &ast::nodes::BuiltInObjectReference,
    ) -> asg::ExpressionKey {
        let expr = match &ast_obj.object {
            ast::BuiltInObject::Function(n) => {
                asg::Expression::BuiltInFunction(asg::expressions::BuiltInFunction { function: *n })
            }
            ast::BuiltInObject::PrimitiveType(n) => {
                asg::Expression::PrimitiveType(asg::expressions::PrimitiveType { ptype: *n })
            }
        };
        self.state.asg.store.expressions.add(expr)
    }

    pub fn parse_symbolreference(
        &mut self,
        astkey: ast::AstKey,
        ast_symref: &ast::nodes::SymbolReference,
    ) -> asg::ExpressionKey {
        let ast = self.context.get_ast(astkey);
        let symbol = ast.get_symbol(&ast_symref.symbol).unwrap().clone();
        let scope = self.state.get_current_symbolscope();
        let symbolref = scope
            .references
            .add(asg::SymbolReference::UnresolvedReference(
                asg::UnresolvedSymbolReference { symbol },
            ));
        let expr =
            asg::Expression::SymbolReference(asg::expressions::SymbolReference { symbolref });
        self.state.asg.store.expressions.add(expr)
    }

    pub fn parse_ifexpression(
        &mut self,
        astkey: ast::AstKey,
        ast_ifexpr: &ast::nodes::IfExpression,
    ) -> asg::ExpressionKey {
        let elsebranch = if let Some(eb) = ast_ifexpr.elsebranch {
            Some(self.parse_expression(astkey, &eb))
        } else {
            None
        };

        let mut branches = Vec::new();

        for branch in &ast_ifexpr.branches {
            let condexpr = self.parse_expression(astkey, &branch.0);
            let thenexpr = self.parse_expression(astkey, &branch.1);
            branches.push((condexpr, thenexpr));
        }

        let ifexpr = asg::expressions::If {
            branches,
            elsebranch,
        };

        self.state
            .asg
            .store
            .expressions
            .add(asg::Expression::If(ifexpr))
    }

    pub fn parse_calloperation(
        &mut self,
        astkey: ast::AstKey,
        ast_callop: &ast::nodes::CallOperation,
    ) -> asg::ExpressionKey {
        let callable = self.parse_expression(astkey, &ast_callop.expr);

        let mut args = Vec::new();

        let ast = self.context.get_ast(astkey);
        let ast_arglist = as_node!(ast, ArgumentList, &ast_callop.arglist);

        for arg in &ast_arglist.args {
            let expr = self.parse_expression(astkey, &arg);
            args.push(expr);
        }

        let callexpr = asg::expressions::Call { callable, args };

        self.state
            .asg
            .store
            .expressions
            .add(asg::Expression::Call(callexpr))
    }

    pub fn parse_binaryoperation(
        &mut self,
        astkey: ast::AstKey,
        ast_binop: &ast::nodes::BinaryOperation,
    ) -> asg::ExpressionKey {
        let lhs = self.parse_expression(astkey, &ast_binop.lhs);
        let rhs = self.parse_expression(astkey, &ast_binop.rhs);

        let op = ast_binop.optype;

        let binopexpr = asg::expressions::BinOp { op, lhs, rhs };

        self.state
            .asg
            .store
            .expressions
            .add(asg::Expression::BinOp(binopexpr))
    }

    pub fn parse_subscript(
        &mut self,
        astkey: ast::AstKey,
        ast_subscript: &ast::nodes::SubScript,
    ) -> asg::ExpressionKey {
        let expr = self.parse_expression(astkey, &ast_subscript.expr);

        let ast = self.context.get_ast(astkey);
        let symbol = ast.get_symbol(&ast_subscript.field).unwrap().clone();

        let subscriptexpr = asg::expressions::Subscript { expr, symbol };

        self.state
            .asg
            .store
            .expressions
            .add(asg::Expression::Subscript(subscriptexpr))
    }
}
