use crate::asg::Expression;
use crate::ast::as_node;

use super::*;

use super::asg::scope::ExpressionKey;

impl<'a> Grapher<'a> {
    fn add_expression(&mut self, object: asg::ExpressionObject) -> ExpressionKey {
        let scope = self.state.get_current_scope();

        scope.expressions.add(Expression::new(object, 666))
    }

    pub fn parse_expression(&mut self, astkey: ast::AstKey, node: &ast::NodeRef) -> ExpressionKey {
        match self.context.get_ast(astkey).get_node(node) {
            ast::Node::StringLiteral(n) => self.parse_stringliteral(astkey, n),
            ast::Node::BooleanLiteral(n) => self.parse_booleanliteral(astkey, n),
            ast::Node::IntegerLiteral(n) => self.parse_integerliteral(astkey, n),
            ast::Node::StructLiteral(n) => self.parse_structliteral(astkey, n),
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

    pub fn parse_stringliteral(
        &mut self,
        _astkey: ast::AstKey,
        ast_lit: &ast::nodes::StringLiteral,
    ) -> ExpressionKey {
        let literal = asg::expressions::literals::StringLiteral {
            string: ast_lit.text.clone(),
        };
        self.add_expression(asg::ExpressionObject::Literal(
            asg::expressions::Literal::StringLiteral(literal),
        ))
    }

    pub fn parse_booleanliteral(
        &mut self,
        _astkey: ast::AstKey,
        ast_lit: &ast::nodes::BooleanLiteral,
    ) -> ExpressionKey {
        let literal = asg::expressions::literals::BoolLiteral {
            value: ast_lit.value,
        };
        self.add_expression(asg::ExpressionObject::Literal(
            asg::expressions::Literal::BoolLiteral(literal),
        ))
    }

    pub fn parse_integerliteral(
        &mut self,
        _astkey: ast::AstKey,
        ast_lit: &ast::nodes::IntegerLiteral,
    ) -> ExpressionKey {
        let literal = asg::expressions::literals::IntegerLiteral {
            data: ast_lit.value,
            signed: ast_lit.signed,
        };
        self.add_expression(asg::ExpressionObject::Literal(
            asg::expressions::Literal::IntegerLiteral(literal),
        ))
    }

    pub fn parse_structliteral(
        &mut self,
        astkey: ast::AstKey,
        ast_lit: &ast::nodes::StructLiteral,
    ) -> ExpressionKey {
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

        self.add_expression(asg::ExpressionObject::Literal(
            asg::expressions::Literal::StructLiteral(literal),
        ))
    }

    pub fn parse_functionliteral(
        &mut self,
        astkey: ast::AstKey,
        ast_lit: &ast::nodes::FunctionLiteral,
    ) -> ExpressionKey {
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

        let parentscope = asg::ScopeRef::new(self.state.current_module, self.state.current_scope);
        let mut function =
            asg::Function::new(name, &mut self.state.get_current_module_mut(), parentscope);

        // Populate in-params
        for inparam in &ast_lit.inputparams {
            let inparam = as_node!(ast, InputParameter, inparam);

            let typeexpr = self.parse_expression(astkey, &inparam.typeexpr);

            let symdecl = self
                .state
                .edit_scope(&function.scope)
                .symboltable
                .declarations
                .add(asg::symboltable::SymbolDeclaration {
                    symbol: ast.get_symbol(&inparam.symbol).unwrap().clone(),
                    typeexpr: Some(typeexpr),
                });

            let symref = asg::ResolvedSymbolReference {
                scope: asg::ScopeRef {
                    module: self.state.current_module,
                    scope: function.scope,
                },
                symbol: symdecl,
            };

            function.inparams.push(asg::FunctionParameter { symref });
        }

        // Parse body
        let statmentbody = self.parse_statement_body(
            astkey,
            as_node!(ast, StatementBody, &ast_lit.body),
            function.scope,
        );

        self.state.edit_scope(&function.scope).statementbody = statmentbody;

        let functionkey = self
            .state
            .get_current_module_mut()
            .functionstore
            .add(function);

        // Create literal expression
        let literal = asg::expressions::literals::FunctionLiteral { functionkey };
        self.add_expression(asg::ExpressionObject::Literal(
            asg::expressions::Literal::FunctionLiteral(literal),
        ))
    }

    pub fn parse_builtinobjectreference(
        &mut self,
        _astkey: ast::AstKey,
        ast_obj: &ast::nodes::BuiltInObjectReference,
    ) -> ExpressionKey {
        let expr = match &ast_obj.object {
            ast::BuiltInObject::Function(n) => {
                asg::ExpressionObject::BuiltInFunction(asg::expressions::BuiltInFunction {
                    function: *n,
                })
            }
            ast::BuiltInObject::PrimitiveType(n) => {
                asg::ExpressionObject::PrimitiveType(asg::expressions::PrimitiveType { ptype: *n })
            }
        };
        self.add_expression(expr)
    }

    pub fn parse_symbolreference(
        &mut self,
        astkey: ast::AstKey,
        ast_symref: &ast::nodes::SymbolReference,
    ) -> ExpressionKey {
        let ast = self.context.get_ast(astkey);
        let symbol = ast.get_symbol(&ast_symref.symbol).unwrap().clone();
        let scopekey = self.state.current_scope;

        let scope = self.state.edit_scope(&scopekey);
        let symbolref = scope.symboltable.references.add(
            asg::symboltable::SymbolReference::UnresolvedReference(
                asg::UnresolvedSymbolReference { symbol },
            ),
        );
        let expr =
            asg::ExpressionObject::SymbolReference(asg::expressions::SymbolReference { symbolref });
        self.add_expression(expr)
    }

    pub fn parse_ifexpression(
        &mut self,
        astkey: ast::AstKey,
        ast_ifexpr: &ast::nodes::IfExpression,
    ) -> ExpressionKey {
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

        self.add_expression(asg::ExpressionObject::If(ifexpr))
    }

    pub fn parse_calloperation(
        &mut self,
        astkey: ast::AstKey,
        ast_callop: &ast::nodes::CallOperation,
    ) -> ExpressionKey {
        let callable = self.parse_expression(astkey, &ast_callop.expr);

        let mut args = Vec::new();

        let ast = self.context.get_ast(astkey);
        let ast_arglist = as_node!(ast, ArgumentList, &ast_callop.arglist);

        for arg in &ast_arglist.args {
            let expr = self.parse_expression(astkey, &arg);
            args.push(expr);
        }

        let callexpr = asg::expressions::Call { callable, args };

        self.add_expression(asg::ExpressionObject::Call(callexpr))
    }

    pub fn parse_binaryoperation(
        &mut self,
        astkey: ast::AstKey,
        ast_binop: &ast::nodes::BinaryOperation,
    ) -> ExpressionKey {
        let lhs = self.parse_expression(astkey, &ast_binop.lhs);
        let rhs = self.parse_expression(astkey, &ast_binop.rhs);

        let op = ast_binop.optype;

        let binopexpr = asg::expressions::BinOp { op, lhs, rhs };

        self.add_expression(asg::ExpressionObject::BinOp(binopexpr))
    }

    pub fn parse_subscript(
        &mut self,
        astkey: ast::AstKey,
        ast_subscript: &ast::nodes::SubScript,
    ) -> ExpressionKey {
        let expr = self.parse_expression(astkey, &ast_subscript.expr);

        let ast = self.context.get_ast(astkey);
        let symbol = ast.get_symbol(&ast_subscript.field).unwrap().clone();

        let subscriptexpr = asg::expressions::Subscript { expr, symbol };

        self.add_expression(asg::ExpressionObject::Subscript(subscriptexpr))
    }
}
