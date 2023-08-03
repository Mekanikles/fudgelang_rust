use super::*;

use crate::shared::BinaryOperationType;

use snailquote;

pub enum OpPrecedence {
    MulDiv,
    AddSub,
    Comparisons,
}

pub const fn binop_precedence(optype: &BinaryOperationType) -> u32 {
    use BinaryOperationType::*;
    match optype {
        Add => OpPrecedence::AddSub as u32,
        Sub => OpPrecedence::AddSub as u32,
        Mul => OpPrecedence::MulDiv as u32,
        Div => OpPrecedence::MulDiv as u32,
        Equals => OpPrecedence::Comparisons as u32,
        LessThan => OpPrecedence::Comparisons as u32,
        LessThanOrEq => OpPrecedence::Comparisons as u32,
        GreaterThan => OpPrecedence::Comparisons as u32,
        GreaterThanOrEq => OpPrecedence::Comparisons as u32,
    }
}

pub const fn has_higher_precedence(a: &BinaryOperationType, b: &BinaryOperationType) -> bool {
    binop_precedence(a) < binop_precedence(b)
}

impl<'a> Parser<'a> {
    pub fn parse_expression(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        self.push_block();
        let result = self.parse_composite_expression();
        self.pop_block();
        return result;
    }

    // Parses expressions composed with operators
    fn parse_composite_expression(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        // For more info, see Shunting Yard Algorithm

        let mut exprstack: Vec<ast::NodeRef> = Vec::new();
        let mut binopstack: Vec<BinaryOperationType> = Vec::new();

        if let Some(expr) = self.parse_left_recursive_expression()? {
            exprstack.push(expr);
        } else {
            return Ok(None);
        }

        // Consume the last binary operator and two expressions and push the result on
        //  the expression stack
        fn consume_last_op(
            ast: &mut ast::Ast,
            exprstack: &mut Vec<ast::NodeRef>,
            binopstack: &mut Vec<BinaryOperationType>,
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

            if let Some(expr) = self.parse_left_recursive_expression()? {
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

    // Parses expressions leading with expressions
    fn parse_left_recursive_expression(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if let Some(expr) = self.parse_primary_expression()? {
            if let Some(tail) = self.parse_tail_expression(&expr)? {
                return Ok(Some(tail));
            } else {
                return Ok(Some(expr));
            }
        }

        return Ok(None);
    }

    fn parse_tail_expression(
        &mut self,
        head: &ast::NodeRef,
    ) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        // TODO: Parsing the tail after the head has been parsed like this will give a suboptimal
        //  ast node order, with the tail node being placed after the head.
        if let Some(expr) = self.parse_tail_expression_inner(head)? {
            if let Some(tail) = self.parse_tail_expression(&expr)? {
                return Ok(Some(tail));
            }

            return Ok(Some(expr));
        }

        return Ok(None);
    }

    fn parse_tail_expression_inner(
        &mut self,
        head: &ast::NodeRef,
    ) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        // Calls
        if self.accept(TokenType::OpeningParenthesis) {
            let node = self.ast.reserve_node();
            let arglist = self.parse_argumentlist()?;

            self.expect(TokenType::ClosingParenthesis)?;

            return Ok(Some(
                self.ast.replace_node(
                    node,
                    ast::nodes::CallOperation {
                        expr: *head,
                        arglist: arglist,
                    }
                    .into(),
                ),
            ));
        }
        // Field subscripts
        else if self.accept(TokenType::Dot) {
            let node = self.ast.reserve_node();
            self.expect(TokenType::Identifier)?;
            let f = self.get_last_token_symbol();

            return Ok(Some(
                self.ast.replace_node(
                    node,
                    ast::nodes::SubScript {
                        expr: *head,
                        field: f,
                    }
                    .into(),
                ),
            ));
        }

        return Ok(None);
    }

    // Parses expressions determined by literal
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

            // Unescape string
            // NOTE: Snailquote requires the string to have double quotes surrounding it
            //  which is why the scanner keeps them around
            let string = snailquote::unescape(text).unwrap();

            return Ok(Some(
                self.ast.add_node(
                    ast::nodes::StringLiteral {
                        text: string,
                    }
                    .into(),
                ),
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
            let s = self.get_last_token_symbol();

            return Ok(Some(
                self.ast
                    .add_node(ast::nodes::SymbolReference { symbol: s }.into()),
            ));
        } else if let Some(n) = self.parse_if_expression()? {
            return Ok(Some(n));
        } else if let Some(n) = self.parse_function_literal_or_type()? {
            return Ok(Some(n));
        } else if let Some(n) = self.parse_struct_literal()? {
            return Ok(Some(n));
        } else if let Some(n) = self.parse_builtin_expression()? {
            return Ok(Some(n));
        }
        return Ok(None);
    }

    fn parse_if_expression(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if self.accept(TokenType::If) {
            let node = self.ast.reserve_node();

            // Expr
            let condition = self.expect_expression()?;

            let mut branches: Vec<(ast::NodeRef, ast::NodeRef)> = Vec::new();
            let mut elsebranch: Option<ast::NodeRef> = None;

            // Primary branch
            self.expect_with_layout(TokenType::FatArrow, TokenLayoutType::BlockKeyword)?;
            {
                branches.push((condition, self.expect_expression()?));
            }

            // Else-if branches
            while self.accept_with_layout(TokenType::ElseIf, TokenLayoutType::BlockLinker) {
                let condition = self.expect_expression()?;

                self.expect_with_layout(TokenType::FatArrow, TokenLayoutType::BlockKeyword)?;

                branches.push((condition, self.expect_expression()?));
            }

            // Final else
            if self.accept_with_layout(TokenType::Else, TokenLayoutType::BlockLinker) {
                // Final else
                elsebranch = Some(self.expect_expression()?);
            }

            return Ok(Some(
                self.ast.replace_node(
                    node,
                    ast::nodes::IfExpression {
                        branches,
                        elsebranch,
                    }
                    .into(),
                ),
            ));
        }

        return Ok(None);
    }
}
