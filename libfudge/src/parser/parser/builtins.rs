use super::*;

impl<'a> Parser<'a> {
    pub fn parse_builtin_expression(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if self.accept(TokenType::Hash) {
            let mut symbolstrings = Vec::new();

            let startpos = self.last_token.as_ref().unwrap().source_span.pos;

            // TODO: What to do with whitespace between # and identifier?
            self.expect(TokenType::Identifier)?;
            symbolstrings.push(self.get_last_token_text().to_string());

            // Eat dot-notated symbol expression
            while self.accept(TokenType::Dot) {
                self.expect(TokenType::Identifier)?;
                symbolstrings.push(self.get_last_token_text().to_string());
            }

            let endpos = self.last_token.as_ref().unwrap().source_span.pos
                + self.last_token.as_ref().unwrap().source_span.len as u64;

            symbolstrings.reverse();

            // TODO: simplify
            if symbolstrings
                .last()
                .filter(|s| **s == "primitives")
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
            } else if symbolstrings.last().filter(|s| **s == "output").is_some() {
                symbolstrings.pop();
                if symbolstrings
                    .last()
                    .filter(|s| **s == "print_format")
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
