mod builtins;
mod expressions;
mod statements;

use std::collections::HashMap;

use crate::asg;
use crate::asg::StatementBody;
use crate::ast;
use crate::error;
use crate::passes;

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
    // TODO: This sucks, the goal is to give literals decent names
    current_symdecl_name: String,
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

#[derive(Debug)]
pub struct GrapherResult {
    pub asg: asg::Asg,
    pub errors: Vec<error::Error>,
}

impl State {
    pub fn get_module(&self, key: &asg::ModuleKey) -> &asg::Module {
        self.asg.store.modules.get(key)
    }

    pub fn get_module_mut(&mut self, key: &asg::ModuleKey) -> &mut asg::Module {
        self.asg.store.modules.get_mut(key)
    }

    pub fn get_current_module(&self) -> &asg::Module {
        return self.get_module(&self.current_module);
    }

    pub fn get_current_module_mut(&mut self) -> &mut asg::Module {
        let key = self.current_module.clone();
        return self.get_module_mut(&key);
    }

    pub fn get_current_symbolscope(&mut self) -> &mut asg::SymbolScope {
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
                current_symdecl_name: "".into(),
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
        self.state
            .asg
            .store
            .functions
            .get_mut(&self.state.asg.main)
            .body = self.parse_statement_body(
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

        // Module literal expression
        let init_exprkey = self
            .state
            .asg
            .store
            .expressions
            .add(asg::Expression::Literal(
                asg::expressions::Literal::ModuleLiteral(
                    asg::expressions::literals::ModuleLiteral { modulekey },
                ),
            ));

        let symbolscope = self.state.get_current_symbolscope();

        // Local symbol declaration
        let symbolkey = symbolscope.declarations.add(asg::SymbolDeclaration::new(
            ast.get_symbol(&ast_module.symbol).unwrap().into(),
            None,
        ));

        // Add to scope definitions
        symbolscope.definitions.insert(symbolkey, init_exprkey);

        let old_module = self.state.current_module.clone();
        self.state.current_module = modulekey.clone();

        // Statementbody
        self.state
            .asg
            .store
            .modules
            .get_mut(&self.state.current_module)
            .initalizer = self.parse_statement_body(
            astkey,
            ast::as_node!(ast, StatementBody, &ast_module.statementbody),
        );

        self.state.current_module = old_module;
    }

    fn parse_statement_body(
        &mut self,
        astkey: ast::AstKey,
        ast_body: &ast::nodes::StatementBody,
    ) -> Option<asg::StatementBodyKey> {
        let mut body = StatementBody::new();

        for s in &ast_body.statements {
            if let Some(s) = self.parse_statement(astkey, s) {
                body.statements.push(s);
            };
        }

        if body.statements.is_empty() {
            return None;
        }

        Some(self.state.asg.store.statementbodies.add(body))
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

    let asg = passes::resolve_symbols(asg);

    return GrapherResult {
        asg: asg,
        errors: errors,
    };
}
