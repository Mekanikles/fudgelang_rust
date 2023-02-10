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
    // Track line_pos_ token_pos and indentation separately to report indentation
    //  that includes spaces correctly.
    line_pos: u64,
    first_token_pos: u64,
    line_number: u64,
    indentation: u32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BlockInfo {
    line: LineInfo,
    start_pos: u64,
    level: u64,
}

struct Parser<'a> {
    tokens: &'a mut TokenStream<'a>,
    current_token: Option<Token>,
    last_token: Option<Token>,
    temp_tokencount: u32,
    pub ast: Ast,
    errors: error::ErrorManager,
    block_level: u64,
    blocks: Vec<BlockInfo>,
    current_line: LineInfo,
    need_normal_layout_check: bool,
    ismain: bool,
}

pub struct ParserResult {
    pub ast: Ast,
    pub errors: Vec<error::Error>,
}

pub fn parse<'a>(tokens: &'a mut TokenStream<'a>, ismain: bool) -> ParserResult {
    let mut parser = Parser::new(tokens, ismain);

    if ismain {
        parser.parse_main_file();
    } else {
        parser.parse_module_file();
    }

    return ParserResult {
        ast: parser.ast,
        errors: parser.errors.error_data.errors,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenLayoutType {
    BlockKeyword,
    BlockLinker,
    BlockEnd,
    None,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a mut TokenStream<'a>, ismain: bool) -> Self {
        let source_name = tokens.get_source_name().to_string();
        Parser {
            tokens: tokens,
            current_token: None,
            last_token: None,
            temp_tokencount: 0,
            ast: Ast::new(source_name),
            errors: error::ErrorManager::new(),
            block_level: 0,
            blocks: Vec::new(),
            current_line: LineInfo {
                line_pos: 0,
                first_token_pos: 0,
                line_number: 0,
                indentation: 0,
            },
            need_normal_layout_check: false,
            ismain: ismain,
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

        let mut found_newline = false;
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
                        current_line.line_number += 1;
                        current_line.indentation = 0;
                        current_line.line_pos =
                            t.unwrap().source_span.pos + t.unwrap().source_span.len as u64;
                        found_newline = true;
                        continue;
                    }
                    TokenType::Indentation => {
                        // TODO: This is cheating a bit, when spaces are involved
                        //  the source span here will not be correct.
                        current_line.indentation = t.unwrap().source_span.len as u32;
                        continue;
                    }
                    _ => (),
                }
            }

            self.last_token = std::mem::replace(&mut self.current_token, t.cloned());
            break;
        }

        // Track line starting pos
        if (found_newline || self.last_token.is_none()) && self.current_token.is_some() {
            current_line.first_token_pos = self.current_token.unwrap().source_span.pos;
            self.need_normal_layout_check = true;
        }
    }

    fn accept(&mut self, t: TokenType) -> bool {
        return self.accept_with_layout(t, TokenLayoutType::None);
    }

    fn accept_with_layout(&mut self, tokentype: TokenType, layouttype: TokenLayoutType) -> bool {
        match &self.current_token {
            Some(ct) if ct.tokentype == tokentype => {
                let pos = ct.source_span.pos;

                // TODO: These can fail with a max error reached
                if let Some(lb) = self.blocks.last() {
                    if layouttype != TokenLayoutType::None {
                        let aligns_vertically = ct.source_span.pos
                            == self.current_line.first_token_pos
                            && self.current_line.indentation == lb.line.indentation
                            && lb.start_pos == lb.line.first_token_pos;
                        let aligns_horizontally = self.current_line == lb.line;

                        // Keyword must be horizontally aligned, if possible,
                        //  except for block linkers
                        let correctly_aligns = aligns_horizontally
                            || (aligns_vertically
                                && (layouttype == TokenLayoutType::BlockLinker
                                    || layouttype == TokenLayoutType::BlockEnd
                                    || (self.current_line.line_number > lb.line.line_number + 1)));

                        // Block keywords needs to align either horizontally or vertically
                        // TODO: Force horizontally if possible
                        if !correctly_aligns {
                            let _ = self.log_error(error::Error::at_span(
                                errors::MismatchedAlignment,
                                ct.source_span,
                                if layouttype == TokenLayoutType::BlockLinker || layouttype == TokenLayoutType::BlockEnd {
                                    format!("Keyword linking/closing a block needs to align to block start horizontally or vertically!")
                                } else {
                                    format!("Keyword needs to align to block start horizontally if possible and vertically otherwise!")
                                }
                                .into(),
                            ));
                        }
                    } else if self.need_normal_layout_check {
                        // Everything except the body-keywords have to be either on the same
                        //  line as the block start, or 1 indentation under it
                        // We only need to check this once for each new line
                        if lb.line.line_number < self.current_line.line_number {
                            let _ = self.expect_indentation(lb.line.indentation + 1);
                        }

                        self.need_normal_layout_check = false;
                    }
                } else {
                    // File-level tokens are not indented
                    let _ = self.expect_indentation(0);
                }

                // If we had blocks "queued up", start them
                self.start_blocks_if_needed(pos);

                // A block linker starts a new block
                if layouttype == TokenLayoutType::BlockLinker {
                    self.replace_current_block(pos);
                }
                // Block keywords "extends" the current block
                //  to allow line breaks in block following keyword rules
                else if layouttype == TokenLayoutType::BlockKeyword {
                    // This will however not work for a keyword in the middle of a row
                    let is_new_line = pos == self.current_line.first_token_pos;
                    if is_new_line {
                        self.replace_current_block(pos)
                    }
                }

                self.advance();
                return true;
            }
            _ => {
                return false;
            }
        }
    }

    fn expect_nobreak(
        &mut self,
        expected_token: TokenType,
        layouttype: TokenLayoutType,
    ) -> Result<bool, error::ErrorId> {
        if !self.accept_with_layout(expected_token, layouttype) {
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
        self.expect_with_layout(expected_token, TokenLayoutType::None)
    }

    fn expect_with_layout(
        &mut self,
        expected_token: TokenType,
        layouttype: TokenLayoutType,
    ) -> Result<(), error::ErrorId> {
        if !self.expect_nobreak(expected_token, layouttype)? {
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

    fn push_block(&mut self) {
        // We just increase the block level here, actual blocks
        //  will be started on demand when tokens are accepted
        self.block_level += 1;
    }

    fn pop_block(&mut self) -> Option<BlockInfo> {
        // If the current block matched this level, end it
        let result = if let Some(cb) = self.blocks.last() {
            debug_assert!(cb.level <= self.block_level);
            if cb.level == self.block_level {
                Some(self.end_current_block())
            } else {
                None
            }
        } else {
            None
        };

        debug_assert!(self.block_level > 0);
        self.block_level -= 1;

        return result;
    }

    fn start_blocks_if_needed(&mut self, start_pos: u64) {
        let current_started_block_level = if let Some(cb) = self.blocks.last() {
            cb.level
        } else {
            0
        };

        debug_assert!(current_started_block_level <= self.block_level);

        for level in current_started_block_level..self.block_level {
            self.blocks.push(BlockInfo {
                line: self.current_line,
                start_pos: start_pos,
                level: level + 1,
            });
        }
    }

    fn end_current_block(&mut self) -> BlockInfo {
        debug_assert!(self.blocks.len() > 0);
        return self.blocks.pop().unwrap();
    }

    fn replace_current_block(&mut self, start_pos: u64) {
        let mut cb = &mut self.blocks.last_mut().unwrap();
        cb.start_pos = start_pos;
        cb.line = self.current_line;
    }

    fn expect_new_line(&mut self, block: &BlockInfo, str: &str) {
        let is_new_line = block.start_pos == block.line.first_token_pos;
        if !is_new_line {
            let _ = self.log_error(error::Error::at_span(
                errors::ExpectedNewLine,
                SourceSpan {
                    pos: block.start_pos - 1,
                    len: 1,
                }, // TODO: Not ideal, we should probably save positions or tokens in blocks
                format!("Expected {} to start on new line", str).into(),
            ));
        }
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
            if self.accept_with_layout(TokenType::Do, TokenLayoutType::BlockKeyword) {
                let body = self.parse_statementbody()?;

                self.expect_with_layout(TokenType::End, TokenLayoutType::BlockEnd)?;

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

    fn parse_module_declaration(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        // TODO: This is pretty hacky, should the parser really extract this info?
        // The module identifier cannot be computed as an expression, though
        if self.accept(TokenType::Module) {
            let statement_pos = self.last_token.unwrap().source_span.pos;

            self.expect(TokenType::Identifier)?;
            let symbol = self.get_last_token_symbol();

            // TODO: This needs to be easier
            let source_span = SourceSpan {
                pos: statement_pos,
                len: (self.last_token.unwrap().source_span.pos - statement_pos) as usize
                    + self.last_token.unwrap().source_span.len,
            };

            if self.accept_with_layout(TokenType::Begin, TokenLayoutType::BlockKeyword) {
                let body = self.parse_statementbody()?;
                self.expect_with_layout(TokenType::End, TokenLayoutType::BlockEnd)?;

                return Ok(Some(
                    self.ast.add_node(
                        ast::nodes::Module {
                            symbol: symbol,
                            statementbody: body,
                        }
                        .into(),
                    ),
                ));
            } else {
                // Main file does not support module self declarations
                if self.ismain {
                    return Err(self.log_error(error::Error::at_span(
                        errors::ModuleDeclarationInMain,
                        source_span,
                        "Main module cannot have module declarations".into(),
                    ))?);
                }

                // TODO: Add support for inline modules with "begin" here
                if self.ast.module.is_some() {
                    return Err(self.log_error(error::Error::at_span(
                        errors::ModuleAlreadyDeclared,
                        source_span,
                        "Module already declared".into(),
                    ))?);
                } else if self.ast.contains_more_than_root() {
                    self.log_error(error::Error::at_span(
                        errors::ModuleDeclarationNotAtTop,
                        source_span,
                        "Module declaration needs to reside before any other statements in this file"
                            .into(),
                    ))?;
                }

                // TODO: Probably should not happen here
                // Feels like this should happen in some symbol declaration step
                //  publishing the module name together with the symbols
                self.ast.module = Some(symbol);

                return Ok(Some(self.ast.add_node(
                    ast::nodes::ModuleSelfDeclaration { symbol: symbol }.into(),
                )));
            }
        }

        Ok(None)
    }

    fn parse_if_statement(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if self.accept(TokenType::If) {
            let node = self.ast.reserve_node();

            // Expr
            let condition = self.expect_expression()?;

            let mut branches: Vec<(ast::NodeRef, ast::NodeRef)> = Vec::new();
            let mut elsebranch: Option<ast::NodeRef> = None;

            // Primary branch
            self.expect_with_layout(TokenType::Then, TokenLayoutType::BlockKeyword)?;
            {
                branches.push((condition, self.parse_statementbody()?));
            }

            // Else-if branches
            while self.accept_with_layout(TokenType::ElseIf, TokenLayoutType::BlockLinker) {
                let condition = self.expect_expression()?;

                self.expect_with_layout(TokenType::Then, TokenLayoutType::BlockKeyword)?;

                branches.push((condition, self.parse_statementbody()?));
            }

            // Final else
            if self.accept_with_layout(TokenType::Else, TokenLayoutType::BlockLinker) {
                // Final else
                elsebranch = Some(self.parse_statementbody()?);
            }

            self.expect_with_layout(TokenType::End, TokenLayoutType::BlockEnd)?;

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
        // Statements that start with an expression can open more blocks on the same token,
        //  like "a = b", where both a and b will open new blocks.
        // In this case we want a total of 3 blocks, 2 starting with a, 1 with b.
        self.push_block();
        let res = self.parse_statement_inner();
        if let Some(block) = self.pop_block() {
            // Statements needs to start on a new line
            self.expect_new_line(&block, "statement");
        }

        return res;
    }

    fn parse_statement_inner(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if let Some(n) = self.parse_symbol_declaration()? {
            return Ok(Some(n));
        } else if let Some(n) = self.parse_if_statement()? {
            return Ok(Some(n));
        } else if let Some(n) = self.parse_return_statement()? {
            return Ok(Some(n));
        } else if let Some(n) = self.parse_expression()? {
            return Ok(Some(n));
        } else if let Some(n) = self.parse_module_declaration()? {
            return Ok(Some(n));
        }
        return Ok(None);
    }

    fn expect_indentation(&mut self, expected: u32) -> Result<(), error::ErrorId> {
        let indentation = self.current_line.indentation;
        if indentation != expected {
            self.log_error(error::Error::at_span(
                errors::MismatchedIndentation,
                SourceSpan {
                    pos: self.current_line.line_pos,
                    len: (self.current_line.first_token_pos - self.current_line.line_pos) as usize,
                },
                format!(
                    "Mismatched indentation level, expected: {}, was {}",
                    expected, indentation
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

    fn parse_module_file_internal(&mut self) -> Result<(), error::ErrorId> {
        let node = self.ast.reserve_node();
        self.ast.set_root(node);

        let body = self.parse_statementbody()?;
        let symbol = self
            .ast
            .module
            .unwrap_or(self.ast.add_symbol("TODO_Need_default_name"));
        self.ast.replace_node(
            node,
            ast::nodes::Module {
                symbol: symbol,
                statementbody: body,
            }
            .into(),
        );

        return Ok(());
    }

    fn parse_main_file_internal(&mut self) -> Result<(), error::ErrorId> {
        let node = self.ast.reserve_node();
        self.ast.set_root(node);

        let body = self.parse_statementbody()?;

        self.ast.replace_node(
            node,
            ast::nodes::EntryPoint {
                statementbody: body,
            }
            .into(),
        );

        return Ok(());
    }

    fn parse(&mut self, inner: &dyn Fn(&mut Self) -> Result<(), error::ErrorId>) {
        self.advance();

        match inner(self) {
            Err(error::ErrorId::FatalError(errors::ErrorLimitExceeded)) => {
                eprintln!("Parsing stopped, error limit exceeed");
                return;
            }
            Err(e) => {
                panic!("Unhandled error! {:?}", e);
            }
            _ => (),
        }

        // TODO: this sucks
        if self.current_token.is_some() {
            let span = self.current_token.unwrap().source_span;
            let _ = self.log_error(error::Error::at_span(
                errors::UnexpectedToken,
                span,
                "Unparsed token!".into(),
            ));
        }

        debug_assert!(self.blocks.is_empty());

        if self.tokens.read_token() == None {
            eprintln!("Parsed all {} tokens successfully!", self.temp_tokencount);
        } else {
            eprintln!("Only parsed {} tokens...", self.temp_tokencount);

            eprintln!("Unparsed tokens:");
            while let Some(t) = self.tokens.read_token() {
                eprintln!("{:?}", t);
            }
        }
    }

    pub fn parse_main_file(&mut self) {
        self.parse(&Self::parse_main_file_internal);
    }

    pub fn parse_module_file(&mut self) {
        self.parse(&Self::parse_module_file_internal);
    }
}
