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

    fn advance(&mut self) {
        self.last_token = std::mem::replace(&mut self.current_token, self.tokens.read_token());
        self.temp_tokencount += 1;
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

    fn expect(&mut self, t : TokenType) {
        if !self.accept(t) {
            if let Some(t) = &self.current_token {
                self.errors.log_error(error::Error::at_span(
                    errors::UnexpectedToken,
                    t.source_span,
                    "Unexpected token!".into(),
                ));
            }
            else {
                self.errors.log_error(error::Error::at_span(
                    errors::UnexpectedEOF,
                    // TODO: This is bad, but can be fixed by introducing EOS token
                    //  pointing to end of file
                    self.last_token.as_ref().unwrap().source_span,
                    "Unexpected termination token!".into(),
                ));
            }
            // TODO: Add error recovery. Should we just bubble up errors to statement body, or nearest block/brackets?
            //panic!("Unexpected token!");
        }
    }

    fn get_last_token_text(&mut self) -> String {
        return self.tokens.get_token_string(self.last_token.as_ref().unwrap());
    }

    fn get_last_token_symbol(&mut self) -> ast::SymbolRef {
        let text = self.get_last_token_text();
        return self.ast.add_symbol(&*text);
    }

    fn parse_input_parameter(&mut self) -> Option<ast::NodeRef> {
        if self.accept(TokenType::Identifier) {
            let node = self.ast.reserve_node();
            let symbol = self.get_last_token_symbol();

            self.expect(TokenType::Colon);

            if let Some(n) = self.parse_expression() {
                return Some(self.ast.replace_node(node, ast::Node::InputParameter {
                    symbol,
                    typeexpr: n,
                }));
            }
            else {
                self.errors.log_error(error::Error::at_span(
                    errors::ExpectedExpression,
                    self.last_token.as_ref().unwrap().source_span,
                    "Expected type expression for input parameter".into(),
                ));
            }
        }

        return None;
    }

    fn parse_output_parameter(&mut self) -> Option<ast::NodeRef> {
        let node = self.ast.reserve_node();

        if let Some(n) = self.parse_expression() {
            return Some(self.ast.replace_node(node, ast::Node::OutputParameter {
                typeexpr: n,
            }));
        }

        self.ast.undo_node_reservation(node);
        return None;
    }

    fn parse_function_literal_or_type(&mut self) -> Option<ast::NodeRef> {
        if self.accept(TokenType::Func) {
            let node = self.ast.reserve_node();
            let mut inputparams = Vec::new();
            let mut outputparams = Vec::new();

            // Optional input parameters
            if self.accept(TokenType::OpeningParenthesis) {
                if let Some(n) = self.parse_input_parameter() {
                    inputparams.push(n);

                    while self.accept(TokenType::Comma) {
                        if let Some(n) = self.parse_input_parameter() {
                            inputparams.push(n);
                        }
                        else {
                            self.errors.log_error(error::Error::at_span(
                                errors::ExpectedInputParameterDeclaration,
                                self.last_token.as_ref().unwrap().source_span,
                                "Expected input parameter declaration".into(),
                            ));
                            // TODO: Recovery?
                        }
                    }
                }

                self.expect(TokenType::ClosingParenthesis);
            }

            // Optional output paramters
            if self.accept(TokenType::Arrow) {
                if self.accept(TokenType::OpeningParenthesis) {
                    if let Some(n) = self.parse_output_parameter() {
                        outputparams.push(n);

                        while self.accept(TokenType::Comma) {
                            if let Some(n) = self.parse_output_parameter() {
                                outputparams.push(n);
                            }
                            else {
                                self.errors.log_error(error::Error::at_span(
                                    errors::ExpectedOutputParameterDeclaration,
                                    self.last_token.as_ref().unwrap().source_span,
                                    "Expected output parameter declaration".into(),
                                ));
                                // TODO: Recovery?
                            }
                        }
                    }
    
                    self.expect(TokenType::ClosingParenthesis);
                }
                else if let Some(n) = self.parse_output_parameter() {
                    outputparams.push(n);
                }
                else {
                    // No recovery necesary here
                    self.errors.log_error(error::Error::at_span(
                        errors::ExpectedOutputParameterDeclaration,
                        self.last_token.as_ref().unwrap().source_span,
                        "Expected output parameter declaration".into(),
                    ));
                }
            }

            // If there is a body following, we are dealing with a function literal
            //  otherwise, a type literal
            if self.accept(TokenType::Do) {
                // TODO: LB and Indent should probably not be hard requirements
                self.expect(TokenType::LineBreak); self.expect(TokenType::Indentation);

                let body = self.parse_statementbody();

                self.expect(TokenType::End);

                return Some(self.ast.replace_node(node, ast::Node::FunctionLiteral {
                    inputparams,
                    outputparams,
                    body,
                }));
            }
        }

        return None;
    }

    fn parse_argumentlist(&mut self) -> ast::NodeRef {
        let node = self.ast.reserve_node();
        let mut args = Vec::new();

        // TODO: Add new node for argument expression
        if let Some(n) = self.parse_expression() {
            args.push(n);

            while self.accept(TokenType::Comma) {
                if let Some(n) = self.parse_expression() {
                    args.push(n);
                }
                else {
                    self.errors.log_error(error::Error::at_span(
                        errors::ExpectedExpression, // TODO: <- should probably be bepoke error
                        self.last_token.as_ref().unwrap().source_span,
                        "Expected argument".into(),
                    ));
                    // TODO: Recovery?
                }
            }
        }

        return self.ast.replace_node(node, ast::Node::ArgumentList {
            args: args,
        });
    }

    fn parse_expression(&mut self) -> Option<ast::NodeRef> {
        if self.accept(TokenType::StringLiteral) {
            let text = self.get_last_token_text();
            return Some(self.ast.add_node(ast::Node::StringLiteral {
                text: text,
            }));
        }
        else if self.accept(TokenType::NumericLiteral) {
            let text = self.get_last_token_text();
            // TODO: Support for other numericals
            return Some(self.ast.add_node(ast::Node::IntegerLiteral {
                value: text.parse::<u64>().unwrap(),
                signed: false,
            }));
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

                let arglist = self.parse_argumentlist();

                self.expect(TokenType::ClosingParenthesis);

                return Some(self.ast.replace_node(node, ast::Node::CallOperation {
                    expr: symbol,
                    arglist: arglist,
                }));
            }

            // TODO: Hack! Hard-coded plus-expression, replace with shunting yard for all operators
            if self.accept(TokenType::Plus) {
                let node = self.ast.reserve_node();
                let lhs = self.ast.add_node(ast::Node::SymbolReference {
                    symbol: s,
                });

                if let Some(n) = self.parse_expression() {
                    return Some(self.ast.replace_node(node, ast::Node::BinaryOperation {
                        lhs: lhs,
                        rhs: n,
                    }));
                }
                else {
                    self.errors.log_error(error::Error::at_span(
                        errors::ExpectedExpression,
                        // TODO: Should be current token, would work with EOS tokens
                        self.last_token.as_ref().unwrap().source_span,
                        "Expected right hand side expression".into(),
                    ));
                }
            }
            else {
                return Some(self.ast.add_node(ast::Node::SymbolReference {
                    symbol: s,
                }));
            }
        }
        else if let Some(n) = self.parse_function_literal_or_type() {
            return Some(n);
        }
        else if self.accept(TokenType::Hash) {
            let mut symbolstrings = Vec::new();

            let startpos = self.last_token.as_ref().unwrap().source_span.pos;

            // TODO: What to do with whitespace between # and identifier?
            self.expect(TokenType::Identifier);
            symbolstrings.push(self.get_last_token_text());

            // Eat dot-notated symbol expression
            while self.accept(TokenType::Dot) {
                self.expect(TokenType::Identifier);
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
                    return Some(self.ast.add_node(ast::Node::BuiltInObjectReference {
                        object: ast::BuiltInObject::PrimitiveType(ast::BuiltInPrimitiveType::U32)
                    }));
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

                    self.expect(TokenType::OpeningParenthesis);

                    let arglist = self.parse_argumentlist();

                    self.expect(TokenType::ClosingParenthesis);

                    return Some(self.ast.replace_node(node, ast::Node::CallOperation {
                        expr: builtinfunc,
                        arglist: arglist,
                    }));
                }
            }

            self.errors.log_error(error::Error::at_span(
                errors::UnknownCompilerDirective,
                SourceSpan {
                    pos: startpos,
                    len: (endpos - startpos) as usize,
                },
                "Unknown compile directive".into(),
            ));
            // TODO: Error recovery
        }
        return None;
    }

    fn parse_symbol_declaration(&mut self) -> Option<ast::NodeRef> {
        if self.accept(TokenType::Def) {
            let node = self.ast.reserve_node();

            self.expect(TokenType::Identifier);

            // TODO: Optional type specifier

            // Defines must be initalized to a value
            self.expect(TokenType::Equals);

            if let Some(n) = self.parse_expression() {
                let s = self.get_last_token_symbol();

                return Some(self.ast.replace_node(node, ast::Node::SymbolDeclaration {
                    symbol: s,
                    typeexpr: None,
                    initexpr: Some(n),
                }));
            }
            else {
                // TODO: error
                return None;
            }
        }

        return None;
    }

    fn parse_statementbody(&mut self) -> ast::NodeRef {
        let node = self.ast.reserve_node();
        
        let mut statements : Vec<ast::NodeRef> = Vec::new();
        
        while self.current_token.is_some() && !self.errors.reached_error_limit() {
            // TODO: For now, just eat all linebreaks and indentation between statements
            while self.accept(TokenType::LineBreak) || self.accept(TokenType::Indentation) {
            }

            if let Some(n) = self.parse_symbol_declaration() {
                statements.push(n);
                continue;
            }
            else if let Some(n) = self.parse_expression() {
                statements.push(n);
                continue;
            }

            break;
        }
   
        return self.ast.replace_node(node, ast::Node::StatementBody {
            statements: statements
        });
    }

    // Parse fragment (usually file)
    fn parse_fragment(&mut self) {
        let body = self.parse_statementbody();

        // If there are still unprocessed tokens, log error
        // TODO: This can be done within the statement body using indentation,
        //  it should really never happen here
        if let Some(t) = &self.current_token {
            self.errors.log_error(error::Error::at_span(
                errors::UnexpectedToken,
                t.source_span,
                "Unexpected token".into(),
            ));

            // Error recovery, eat everything until next new line
            // TODO: Use indentation to skip breaks that mean line continuations
            while let Some(t) = self.tokens.read_token() {
                if t.tokentype == TokenType::LineBreak {
                    self.current_token = Some(t);
                    break;
                }
            }
        }

        self.ast.set_root(body);
    }

    pub fn parse(&mut self) {
        self.advance();

        self.parse_fragment();

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
