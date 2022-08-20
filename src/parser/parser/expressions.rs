use super::*;

pub const fn binop_precedence(optype: &ast::BinaryOperationType) -> u32 {
    use ast::BinaryOperationType::*;
    match optype {
        Add => 1,
        Sub => 1,
        Mul => 2,
        Div => 2,
    }
}

pub const fn has_higher_precedence(
    a: &ast::BinaryOperationType,
    b: &ast::BinaryOperationType,
) -> bool {
    binop_precedence(a) > binop_precedence(b)
}

impl<'a, T: TokenStream> Parser<'a, T> {
    pub fn parse_expression(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        // For more info, see Shunting Yard Algorithm

        let mut exprstack: Vec<ast::NodeRef> = Vec::new();
        let mut binopstack: Vec<ast::BinaryOperationType> = Vec::new();

        if let Some(expr) = self.parse_primary_expression()? {
            exprstack.push(expr);
        } else {
            return Ok(None);
        }

        // Consume the last binary operator and two expressions and push the result on
        //  the expression stack
        fn consume_last_op(
            ast: &mut ast::Ast,
            exprstack: &mut Vec<ast::NodeRef>,
            binopstack: &mut Vec<ast::BinaryOperationType>,
        ) {
            assert!(exprstack.len() > 1);
            let rhs = exprstack.pop().unwrap();
            let lhs = exprstack.pop().unwrap();
            exprstack.push(
                ast.add_node(
                    ast::nodes::BinaryOperation {
                        optype: binopstack.pop().unwrap(),
                        lhs,
                        rhs,
                    }
                    .into(),
                ),
            );
        }

        // Parse entire expression, separating expressions and operators
        while let Some(optype) = self.accept_binaryoperator() {
            // Bind expressions as long as the new operator has lower or same priority
            // This ensures left-associativity since all available expressions are bound as soon as possible
            while !binopstack.is_empty()
                && !has_higher_precedence(&optype, binopstack.last().unwrap())
            {
                consume_last_op(&mut self.ast, &mut exprstack, &mut binopstack);
            }

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

        // At this point, we know that all remaining operations are in strict ascending priority order
        // Bind them right-to-left
        while !binopstack.is_empty() {
            consume_last_op(&mut self.ast, &mut exprstack, &mut binopstack);
        }

        return Ok(exprstack.pop());
    }

    fn parse_primary_expression(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if self.accept(TokenType::True) {
            return Ok(Some(
                self.ast
                    .add_node(ast::nodes::BooleanLiteral { value: true }.into()),
            ));
        } else if self.accept(TokenType::False) {
            return Ok(Some(
                self.ast
                    .add_node(ast::nodes::BooleanLiteral { value: false }.into()),
            ));
        } else if self.accept(TokenType::StringLiteral) {
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
