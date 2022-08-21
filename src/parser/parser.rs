use super::ast;
use super::ast::Ast;
use super::tokenstream::TokenStream;
use crate::error;
use crate::error::errors;
use crate::scanner::*;
use crate::source::*;

use crate::parser::stringstore::*;
use crate::typesystem::*;

mod builtins;
mod expressions;

use StringRef as SymbolRef;

pub struct Parser<'a, T: TokenStream> {
    tokens: &'a mut T,
    current_token: Option<Token>,
    last_token: Option<Token>,
    temp_tokencount: u32,
    pub ast: Ast,
    errors: error::ErrorManager,
}

impl<'a, T: TokenStream> Parser<'a, T> {
    pub fn new(tokens: &'a mut T) -> Self {
        Parser {
            tokens: tokens,
            current_token: None,
            last_token: None,
            temp_tokencount: 0,
            ast: Ast::new(),
            errors: error::ErrorManager::new(),
        }
    }

    pub fn get_parser_errors(&self) -> &Vec<error::Error> {
        return self.errors.get_errors();
    }

    // Bah, an error manager should really be passed into both scanner and parser
    //  but mutable shared refs are a mess in rust
    pub fn get_tokenstream_errors(&self) -> &Vec<error::Error> {
        return self.tokens.get_errors();
    }

    pub fn log_error(&mut self, error: error::Error) -> Result<error::ErrorId, error::ErrorId> {
        let errorid = error.id;
        self.errors.log_error(error);
        if self.errors.reached_error_limit() {
            return Err(error::new_error_id(errors::ErrorLimitExceeded));
        }

        return Ok(errorid);
    }

    fn advance(&mut self) {
        loop {
            let t = self.tokens.read_token();

            if t.is_some() {
                self.temp_tokencount += 1;

                if t.as_ref().unwrap().tokentype == TokenType::Comment {
                    continue;
                }
            }

            self.last_token = std::mem::replace(&mut self.current_token, t);
            break;
        }
    }

    fn accept(&mut self, t: TokenType) -> bool {
        match &self.current_token {
            Some(ct) if ct.tokentype == t => {
                self.advance();
                return true;
            }
            _ => {
                return false;
            }
        }
    }

    fn expect(&mut self, expected_token: TokenType) -> Result<(), error::ErrorId> {
        if !self.accept(expected_token) {
            if let Some(current_token) = &self.current_token {
                let span = current_token.source_span;
                let error = format!(
                    "Unexpected token! Expected '{:?}', got '{:?}'",
                    expected_token, current_token.tokentype
                );
                return Err(self.log_error(error::Error::at_span(
                    errors::UnexpectedToken,
                    span,
                    error,
                ))?);
            } else {
                return Err(self.log_error(error::Error::at_span(
                    errors::UnexpectedEOF,
                    // TODO: This is bad, but can be fixed by introducing EOS token
                    //  pointing to end of file
                    self.last_token.as_ref().unwrap().source_span,
                    "Unexpected end of file!".into(),
                ))?);
            }
        }

        return Ok(());
    }

    fn get_last_token_text(&mut self) -> String {
        return self
            .tokens
            .get_token_string(self.last_token.as_ref().unwrap());
    }

    fn get_last_token_symbol(&mut self) -> SymbolRef {
        let text = self.get_last_token_text();
        return self.ast.add_symbol(&*text);
    }

    fn parse_input_parameter(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if self.accept(TokenType::Identifier) {
            let node = self.ast.reserve_node();
            let symbol = self.get_last_token_symbol();

            self.expect(TokenType::Colon)?;

            if let Some(n) = self.parse_expression()? {
                return Ok(Some(
                    self.ast.replace_node(
                        node,
                        ast::nodes::InputParameter {
                            symbol,
                            typeexpr: n,
                        }
                        .into(),
                    ),
                ));
            } else {
                return Err(self.log_error(error::Error::at_span(
                    errors::ExpectedExpression,
                    self.last_token.as_ref().unwrap().source_span,
                    "Expected type expression for input parameter".into(),
                ))?);
            }
        }

        return Ok(None);
    }

    fn parse_output_parameter(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        let node = self.ast.reserve_node();

        if let Some(n) = self.parse_expression()? {
            return Ok(Some(self.ast.replace_node(
                node,
                ast::nodes::OutputParameter { typeexpr: n }.into(),
            )));
        }

        self.ast.undo_node_reservation(node);
        return Ok(None);
    }

    fn parse_function_literal_or_type(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if self.accept(TokenType::Func) {
            let node = self.ast.reserve_node();
            let mut inputparams = Vec::new();
            let mut outputparams = Vec::new();

            // Optional input parameters
            if self.accept(TokenType::OpeningParenthesis) {
                if let Some(n) = self.parse_input_parameter()? {
                    inputparams.push(n);

                    while self.accept(TokenType::Comma) {
                        if let Some(n) = self.parse_input_parameter()? {
                            inputparams.push(n);
                        } else {
                            return Err(self.log_error(error::Error::at_span(
                                errors::ExpectedInputParameterDeclaration,
                                self.last_token.as_ref().unwrap().source_span,
                                "Expected input parameter declaration".into(),
                            ))?);
                        }
                    }
                }

                self.expect(TokenType::ClosingParenthesis)?;
            }

            // Optional output paramters
            if self.accept(TokenType::Arrow) {
                if self.accept(TokenType::OpeningParenthesis) {
                    if let Some(n) = self.parse_output_parameter()? {
                        outputparams.push(n);

                        while self.accept(TokenType::Comma) {
                            if let Some(n) = self.parse_output_parameter()? {
                                outputparams.push(n);
                            } else {
                                return Err(self.log_error(error::Error::at_span(
                                    errors::ExpectedOutputParameterDeclaration,
                                    self.last_token.as_ref().unwrap().source_span,
                                    "Expected output parameter declaration".into(),
                                ))?);
                                // TODO: Recovery?
                            }
                        }
                    }

                    self.expect(TokenType::ClosingParenthesis)?;
                } else if let Some(n) = self.parse_output_parameter()? {
                    outputparams.push(n);
                } else {
                    // No recovery necesary here
                    return Err(self.log_error(error::Error::at_span(
                        errors::ExpectedOutputParameterDeclaration,
                        self.last_token.as_ref().unwrap().source_span,
                        "Expected output parameter declaration".into(),
                    ))?);
                }
            }

            // If there is a body following, we are dealing with a function literal
            //  otherwise, a type literal
            if self.accept(TokenType::Do) {
                // TODO: LB and Indent should probably not be hard requirements
                self.expect(TokenType::LineBreak)?;
                //self.expect(TokenType::Indentation)?; // Does not work for empty bodies

                let body = self.parse_statementbody()?;

                self.expect(TokenType::End)?;

                return Ok(Some(
                    self.ast.replace_node(
                        node,
                        ast::nodes::FunctionLiteral {
                            inputparams,
                            outputparams,
                            body,
                        }
                        .into(),
                    ),
                ));
            }
        }

        return Ok(None);
    }

    fn parse_argumentlist(&mut self) -> Result<ast::NodeRef, error::ErrorId> {
        let node = self.ast.reserve_node();
        let mut args = Vec::new();

        // TODO: Add new node for argument expression
        if let Some(n) = self.parse_expression()? {
            args.push(n);

            while self.accept(TokenType::Comma) {
                if let Some(n) = self.parse_expression()? {
                    args.push(n);
                } else {
                    return Err(self.log_error(error::Error::at_span(
                        errors::ExpectedExpression, // TODO: <- should probably be bepoke error
                        self.last_token.as_ref().unwrap().source_span,
                        "Expected argument".into(),
                    ))?);
                }
            }
        }

        return Ok(self
            .ast
            .replace_node(node, ast::nodes::ArgumentList { args: args }.into()));
    }

    fn accept_binaryoperator(&mut self) -> Option<ast::BinaryOperationType> {
        if let Some(tt) = &self.current_token {
            let r = match tt.tokentype {
                TokenType::Plus => Some(ast::BinaryOperationType::Add),
                TokenType::Minus => Some(ast::BinaryOperationType::Sub),
                TokenType::Star => Some(ast::BinaryOperationType::Mul),
                TokenType::Slash => Some(ast::BinaryOperationType::Div),
                _ => None,
            };

            if r.is_some() {
                self.advance();
            }

            return r;
        }

        return None;
    }

    fn parse_if_statement(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if self.accept(TokenType::If) {
            let node = self.ast.reserve_node();

            if let Some(condition) = self.parse_expression()? {
                let mut usingstatementbody = false;

                // Then
                let thenstmnt = if self.accept(TokenType::Then) {
                    let body = self.parse_statementbody()?;
                    usingstatementbody = true;
                    Ok(body)
                } else {
                    if let Some(expr) = self.parse_expression()? {
                        Ok(expr)
                    } else {
                        Err(self.log_error(error::Error::at_span(
                            errors::ExpectedExpression,
                            self.current_token.as_ref().unwrap().source_span,
                            "Expected expression".into(),
                        ))?)
                    }
                };

                // Else
                let elsestmnt = if self.accept(TokenType::Else) {
                    if usingstatementbody {
                        let body = self.parse_statementbody()?;
                        self.expect(TokenType::End)?;
                        Ok(Some(body))
                    } else {
                        if let Some(expr) = self.parse_expression()? {
                            Ok(Some(expr))
                        } else {
                            Err(self.log_error(error::Error::at_span(
                                errors::ExpectedExpression,
                                self.current_token.as_ref().unwrap().source_span,
                                "Expected expression".into(),
                            ))?)
                        }
                    }
                } else {
                    if usingstatementbody {
                        self.expect(TokenType::End)?;
                    }
                    Ok(None)
                };

                if let Err(id) = elsestmnt {
                    return Err(id);
                }

                return Ok(Some(
                    self.ast.replace_node(
                        node,
                        ast::nodes::IfStatement {
                            condition,
                            thenstmnt: thenstmnt.unwrap(),
                            elsestmnt: elsestmnt.unwrap(),
                        }
                        .into(),
                    ),
                ));
            } else {
                return Err(self.log_error(error::Error::at_span(
                    errors::ExpectedExpression,
                    self.current_token.as_ref().unwrap().source_span,
                    "Expected expression in conditional".into(),
                ))?);
            }
        }

        return Ok(None);
    }

    fn parse_return_statement(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if self.accept(TokenType::Return) {
            let node = self.ast.reserve_node();

            let expr = self.parse_expression()?;

            // TODO: Parse end of statement

            return Ok(Some(
                self.ast
                    .replace_node(node, ast::nodes::ReturnStatement { expr }.into()),
            ));
        }

        return Ok(None);
    }

    fn parse_symbol_declaration(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        let decltype = if self.accept(TokenType::Def) {
            Some(ast::SymbolDeclarationType::Def)
        } else if self.accept(TokenType::Var) {
            Some(ast::SymbolDeclarationType::Var)
        } else {
            None
        };

        if let Some(decltype) = decltype {
            let node = self.ast.reserve_node();

            self.expect(TokenType::Identifier)?;
            let symbol = self.get_last_token_symbol();

            // TODO: Optional type specifier

            // Defines must be initalized to a value
            self.expect(TokenType::Equals)?;

            if let Some(n) = self.parse_expression()? {
                return Ok(Some(
                    self.ast.replace_node(
                        node,
                        ast::nodes::SymbolDeclaration {
                            symbol: symbol,
                            decltype: decltype,
                            typeexpr: None,
                            initexpr: n,
                        }
                        .into(),
                    ),
                ));
            } else {
                return Err(self.log_error(error::Error::at_span(
                    errors::ExpectedExpression,
                    self.current_token.as_ref().unwrap().source_span,
                    "Expected expression for initialization value".into(),
                ))?);
            }
        }

        return Ok(None);
    }

    fn parse_statement(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if let Some(n) = self.parse_symbol_declaration()? {
            return Ok(Some(n));
        } else if let Some(n) = self.parse_if_statement()? {
            return Ok(Some(n));
        } else if let Some(n) = self.parse_return_statement()? {
            return Ok(Some(n));
        } else if let Some(n) = self.parse_expression()? {
            return Ok(Some(n));
        }
        return Ok(None);
    }

    fn parse_statementbody(&mut self) -> Result<ast::NodeRef, error::ErrorId> {
        let node = self.ast.reserve_node();

        let mut statements: Vec<ast::NodeRef> = Vec::new();

        while self.current_token.is_some() {
            // TODO: For now, just eat all linebreaks and indentation between statements
            while self.accept(TokenType::LineBreak) || self.accept(TokenType::Indentation) {}

            match self.parse_statement() {
                Err(error::ErrorId::FatalError(errors::ErrorLimitExceeded)) => {
                    return Err(error::ErrorId::FatalError(errors::ErrorLimitExceeded))
                }
                Err(_) => {
                    // Error recovery, eat everything until next new line
                    // TODO: Use indentation to skip breaks that mean line continuations
                    while let Some(t) = &self.current_token {
                        if t.tokentype == TokenType::LineBreak {
                            break;
                        }
                        self.advance();
                    }
                    continue;
                }
                Ok(Some(n)) => {
                    statements.push(n);
                }
                _ => break,
            }
        }

        return Ok(self.ast.replace_node(
            node,
            ast::nodes::StatementBody {
                statements: statements,
            }
            .into(),
        ));
    }

    // Parse fragment (usually file)
    fn parse_fragment(&mut self) -> Result<(), error::ErrorId> {
        let node = self.ast.reserve_node();
        self.ast.set_root(node);

        let body = self.parse_statementbody()?;
        self.ast.replace_node(
            node,
            ast::nodes::ModuleFragment {
                statementbody: body,
            }
            .into(),
        );

        return Ok(());
    }

    pub fn parse(&mut self) {
        self.advance();

        match self.parse_fragment() {
            Err(error::ErrorId::FatalError(errors::ErrorLimitExceeded)) => {
                println!("Parsing stopped, error limit exceeed");
                return;
            }
            Err(e) => {
                panic!("Unhandled error! {:?}", e);
            }
            _ => (),
        }

        if self.tokens.read_token() == None {
            println!("Parsed all {} tokens successfully!", self.temp_tokencount);
        } else {
            println!("Only parsed {} tokens...", self.temp_tokencount);

            println!("Unparsed tokens:");
            while let Some(t) = self.tokens.read_token() {
                println!("{:?}", t);
            }
        }
    }

    pub fn print_ast(&mut self) {
        // TODO: Move to fudgec
        println!("AST:");
        self.ast.print(4);
    }
}
