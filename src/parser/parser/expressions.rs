use super::*;

impl<'a, T: TokenStream> Parser<'a, T> {
    pub fn parse_expression(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if let Some(lhs) = self.parse_inner_expression()? {
            // TODO: Replace with shunting yard for correct operator precedence
            if let Some(optype) = self.accept_binaryoperator() {
                let node = self.ast.reserve_node();

                if let Some(rhs) = self.parse_expression()? {
                    return Ok(Some(self.ast.replace_node(
                        node,
                        ast::nodes::BinaryOperation { optype, lhs, rhs }.into(),
                    )));
                } else {
                    return Err(self.log_error(error::Error::at_span(
                        errors::ExpectedExpression,
                        // TODO: Should be current token, would work with EOS tokens
                        self.last_token.as_ref().unwrap().source_span,
                        "Expected right hand side expression".into(),
                    ))?);
                }
            } else {
                return Ok(Some(lhs));
            }
        }

        return Ok(None);
    }

    fn parse_inner_expression(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
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
        } else if self.accept(TokenType::Hash) {
            let mut symbolstrings = Vec::new();

            let startpos = self.last_token.as_ref().unwrap().source_span.pos;

            // TODO: What to do with whitespace between # and identifier?
            self.expect(TokenType::Identifier)?;
            symbolstrings.push(self.get_last_token_text());

            // Eat dot-notated symbol expression
            while self.accept(TokenType::Dot) {
                self.expect(TokenType::Identifier)?;
                symbolstrings.push(self.get_last_token_text());
            }

            let endpos = self.last_token.as_ref().unwrap().source_span.pos
                + self.last_token.as_ref().unwrap().source_span.len as u64;

            symbolstrings.reverse();

            // TODO: simplify
            if symbolstrings
                .last()
                .filter(|s| *s == "primitives")
                .is_some()
            {
                symbolstrings.pop();

                if let Some(s) = symbolstrings.last() {
                    let object = ast::BuiltInObject::PrimitiveType(PRIMITIVES[s]);
                    symbolstrings.pop();
                    return Ok(Some(
                        self.ast
                            .add_node(ast::nodes::BuiltInObjectReference { object }.into()),
                    ));
                } else {
                    // TODO: Error
                }
            } else if symbolstrings.last().filter(|s| *s == "output").is_some() {
                symbolstrings.pop();
                if symbolstrings
                    .last()
                    .filter(|s| *s == "print_format")
                    .is_some()
                {
                    symbolstrings.pop();
                    let node = self.ast.reserve_node();

                    let builtinfunc = self.ast.add_node(
                        ast::nodes::BuiltInObjectReference {
                            object: ast::BuiltInObject::Function(BuiltInFunction::PrintFormat),
                        }
                        .into(),
                    );

                    self.expect(TokenType::OpeningParenthesis)?;

                    let arglist = self.parse_argumentlist()?;

                    self.expect(TokenType::ClosingParenthesis)?;

                    return Ok(Some(
                        self.ast.replace_node(
                            node,
                            ast::nodes::CallOperation {
                                expr: builtinfunc,
                                arglist: arglist,
                            }
                            .into(),
                        ),
                    ));
                }
            }

            return Err(self.log_error(error::Error::at_span(
                errors::UnknownCompilerDirective,
                SourceSpan {
                    pos: startpos,
                    len: (endpos - startpos) as usize,
                },
                "Unknown compiler directive".into(),
            ))?);
        }
        return Ok(None);
    }
}
