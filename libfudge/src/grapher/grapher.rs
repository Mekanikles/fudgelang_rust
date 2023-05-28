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
    current_scope: asg::ScopeKey,
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
        self.asg.modulestore.get(key)
    }

    pub fn get_module_mut(&mut self, key: asg::ModuleKey) -> &mut asg::Module {
        self.asg.modulestore.get_mut(&key)
    }

    pub fn get_current_module(&self) -> &asg::Module {
        return self.get_module(&self.current_module);
    }

    pub fn get_current_module_mut(&mut self) -> &mut asg::Module {
        let key = self.current_module.clone();
        return self.get_module_mut(key);
    }

    pub fn get_current_scope(&mut self) -> &mut asg::scope::Scope {
        let scope = self.current_scope;
        self.get_current_module_mut().scopestore.get_mut(&scope)
    }

    pub fn create_scope(&mut self) -> asg::ScopeKey {
        let scope = asg::scope::Scope::new(Some(asg::ScopeRef::new(
            self.current_module.clone(),
            self.current_scope.clone(),
        )));
        self.get_current_module_mut().scopestore.add(scope)
    }

    pub fn edit_scope(&mut self, scope: &asg::ScopeKey) -> &mut asg::scope::Scope {
        self.get_current_module_mut().scopestore.get_mut(scope)
    }
}

impl<'a> Grapher<'a> {
    pub fn new(context: &'a Context) -> Self {
        let asg = asg::Asg::new();
        let current_module = asg.global_module;
        let current_scope = asg.modulestore.get(&current_module).scope;

        Self {
            context: context,
            state: State {
                asg,
                current_module,
                current_scope,
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
        let ast = self.context.get_ast(astkey);

        let mainscope = self
            .state
            .get_module(&self.state.asg.global_module)
            .functionstore
            .get(&self.state.asg.main)
            .scope;

        let body = self.parse_statement_body(
            astkey,
            ast::as_node!(ast, StatementBody, &ast_entrypoint.statementbody),
            mainscope,
        );

        self.state
            .get_module_mut(self.state.asg.global_module)
            .scopestore
            .get_mut(&mainscope)
            .statementbody = body;
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
                self.state.current_module.clone(),
                self.state.current_scope.clone(),
            )),
        );
        let modulekey = self.state.asg.modulestore.add(module);

        {
            let old_module = self.state.current_module.clone();
            self.state.current_module = modulekey.clone();

            let scope = self.state.get_current_module().scope;

            // Statementbody
            let body = self.parse_statement_body(
                astkey,
                ast::as_node!(ast, StatementBody, &ast_module.statementbody),
                scope,
            );

            // Add to new module scope
            self.state
                .get_current_module_mut()
                .scopestore
                .get_mut(&scope)
                .statementbody = body;

            self.state.current_module = old_module;
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
        scope: asg::ScopeKey,
    ) -> Option<asg::StatementBody> {
        let old_scope = self.state.current_scope;
        self.state.current_scope = scope;

        let mut body = StatementBody::new();

        for s in &ast_body.statements {
            if let Some(s) = self.parse_statement(astkey, s) {
                body.statements.push(s);
            };
        }

        self.state.current_scope = old_scope;

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
