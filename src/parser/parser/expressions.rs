use super::*;

impl<'a, T: TokenStream> Parser<'a, T> {
    pub fn parse_expression(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        // Shunting yard algorithm

        let mut exprstack: Vec<ast::NodeRef> = Vec::new();
        let mut binopstack: Vec<ast::BinaryOperationType> = Vec::new();

        if let Some(expr) = self.parse_primary_expression()? {
            exprstack.push(expr);
        } else {
            return Ok(None);
        }

        // Parse entire expression, separating expressions and operators
        while let Some(optype) = self.accept_binaryoperator() {
            binopstack.push(optype);

            if let Some(expr) = self.parse_primary_expression()? {
                exprstack.push(expr);
            } else {
                return Err(self.log_error(error::Error::at_span(
                    errors::ExpectedExpression,
                    // TODO: Should be current token, would work with EOS tokens
                    self.last_token.as_ref().unwrap().source_span,
                    "Expected right hand side expression".into(),
                ))?);
            }
        }

        // Bind any remaining expressions left-to-right, for operator left-associativity
        fn bind_remaining_ops(
            ast: &mut ast::Ast,
            exprstack: &mut Vec<ast::NodeRef>,
            binopstack: &mut Vec<ast::BinaryOperationType>,
        ) -> ast::NodeRef {
            // Important to pop rhs first
            if let Some(optype) = binopstack.pop() {
                let node = ast.reserve_node();
                let rhs = exprstack.pop().unwrap();
                let lhs = bind_remaining_ops(ast, exprstack, binopstack);
                return ast.replace_node(
                    node,
                    ast::nodes::BinaryOperation { optype, lhs, rhs }.into(),
                );
            } else {
                assert!(exprstack.len() == 1);
                return exprstack.pop().unwrap();
            }
        }

        return Ok(Some(bind_remaining_ops(
            &mut self.ast,
            &mut exprstack,
            &mut binopstack,
        )));
    }

    fn parse_primary_expression(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if self.accept(TokenType::StringLiteral) {
            let text = self.get_last_token_text();
            return Ok(Some(
                self.ast
                    .add_node(ast::nodes::StringLiteral { text: text }.into()),
            ));
        } else if self.accept(TokenType::NumericLiteral) {
            let text = self.get_last_token_text();
            // TODO: Support for other numericals
            return Ok(Some(
                self.ast.add_node(
                    ast::nodes::IntegerLiteral {
                        value: text.parse::<u64>().unwrap(),
                        signed: false,
                    }
                    .into(),
                ),
            ));
        } else if self.accept(TokenType::OpeningParenthesis) {
            let expr = self.parse_expression()?;

            self.expect(TokenType::ClosingParenthesis)?;
            return Ok(expr);
        } else if self.accept(TokenType::Identifier) {
            // TODO: Function calls
            let s = self.get_last_token_symbol();

            // Calls
            if self.accept(TokenType::OpeningParenthesis) {
                let node = self.ast.reserve_node();
                let symbol = self
                    .ast
                    .add_node(ast::nodes::SymbolReference { symbol: s }.into());

                let arglist = self.parse_argumentlist()?;

                self.expect(TokenType::ClosingParenthesis)?;

                return Ok(Some(
                    self.ast.replace_node(
                        node,
                        ast::nodes::CallOperation {
                            expr: symbol,
                            arglist: arglist,
                        }
                        .into(),
                    ),
                ));
            } else {
                return Ok(Some(
                    self.ast
                        .add_node(ast::nodes::SymbolReference { symbol: s }.into()),
                ));
            }
        } else if let Some(n) = self.parse_function_literal_or_type()? {
            return Ok(Some(n));
        } else if let Some(n) = self.parse_builtin_expression()? {
            return Ok(Some(n));
        }
        return Ok(None);
    }
}
