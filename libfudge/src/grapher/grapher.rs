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
    global_module: asg::ModuleKey,
    main: Option<asg::FunctionKey>,
    modulestore: asg::ModuleStore,
    module_stack: Vec<asg::ModuleKey>,
    scope_stack: Vec<asg::ScopeKey>,
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
        self.modulestore.get(key)
    }

    pub fn get_module_mut(&mut self, key: asg::ModuleKey) -> &mut asg::Module {
        self.modulestore.get_mut(&key)
    }

    pub fn get_current_module_key(&self) -> asg::ModuleKey {
        *self.module_stack.last().unwrap()
    }

    pub fn get_current_scope_key(&self) -> asg::ScopeKey {
        *self.scope_stack.last().unwrap()
    }

    pub fn get_current_module(&self) -> &asg::Module {
        return self.get_module(&self.module_stack.last().unwrap());
    }

    pub fn get_current_module_mut(&mut self) -> &mut asg::Module {
        return self.get_module_mut(self.get_current_module_key());
    }

    pub fn get_current_scope(&mut self) -> &mut asg::scope::Scope {
        let scope = self.get_current_scope_key();
        self.get_current_module_mut().scopestore.get_mut(&scope)
    }

    pub fn create_scope(&mut self) -> asg::ScopeKey {
        let scope = asg::scope::Scope::new(Some(asg::ScopeRef::new(
            self.get_current_module_key(),
            self.get_current_scope_key(),
        )));
        self.get_current_module_mut().scopestore.add(scope)
    }

    pub fn edit_scope(&mut self, scope: &asg::ScopeKey) -> &mut asg::scope::Scope {
        self.get_current_module_mut().scopestore.get_mut(scope)
    }

    pub fn push_module(&mut self, module: &asg::ModuleKey) {
        self.module_stack.push(module.clone());
        let scope = self.get_current_module().scope;
        self.push_scope(&scope);
    }

    pub fn pop_module(&mut self) {
        self.pop_scope();
        self.module_stack.pop();
    }

    pub fn push_scope(&mut self, scope: &asg::ScopeKey) {
        self.scope_stack.push(scope.clone());
    }

    pub fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }
}

impl<'a> Grapher<'a> {
    pub fn new(context: &'a Context) -> Self {
        let global_module = asg::Module::new("global".into(), None);

        let mut modulestore = asg::ModuleStore::new();
        let global_module = modulestore.add(global_module);

        let current_module = global_module;
        let current_scope = modulestore.get(&current_module).scope;

        Self {
            context: context,
            state: State {
                global_module,
                main: None,
                modulestore,
                module_stack: [current_module].into(),
                scope_stack: [current_scope].into(),
                current_symdecl_name: "".into(),
            },
            errors: error::ErrorManager::new(),
        }
    }

    pub fn create_asg(mut self) -> (asg::Asg, Vec<error::Error>) {
        for ast in self.context.asts.keys() {
            self.parse_ast(*ast);
        }

        let asg = asg::Asg {
            global_module: self.state.global_module,
            main: self.state.main.unwrap(),
            modulestore: self.state.modulestore,
        };

        (asg, self.errors.error_data.errors)
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
        let ast = self.context.get_ast(astkey);

        // Main scope is _not_ global scope, other globals cannot access stuff in here
        let mainscope = self.state.create_scope();
        self.state.push_scope(&mainscope);

        let body = self.parse_statement_body(
            astkey,
            ast::as_node!(ast, StatementBody, &ast_entrypoint.statementbody),
        );

        self.state.pop_scope();

        // Note: main should not be available for symbol lookup, so don't add it to any scope
        let function = asg::Function::new("main".into(), mainscope, Vec::new(), body);

        let functionkey = self
            .state
            .get_module_mut(self.state.global_module)
            .functionstore
            .add(function);

        self.state.main = Some(functionkey);
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
            name,
            Some(asg::ScopeRef::new(
                self.state.get_current_module_key(),
                self.state.get_current_scope_key(),
            )),
        );
        let modulekey = self.state.modulestore.add(module);

        {
            self.state.push_module(&modulekey);

            // Statementbody for initializer
            let body = self.parse_statement_body(
                astkey,
                ast::as_node!(ast, StatementBody, &ast_module.statementbody),
            );

            self.state.get_current_module_mut().body = body;

            self.state.pop_module();
        }

        let parentscope = self.state.get_current_scope();

        // Module literal expression
        let init_exprkey = parentscope.expressions.add(asg::Expression::new(
            asg::ExpressionObject::Literal(asg::expressions::Literal::ModuleLiteral(
                asg::expressions::literals::ModuleLiteral { modulekey },
            )),
            123,
        ));

        // Local symbol declaration
        let symbolkey =
            parentscope
                .symboltable
                .declarations
                .add(asg::symboltable::SymbolDeclaration::new(
                    ast.get_symbol(&ast_module.symbol).unwrap().into(),
                    None,
                ));

        // Add to parent scope definitions
        parentscope
            .symboltable
            .definitions
            .insert(symbolkey, init_exprkey);
    }

    fn parse_statement_body(
        &mut self,
        astkey: ast::AstKey,
        ast_body: &ast::nodes::StatementBody,
    ) -> Option<asg::StatementBody> {
        let mut body = StatementBody::new(self.state.get_current_scope_key());

        for s in &ast_body.statements {
            if let Some(s) = self.parse_statement(astkey, s) {
                body.statements.push(s);
            };
        }

        if body.statements.is_empty() {
            return None;
        }

        Some(body)
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

    let asg = passes::process_asg(asg);

    return GrapherResult {
        asg: asg,
        errors: errors,
    };
}
