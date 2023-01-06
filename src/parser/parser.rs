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

#[derive(Debug, Clone, Copy, PartialEq)]
struct LineInfo {
    start_pos: u64,
    line_number: u64,
    indentation: u32,
    padding: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum LineState {
    AcceptingIndentationOrPadding,
    AcceptingPadding,
    AcceptingContent,
}

struct Parser<'a> {
    tokens: &'a mut TokenStream<'a>,
    current_token: Option<Token>,
    last_token: Option<Token>,
    temp_tokencount: u32,
    pub ast: Ast,
    errors: error::ErrorManager,
    block_starts: Vec<LineInfo>,
    current_line: LineInfo,
    layout_checked_at_line_number: u64,
    line_state: LineState,
}

pub struct ParserResult {
    pub ast: Ast,
    pub errors: Vec<error::Error>,
}

pub fn parse<'a>(tokens: &'a mut TokenStream<'a>) -> ParserResult {
    let mut parser = Parser::new(tokens);
    parser.parse();

    return ParserResult {
        ast: parser.ast,
        errors: parser.errors.error_data.errors,
    };
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a mut TokenStream<'a>) -> Self {
        Parser {
            tokens: tokens,
            current_token: None,
            last_token: None,
            temp_tokencount: 0,
            ast: Ast::new(),
            errors: error::ErrorManager::new(),
            block_starts: Vec::from([LineInfo {
                start_pos: 0,
                line_number: 1,
                indentation: 0,
                padding: 0,
            }]),
            current_line: LineInfo {
                start_pos: 0,
                line_number: 1,
                indentation: 0,
                padding: 0,
            },
            layout_checked_at_line_number: 0,
            line_state: LineState::AcceptingIndentationOrPadding,
        }
    }

    pub fn log_error(&mut self, error: error::Error) -> Result<error::ErrorId, error::ErrorId> {
        let errorid = error.id;
        self.errors.log_error(error);
        if self.errors.reached_error_limit() {
            return Err(error::new_error_id(errors::ErrorLimitExceeded));
        }

        return Ok(errorid);
    }

    pub fn last_errorid(&self) -> Option<error::ErrorId> {
        if let Some(e) = self.errors.error_data.errors.last() {
            Some(e.id)
        } else {
            None
        }
    }

    fn advance(&mut self) {
        let mut current_line = &mut self.current_line;
        loop {
            let t = self.tokens.read_token();

            if t.is_some() {
                self.temp_tokencount += 1;

                let tt = t.unwrap().tokentype;

                match tt {
                    TokenType::Comment => {
                        continue;
                    }
                    TokenType::LineBreak => {
                        self.line_state = LineState::AcceptingIndentationOrPadding;
                        current_line.line_number += 1;
                        current_line.indentation = 0;
                        current_line.padding = 0;
                        continue;
                    }
                    TokenType::Indentation => {
                        assert!(self.line_state == LineState::AcceptingIndentationOrPadding);
                        current_line.indentation = t.unwrap().source_span.len as u32;
                        self.line_state = LineState::AcceptingPadding;
                        continue;
                    }
                    TokenType::Padding => {
                        assert!(
                            self.line_state == LineState::AcceptingPadding
                                || self.line_state == LineState::AcceptingIndentationOrPadding
                        );
                        current_line.padding = t.unwrap().source_span.len as u32;
                        self.line_state = LineState::AcceptingContent;
                        continue;
                    }
                    _ => (),
                }
            }

            self.last_token = std::mem::replace(&mut self.current_token, t.cloned());
            break;
        }
    }

    fn accept(&mut self, t: TokenType) -> bool {
        match &self.current_token {
            Some(ct) if ct.tokentype == t => {
                if self.layout_checked_at_line_number < self.current_line.line_number {
                    let lb = *self.block_starts.last().unwrap();
                    // TODO: These can fail with a max error reached
                    let _ = self.expect_indentation(self.current_line.indentation, &lb);
                    let _ = self.expect_padding(self.current_line.padding, &lb);
                    self.layout_checked_at_line_number = self.current_line.line_number;
                }

                self.advance();
                return true;
            }
            _ => {
                return false;
            }
        }
    }

    fn expect_nobreak(&mut self, expected_token: TokenType) -> Result<bool, error::ErrorId> {
        if !self.accept(expected_token) {
            if let Some(current_token) = &self.current_token {
                let span = current_token.source_span;
                let error = format!(
                    "Unexpected token! Expected '{:?}', got '{:?}'",
                    expected_token, current_token.tokentype
                );
                self.log_error(error::Error::at_span(errors::UnexpectedToken, span, error))?;
            } else {
                return Err(self.log_error(error::Error::at_span(
                    errors::UnexpectedEOF,
                    // TODO: This is bad, but can be fixed by introducing EOS token
                    //  pointing to end of file
                    self.last_token.as_ref().unwrap().source_span,
                    "Unexpected end of file!".into(),
                ))?);
            }

            return Ok(false);
        }

        return Ok(true);
    }

    fn expect(&mut self, expected_token: TokenType) -> Result<(), error::ErrorId> {
        if !self.expect_nobreak(expected_token)? {
            return Err(self.last_errorid().unwrap());
        }

        return Ok(());
    }

    fn get_last_token_text(&self) -> &str {
        return self
            .tokens
            .get_token_string(self.last_token.as_ref().unwrap());
    }

    fn get_last_token_symbol(&mut self) -> SymbolRef {
        let text = self.get_last_token_text().to_string();
        return self.ast.add_symbol(&*text);
    }

    fn block_start(&mut self) {
        let current_block = self.block_starts.last().unwrap();
        self.block_starts.push(LineInfo {
            start_pos: current_block.start_pos,
            line_number: current_block.line_number,
            indentation: current_block.indentation + 1,
            padding: current_block.padding,
        });
    }

    fn block_end(&mut self) {
        self.block_starts.pop();
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
                self.block_start();

                let body = self.parse_statementbody()?;

                self.block_end();

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
                TokenType::CompareEq => Some(ast::BinaryOperationType::Equals),
                TokenType::GreaterThan => Some(ast::BinaryOperationType::GreaterThan),
                TokenType::GreaterThanOrEq => Some(ast::BinaryOperationType::GreaterThanOrEq),
                TokenType::LessThan => Some(ast::BinaryOperationType::LessThan),
                TokenType::LessThanOrEq => Some(ast::BinaryOperationType::LessThanOrEq),
                _ => None,
            };

            if r.is_some() {
                self.advance();
            }

            return r;
        }

        return None;
    }

    fn expect_expression(&mut self) -> Result<ast::NodeRef, error::ErrorId> {
        if let Some(expr) = self.parse_expression()? {
            return Ok(expr);
        } else {
            return Err(self.log_error(error::Error::at_span(
                errors::ExpectedExpression,
                self.current_token.as_ref().unwrap().source_span,
                "Expected expression".into(),
            ))?);
        }
    }

    fn parse_if_statement(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if self.accept(TokenType::If) {
            let node = self.ast.reserve_node();

            // Expr
            let condition = self.expect_expression()?;

            let mut branches: Vec<(ast::NodeRef, ast::NodeRef)> = Vec::new();
            let mut elsebranch: Option<ast::NodeRef> = None;

            // Primary branch
            self.expect(TokenType::Then)?;
            {
                self.block_start();

                branches.push((condition, self.parse_statementbody()?));

                self.block_end();
            }

            // Else-if branches
            while self.accept(TokenType::ElseIf) {
                let condition = self.expect_expression()?;

                self.expect(TokenType::Then)?;
                self.block_start();

                branches.push((condition, self.parse_statementbody()?));

                self.block_end();
            }

            // Final else
            if self.accept(TokenType::Else) {
                self.block_start();

                // Final else
                elsebranch = Some(self.parse_statementbody()?);

                self.block_end();
            }

            self.expect(TokenType::End)?;

            return Ok(Some(
                self.ast.replace_node(
                    node,
                    ast::nodes::IfStatement {
                        branches,
                        elsebranch,
                    }
                    .into(),
                ),
            ));
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

    fn expect_indentation(
        &mut self,
        indentation: u32,
        line_info: &LineInfo,
    ) -> Result<(), error::ErrorId> {
        if line_info.indentation != indentation {
            self.log_error(error::Error::at_span(
                errors::MismatchedIndentation,
                self.current_token.as_ref().unwrap().source_span,
                format!(
                    "Mismatched indentation level, expected: {}, was {}",
                    line_info.indentation, indentation
                )
                .into(),
            ))?;
        }
        return Ok(());
    }

    fn expect_padding(&mut self, padding: u32, line_info: &LineInfo) -> Result<(), error::ErrorId> {
        if line_info.padding != padding {
            self.log_error(error::Error::at_span(
                errors::MismatchedPadding,
                self.current_token.as_ref().unwrap().source_span,
                format!(
                    "Mismatched padding, should be {}, was {}",
                    line_info.padding, padding
                )
                .into(),
            ))?;
        }
        return Ok(());
    }

    fn parse_statementbody(&mut self) -> Result<ast::NodeRef, error::ErrorId> {
        let node = self.ast.reserve_node();

        let mut statements: Vec<ast::NodeRef> = Vec::new();

        while self.current_token.is_some() {
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

        // TODO: this sucks
        if self.current_token.is_some() {
            let span = self.current_token.unwrap().source_span;
            self.log_error(error::Error::at_span(
                errors::UnexpectedToken,
                span,
                "Unparsed token!".into(),
            ))?;
        }

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
}
