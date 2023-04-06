use std::collections::HashMap;

use crate::asg;
use crate::ast;
use crate::error;
use crate::typesystem;

// To be able to call methods on "Stores"... :(
use crate::utils::objectstore::ObjectStore;

struct Grapher<'a> {
    context: &'a Context<'a>,
    state: State,
    errors: error::ErrorManager,
}

struct Context<'a> {
    pub asts: HashMap<ast::AstKey, &'a ast::Ast>,
}

struct State {
    asg: asg::Asg,
    current_module: asg::ModuleKey,
    current_function: Option<asg::FunctionKey>,
}

impl<'a> Context<'a> {
    pub fn new() -> Context<'a> {
        Context {
            asts: HashMap::new(),
        }
    }

    pub fn get_ast(&self, key: ast::AstKey) -> &'a ast::Ast {
        self.asts.get(&key).unwrap()
    }
}

pub struct GrapherResult {
    pub asg: asg::Asg,
    pub errors: Vec<error::Error>,
}

impl State {
    fn get_module(&self, key: &asg::ModuleKey) -> &asg::Module {
        self.asg.store.modules.get(key)
    }

    fn get_module_mut(&mut self, key: &asg::ModuleKey) -> &mut asg::Module {
        self.asg.store.modules.get_mut(key)
    }

    fn get_current_module(&self) -> &asg::Module {
        return self.get_module(&self.current_module);
    }

    fn get_current_module_mut(&mut self) -> &mut asg::Module {
        let key = self.current_module.clone();
        return self.get_module_mut(&key);
    }

    fn get_current_statementbody(&mut self) -> &mut asg::StatementBody {
        if let Some(function) = self.current_function {
            &mut self.asg.store.functions.get_mut(&function).body
        } else {
            &mut self
                .asg
                .store
                .modules
                .get_mut(&self.current_module)
                .initalizer
        }
    }

    fn get_current_symbolscope(&mut self) -> &mut asg::SymbolScope {
        let scope = self.get_current_module_mut().symbolscope.clone();
        self.asg.store.symbolscopes.get_mut(&scope)
    }
}

impl<'a> Grapher<'a> {
    pub fn new(context: &'a Context) -> Self {
        let asg = asg::Asg::new();
        let current_module = asg.global_module;
        let current_function = None;
        Self {
            context: context,
            state: State {
                asg,
                current_module,
                current_function,
            },
            errors: error::ErrorManager::new(),
        }
    }

    pub fn create_asg(mut self) -> (asg::Asg, Vec<error::Error>) {
        for ast in self.context.asts.keys() {
            self.parse_ast(*ast);
        }

        (self.state.asg, self.errors.error_data.errors)
    }

    fn parse_ast(&mut self, astkey: ast::AstKey) {
        let ast = self.context.get_ast(astkey);
        if let Some(root) = ast.get_root() {
            match ast.get_node(&root) {
                ast::Node::Module(m) => self.parse_module(astkey, m),
                ast::Node::EntryPoint(e) => self.parse_entrypoint(astkey, e),
                // TODO: This can be done better in the ast
                _ => panic!("Invalid ast root node!"),
            }
        }
    }

    fn parse_entrypoint(&mut self, astkey: ast::AstKey, ast_entrypoint: &ast::nodes::EntryPoint) {
        self.state.current_function = Some(self.state.asg.main);

        let ast = self.context.get_ast(astkey);
        self.parse_function_statement_body(
            astkey,
            ast::as_node!(ast, StatementBody, &ast_entrypoint.statementbody),
        );

        self.state.current_function = None;
    }

    fn parse_module(&mut self, astkey: ast::AstKey, ast_module: &ast::nodes::Module) {
        let ast = self.context.get_ast(astkey);

        // Module name is dot-delimited module path
        let name = format!(
            "{}.{}",
            self.state.get_current_module().name,
            ast.get_symbol(&ast_module.symbol).unwrap()
        );

        // Create module
        let module = asg::Module::new(
            &mut self.state.asg.store,
            name,
            Some(self.state.current_module.clone()),
        );
        let modulekey = self.state.asg.store.modules.add(module);

        // Initializing expression
        let init_exprkey = self
            .state
            .asg
            .store
            .expressions
            .add(asg::Expression::Literal(
                asg::expressions::Literal::ModuleLiteral(
                    asg::expressions::literals::ModuleLiteral { key: modulekey },
                ),
            ));

        // Local symbol declaration
        self.state
            .get_current_symbolscope()
            .declarations
            .add(asg::SymbolDeclaration::new(
                ast.get_symbol(&ast_module.symbol).unwrap().into(),
                None,
                Some(init_exprkey),
            ));

        let old_module = self.state.current_module.clone();
        self.state.current_module = modulekey.clone();

        self.parse_module_statement_body(
            astkey,
            ast::as_node!(ast, StatementBody, &ast_module.statementbody),
        );

        self.state.current_module = old_module;
    }

    fn parse_module_statement_body(
        &mut self,
        astkey: ast::AstKey,
        ast_body: &ast::nodes::StatementBody,
    ) {
        for s in &ast_body.statements {
            self.parse_statement(astkey, s);
        }
    }

    fn parse_function_statement_body(
        &mut self,
        astkey: ast::AstKey,
        ast_body: &ast::nodes::StatementBody,
    ) {
        for s in &ast_body.statements {
            self.parse_statement(astkey, s);
        }
    }

    fn parse_statement(&mut self, astkey: ast::AstKey, node: &ast::NodeRef) {
        return match self.context.get_ast(astkey).get_node(node) {
            ast::Node::ModuleSelfDeclaration(_) => {
                /* TODO: This should be pruned before any intepretation step */
            }
            ast::Node::Module(n) => self.parse_module(astkey, n),
            ast::Node::StatementBody(_n) => todo!(), // TODO: Can this happen?
            ast::Node::SymbolDeclaration(n) => self.parse_symbol_declaration(astkey, n),
            ast::Node::IfStatement(n) => todo!(),
            ast::Node::ReturnStatement(n) => todo!(),
            ast::Node::AssignStatement(n) => todo!(),
            n => {
                panic!("{:?} is not a valid statement", n);
            }
        };
    }

    fn parse_symbol_declaration(
        &mut self,
        astkey: ast::AstKey,
        ast_symdecl: &ast::nodes::SymbolDeclaration,
    ) {
        let ast = self.context.get_ast(astkey);

        let type_expr = if let Some(e) = ast_symdecl.typeexpr {
            Some(self.parse_expression(astkey, &e))
        } else {
            None
        };

        let init_expr = if let Some(e) = ast_symdecl.initexpr {
            Some(self.parse_expression(astkey, &e))
        } else {
            None
        };

        self.state
            .get_current_symbolscope()
            .declarations
            .add(asg::SymbolDeclaration::new(
                ast.get_symbol(&ast_symdecl.symbol).unwrap().into(),
                type_expr,
                init_expr,
            ));
    }

    fn parse_expression(&mut self, astkey: ast::AstKey, node: &ast::NodeRef) -> asg::ExpressionKey {
        match self.context.get_ast(astkey).get_node(node) {
            ast::Node::StructLiteral(n) => self.parse_struct_literal(astkey, n),
            ast::Node::StringLiteral(n) => todo!(),
            ast::Node::FunctionLiteral(n) => todo!(),
            ast::Node::BuiltInObjectReference(n) => todo!(),
            ast::Node::SymbolReference(n) => todo!(),
            ast::Node::IfExpression(n) => todo!(),
            ast::Node::CallOperation(n) => todo!(),
            ast::Node::BinaryOperation(n) => todo!(),
            ast::Node::SubScript(n) => todo!(),
            n => {
                panic!("{:?} is not a valid expression!", n);
            }
        }
    }

    fn parse_struct_literal(
        &mut self,
        astkey: ast::AstKey,
        ast_lit: &ast::nodes::StructLiteral,
    ) -> asg::ExpressionKey {
        let mut fields = Vec::new();

        let ast = self.context.get_ast(astkey);
        for f in &ast_lit.fields {
            let sf = ast::as_node!(ast, StructField, &f);
            fields.push(asg::misc::StructField {
                name: ast.get_symbol(&sf.symbol).unwrap().clone(),
                typeexpr: self.parse_expression(astkey, &sf.typeexpr),
            });
        }

        let literal = asg::expressions::literals::StructLiteral { fields };

        self.state
            .asg
            .store
            .expressions
            .add(asg::Expression::Literal(
                asg::expressions::Literal::StructLiteral(literal),
            ))
    }
}

pub fn create_graph<'a>(main_ast: &'a ast::Ast, module_asts: &'a Vec<ast::Ast>) -> GrapherResult {
    let mut context = Context::new();

    context.asts.insert(main_ast.key, main_ast);
    for module_ast in module_asts {
        context.asts.insert(module_ast.key, module_ast);
    }

    let grapher = Grapher::new(&context);

    let (asg, errors) = grapher.create_asg();
    return GrapherResult {
        asg: asg,
        errors: errors,
    };
}
