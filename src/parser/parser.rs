use crate::error;
use crate::error::errors;
use crate::scanner::*;
use crate::source::*;
use super::tokenstream::TokenStream;
use super::ast;
use super::ast::Ast;

pub struct Parser<'a, T: TokenStream>
{
    tokens: &'a mut T,
    current_token: Option<Token>,
    last_token: Option<Token>,
    temp_tokencount: u32,
    ast: Ast,
    errors : error::ErrorManager,
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

    pub fn get_errors(&self) -> &Vec<error::Error> {
        return self.errors.get_errors();
    }

    pub fn log_error(&mut self, error : error::Error) -> Result<error::ErrorId, error::ErrorId> {
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

    fn accept(&mut self, t : TokenType) -> bool {
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

    fn expect(&mut self, t : TokenType) -> Result<(), error::ErrorId> {
        if !self.accept(t) {
            if let Some(t) = &self.current_token {
                let span = t.source_span;
                return Err(self.log_error(error::Error::at_span(
                    errors::UnexpectedToken,
                    span,
                    "Unexpected token!".into(),
                ))?);
            }
            else {
                return Err(self.log_error(error::Error::at_span(
                    errors::UnexpectedEOF,
                    // TODO: This is bad, but can be fixed by introducing EOS token
                    //  pointing to end of file
                    self.last_token.as_ref().unwrap().source_span,
                    "Unexpected termination token!".into(),
                ))?);
            }
            // TODO: Add error recovery. Should we just bubble up errors to statement body, or nearest block/brackets?
            //panic!("Unexpected token!");
        }

        return Ok(());
    }

    fn get_last_token_text(&mut self) -> String {
        return self.tokens.get_token_string(self.last_token.as_ref().unwrap());
    }

    fn get_last_token_symbol(&mut self) -> ast::SymbolRef {
        let text = self.get_last_token_text();
        return self.ast.add_symbol(&*text);
    }

    fn parse_input_parameter(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if self.accept(TokenType::Identifier) {
            let node = self.ast.reserve_node();
            let symbol = self.get_last_token_symbol();

            self.expect(TokenType::Colon)?;

            if let Some(n) = self.parse_expression()? {
                return Ok(Some(self.ast.replace_node(node, ast::Node::InputParameter {
                    symbol,
                    typeexpr: n,
                })));
            }
            else {
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
            return Ok(Some(self.ast.replace_node(node, ast::Node::OutputParameter {
                typeexpr: n,
            })));
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
                        }
                        else {
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
                            }
                            else {
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
                }
                else if let Some(n) = self.parse_output_parameter()? {
                    outputparams.push(n);
                }
                else {
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
                self.expect(TokenType::LineBreak)?; self.expect(TokenType::Indentation)?;

                let body = self.parse_statementbody()?;

                self.expect(TokenType::End)?;

                return Ok(Some(self.ast.replace_node(node, ast::Node::FunctionLiteral {
                    inputparams,
                    outputparams,
                    body,
                })));
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
                }
                else {
                    return Err(self.log_error(error::Error::at_span(
                        errors::ExpectedExpression, // TODO: <- should probably be bepoke error
                        self.last_token.as_ref().unwrap().source_span,
                        "Expected argument".into(),
                    ))?);
                }
            }
        }

        return Ok(self.ast.replace_node(node, ast::Node::ArgumentList {
            args: args,
        }));
    }

    fn parse_expression(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if self.accept(TokenType::StringLiteral) {
            let text = self.get_last_token_text();
            return Ok(Some(self.ast.add_node(ast::Node::StringLiteral {
                text: text,
            })));
        }
        else if self.accept(TokenType::NumericLiteral) {
            let text = self.get_last_token_text();
            // TODO: Support for other numericals
            return Ok(Some(self.ast.add_node(ast::Node::IntegerLiteral {
                value: text.parse::<u64>().unwrap(),
                signed: false,
            })));
        }
        else if self.accept(TokenType::Identifier) {
            // TODO: Function calls
            let s = self.get_last_token_symbol();

            // Calls
            if self.accept(TokenType::OpeningParenthesis) {
                let node = self.ast.reserve_node();
                let symbol = self.ast.add_node(ast::Node::SymbolReference {
                    symbol: s,
                });

                let arglist = self.parse_argumentlist()?;

                self.expect(TokenType::ClosingParenthesis)?;

                return Ok(Some(self.ast.replace_node(node, ast::Node::CallOperation {
                    expr: symbol,
                    arglist: arglist,
                })));
            }

            // TODO: Hack! Hard-coded plus-expression, replace with shunting yard for all operators
            if self.accept(TokenType::Plus) {
                let node = self.ast.reserve_node();
                let lhs = self.ast.add_node(ast::Node::SymbolReference {
                    symbol: s,
                });

                if let Some(n) = self.parse_expression()? {
                    return Ok(Some(self.ast.replace_node(node, ast::Node::BinaryOperation {
                        lhs: lhs,
                        rhs: n,
                    })));
                }
                else {
                    return Err(self.log_error(error::Error::at_span(
                        errors::ExpectedExpression,
                        // TODO: Should be current token, would work with EOS tokens
                        self.last_token.as_ref().unwrap().source_span,
                        "Expected right hand side expression".into(),
                    ))?);
                }
            }
            else {
                return Ok(Some(self.ast.add_node(ast::Node::SymbolReference {
                    symbol: s,
                })));
            }
        }
        else if let Some(n) = self.parse_function_literal_or_type()? {
            return Ok(Some(n));
        }
        else if self.accept(TokenType::Hash) {
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

            let endpos = self.last_token.as_ref().unwrap().source_span.pos + 
                self.last_token.as_ref().unwrap().source_span.len as u64;

            symbolstrings.reverse();

            // TODO: simplify
            if symbolstrings.last().filter(|s| *s == "primitives").is_some() {
                symbolstrings.pop();
                if symbolstrings.last().filter(|s| *s == "u32").is_some() {
                    symbolstrings.pop();
                    return Ok(Some(self.ast.add_node(ast::Node::BuiltInObjectReference {
                        object: ast::BuiltInObject::PrimitiveType(ast::BuiltInPrimitiveType::U32)
                    })));
                }
            }
            else if symbolstrings.last().filter(|s| *s == "output").is_some() {
                symbolstrings.pop();
                if symbolstrings.last().filter(|s| *s == "print_format").is_some() {
                    symbolstrings.pop();
                    let node = self.ast.reserve_node();

                    let builtinfunc = self.ast.add_node(ast::Node::BuiltInObjectReference {
                        object: ast::BuiltInObject::Function(ast::BuiltInFunction::PrintFormat)
                    });

                    self.expect(TokenType::OpeningParenthesis)?;

                    let arglist = self.parse_argumentlist()?;

                    self.expect(TokenType::ClosingParenthesis)?;

                    return Ok(Some(self.ast.replace_node(node, ast::Node::CallOperation {
                        expr: builtinfunc,
                        arglist: arglist,
                    })));
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

    fn parse_symbol_declaration(&mut self) -> Result<Option<ast::NodeRef>, error::ErrorId> {
        if self.accept(TokenType::Def) {
            let node = self.ast.reserve_node();

            self.expect(TokenType::Identifier)?;

            // TODO: Optional type specifier

            // Defines must be initalized to a value
            self.expect(TokenType::Equals)?;

            if let Some(n) = self.parse_expression()? {
                let s = self.get_last_token_symbol();

                return Ok(Some(self.ast.replace_node(node, ast::Node::SymbolDeclaration {
                    symbol: s,
                    typeexpr: None,
                    initexpr: Some(n),
                })));
            }
            else {
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
        }
        else if let Some(n) = self.parse_expression()? {
            return Ok(Some(n));
        }
        return Ok(None);
    }

    fn parse_statementbody(&mut self) -> Result<ast::NodeRef, error::ErrorId> {
        let node = self.ast.reserve_node();
        
        let mut statements : Vec<ast::NodeRef> = Vec::new();
        
        while self.current_token.is_some() {
            // TODO: For now, just eat all linebreaks and indentation between statements
            while self.accept(TokenType::LineBreak) || self.accept(TokenType::Indentation) {
            }

            match self.parse_statement() {
                Err(error::ErrorId::FatalError(errors::ErrorLimitExceeded)) =>
                    return Err(error::ErrorId::FatalError(errors::ErrorLimitExceeded)),
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
                },
                Ok(Some(n)) => {
                    statements.push(n);
                },
                _ => break,
            }
        }

        return Ok(self.ast.replace_node(node, ast::Node::StatementBody {
            statements: statements
        }));
    }

    // Parse fragment (usually file)
    fn parse_fragment(&mut self) -> Result<(), error::ErrorId>{
        let body = self.parse_statementbody()?;

        self.ast.set_root(body);

        return Ok(());
    }

    pub fn parse(&mut self) {
        self.advance();

        match self.parse_fragment() {
            Err(error::ErrorId::FatalError(errors::ErrorLimitExceeded)) => {
                println!("Parsing stopped, error limit exceeed");
                return;
            },
            Err(e) => {
                panic!("Unhandled error! {:?}", e);
            },
            _ => ()
        }

        if self.tokens.read_token() == None {
            println!("Parsed all {} tokens successfully!", self.temp_tokencount);
        }
        else {
            println!("Only parsed {} tokens..." , self.temp_tokencount);
        }

        println!("AST:");
        self.ast.print(4);
    }
}
