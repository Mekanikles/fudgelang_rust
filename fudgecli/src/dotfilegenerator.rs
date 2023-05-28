use libfudgec::asg::{ModuleKey, ScopeKey};
use libfudgec::utils::objectstore::ObjectStore;
use libfudgec::*;

use std::fs::File;
use std::io::{BufWriter, Write};

struct Instance {
    writer: Writer,
    state: State,
}

impl Instance {
    fn new(file: File) -> Self {
        Self {
            writer: Writer::new(file),
            state: State::new(),
        }
    }

    fn with_module<F>(&mut self, module: &ModuleKey, asg: &asg::Asg, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.state.module_stack.push(module.clone());
        self.state
            .scope_stack
            .push(self.state.get_current_module(asg).scope);
        f(self);
        self.state.scope_stack.pop();
        self.state.module_stack.pop();
    }

    fn with_scope<F>(&mut self, scope: &ScopeKey, f: F)
    where
        F: FnOnce(&mut Self),
    {
        self.state.scope_stack.push(scope.clone());
        f(self);
        self.state.scope_stack.pop();
    }
}

struct Writer {
    writer: BufWriter<File>,
    queue: Vec<String>,
    indent: usize,
}

impl Writer {
    pub fn new(file: File) -> Self {
        Self {
            writer: BufWriter::new(file),
            queue: Vec::new(),
            indent: 0,
        }
    }

    fn writeline(&mut self, str: String) {
        for _i in 0..self.indent {
            self.writer.write_all("    ".as_bytes()).unwrap();
        }
        self.writer.write_all(str.as_bytes()).unwrap();
        self.writer.write_all(&[b'\n']).unwrap();
    }

    fn queueline(&mut self, str: String) {
        self.queue.push(str);
    }

    fn flushqueue(&mut self) {
        while let Some(item) = self.queue.pop() {
            self.writeline(item);
        }
    }

    pub fn indent(&mut self) {
        self.indent += 1;
    }

    pub fn unindent(&mut self) {
        self.indent -= 1;
    }
}

struct State {
    module_stack: Vec<ModuleKey>,
    scope_stack: Vec<ScopeKey>,
}

impl<'a> State {
    pub fn new() -> Self {
        Self {
            module_stack: Vec::new(),
            scope_stack: Vec::new(),
        }
    }

    fn get_module(&self, module: &asg::ModuleKey, asg: &'a asg::Asg) -> &'a asg::Module {
        asg.modulestore.get(module)
    }

    fn get_current_module(&self, asg: &'a asg::Asg) -> &'a asg::Module {
        self.get_module(self.module_stack.last().unwrap(), asg)
    }

    fn get_scope_from_ref(
        &self,
        scoperef: &asg::ScopeRef,
        asg: &'a asg::Asg,
    ) -> &'a asg::scope::Scope {
        self.get_module(&scoperef.module, asg)
            .scopestore
            .get(&scoperef.scope)
    }

    fn get_current_scope(&self, asg: &'a asg::Asg) -> &'a asg::scope::Scope {
        self.get_current_module(asg)
            .scopestore
            .get(self.scope_stack.last().unwrap())
    }

    fn get_function(&self, key: &asg::FunctionKey, asg: &'a asg::Asg) -> &'a asg::Function {
        self.get_current_module(asg).functionstore.get(&key)
    }

    fn get_expression(
        &self,
        key: &asg::scope::ExpressionKey,
        asg: &'a asg::Asg,
    ) -> &'a asg::Expression {
        self.get_current_scope(asg).expressions.get(&key)
    }

    fn get_scope_node_id_from_ref(&self, scoperef: &asg::ScopeRef) -> String {
        format!("m{}sc{}", scoperef.module, scoperef.scope)
    }

    fn get_scope_node_id(&self, key: &asg::ScopeKey) -> String {
        self.get_scope_node_id_from_ref(&asg::ScopeRef::new(
            *self.module_stack.last().unwrap(),
            *key,
        ))
    }

    fn get_symbol_node_id(&self, key: &asg::symboltable::SymbolKey) -> String {
        format!("s{}", key)
    }

    fn get_expression_node_id(&self, key: &asg::scope::ExpressionKey) -> String {
        format!(
            "m{}s{}e{}",
            self.module_stack.last().unwrap(),
            self.scope_stack.last().unwrap(),
            key
        )
    }

    fn get_module_node_id(&self, key: &asg::ModuleKey) -> String {
        format!("m{}", key)
    }

    fn get_function_node_id(&self, key: &asg::FunctionKey) -> String {
        format!("m{}f{}", self.module_stack.last().unwrap(), key)
    }
}

/*
fn get_statementbody_node_id(key: &StatementBodyKey) -> String {
    format!("sb{}", key)
}
*/
fn write_module(instance: &mut Instance, asg: &asg::Asg, key: &asg::ModuleKey) {
    let node_id = instance.state.get_module_node_id(&key);

    instance
        .writer
        .writeline(format!("subgraph cluster_{} {{", node_id));
    instance.writer.indent();

    instance.with_module(key, asg, |instance| {
        let module = instance.state.get_current_module(asg);

        // Label
        instance
            .writer
            .writeline(format!("label=\"Module: {}\"", module.name));

        // Root node
        // TODO: Should this be merged with symbol scope?
        write_simple_node(instance, &node_id, &format!("Root: {}", node_id));

        write_simple_edge(
            instance,
            &node_id,
            &instance.state.get_scope_node_id(&module.scope),
        );

        write_scope(instance, asg, &module.scope);
    });

    instance.writer.unindent();
    instance.writer.writeline(format!("}}"));

    /*
    // Module node
    let label = format!("Module: {}", module.name);
    let shape = format!("box");
    let style = format!("rounded");
    instance.writer.write(node_id).unwrap();
    writer
        .write_all(
            format!(
                " [shape=\"{}\", style=\"{}\", label=\"{}\"]\n",
                shape, style, label
            )
            ,
        )
        .unwrap();

    // Edges
    {
        // Symbolscope
        writer
            .write_all(
                format!(
                    "{} -> {}\n",
                    node_id,
                    get_symbolscope_node_id(&module.symbolscope)
                )
                ,
            )
            .unwrap();

        // Initialiazer body
        if let Some(body) = &module.initalizer {
            writer
                .write_all(
                    format!("{} -> {}\n", node_id, get_statementbody_node_id(&body)),
                )
                .unwrap();
        }
    }
    */
}

fn write_scope(instance: &mut Instance, asg: &asg::Asg, key: &asg::ScopeKey) {
    let node_id = instance.state.get_scope_node_id(&key);

    let mut symbol_labels = String::new();

    instance.with_scope(key, |instance| {
        let scope = instance.state.get_current_module(asg).scopestore.get(&key);

        // Declarations
        let mut it = scope.symboltable.declarations.keys().peekable();
        while let Some(symbol_decl_key) = it.next() {
            let symbol_decl = scope.symboltable.declarations.get(&symbol_decl_key);

            let local_decl_from_id =
                format!("d{}", instance.state.get_symbol_node_id(&symbol_decl_key));

            let label = format!("{}", symbol_decl.symbol);
            symbol_labels.push_str(format!("<{}> {}", local_decl_from_id, label).as_str());
            if it.peek().is_some() {
                symbol_labels.push_str("|");
            }

            let decl_from_id = format!("{}:{}", node_id, local_decl_from_id);

            if let Some(typeexpr) = &symbol_decl.typeexpr {
                let typeexpr_to_id = instance.state.get_expression_node_id(&typeexpr);

                // Type expr edge
                instance.writer.queueline(format!(
                    "{} -> {} [label=\"type\"]",
                    decl_from_id, typeexpr_to_id
                ));
            }

            // Definition Edge
            if let Some(defexpr) = scope.symboltable.definitions.get(&symbol_decl_key) {
                instance.writer.queueline(format!(
                    "{} -> {} [label=\"definition\"]",
                    decl_from_id,
                    instance.state.get_expression_node_id(&defexpr)
                ));
            }
        }

        // Parent Edge
        if let Some(parent) = &scope.parent {
            instance.writer.queueline(format!(
                "{} -> {} [style=dotted constraint=false]",
                node_id,
                instance.state.get_scope_node_id_from_ref(parent)
            ));
        }

        // Expressions
        for expr in scope.expressions.keys() {
            write_expression(instance, asg, &expr);
        }
    });

    let label = format!("{{ Symbols |{{ |{{ {} }}| }} }}", symbol_labels);
    let shape = format!("record");
    let style = format!("");
    instance.writer.writeline(format!(
        "{} [shape=\"{}\", style=\"{}\", label=\"{}\", xlabel=\"{}\"]",
        node_id, shape, style, label, node_id
    ));
}

fn write_functionparameter(
    instance: &mut Instance,
    asg: &asg::Asg,
    function_id: &String,
    index: usize,
    param: &asg::FunctionParameter,
) -> String {
    let node_id = format!("{}p{}", function_id, index);

    // TODO: Should only be able to reference symbols in function scope
    let symdecl = instance
        .state
        .get_scope_from_ref(&param.symref.scope, asg)
        .symboltable
        .declarations
        .get(&param.symref.symbol);

    let local_expr_from_id = "t";
    let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);
    let expr_to_id = instance
        .state
        .get_expression_node_id(&symdecl.typeexpr.unwrap());

    // Edges
    instance
        .writer
        .queueline(format!("{} -> {}", expr_from_id, expr_to_id));

    let label = format!(
        "in param | {} | <{}> type",
        &symdecl.symbol, local_expr_from_id
    );

    let shape = format!("record");
    let style = format!("");
    instance.writer.writeline(format!(
        "{} [shape=\"{}\", style=\"{}\", label=\"{}\"]",
        node_id, shape, style, label
    ));

    node_id
}

fn write_function(instance: &mut Instance, asg: &asg::Asg, key: &asg::FunctionKey) {
    let node_id = instance.state.get_function_node_id(&key);

    instance
        .writer
        .writeline(format!("subgraph cluster_{} {{", node_id));
    instance.writer.indent();

    {
        let function = instance.state.get_function(&key, asg);

        // Label
        instance
            .writer
            .writeline(format!("label=\"Function: {}\"", function.name));

        // Root node
        // TODO: Should this be merged with symbol scope?
        write_simple_node(instance, &node_id, &format!("Root: {}", node_id));

        write_simple_edge(
            instance,
            &node_id,
            &instance.state.get_scope_node_id(&function.scope),
        );

        write_scope(instance, asg, &function.scope);
    }

    instance.writer.unindent();
    instance.writer.writeline(format!("}}"));

    /*let function = instance.state.get_function(&key, asg);

    let node_id = instance.state.get_function_node_id(&key);

    let label = format!("Function: {}", function.name);
    let shape = format!("component");
    let style = format!("");
    instance.writer.write(node_id);
    instance.writer.write(format!(
        " [shape=\"{}\", style=\"{}\", label=\"{}\"]\n",
        shape, style, label
    ));

    // Params
    let mut count = 0;
    for inparam in &function.inparams {
        let paramid = write_functionparameter(instance, &asg, &node_id, count, &inparam);

        instance
            .writer
            .write(format!("{} -> {}\n", node_id, paramid));

        count += 1;
    }

    // Edges
    {
        // Module
        instance.writer
        .write(format!(
            "{} -> {}\n",
            get_module_node_id(&function.module),
            node_id
        ));

        // Body
        if let Some(body) = &function.body {
            writer
                .write_all(format!(
                    "{} -> {}\n",
                    node_id,
                    get_statementbody_node_id(&body)
                ))
                .unwrap();
        }

        // Scope
        writer
            .write_all(format!(
                "{} -> {}\n",
                node_id,
                get_symbolscope_node_id(&function.symbolscope)
            ))
            .unwrap();
    }*/
}

/*
fn write_statement(
    instance: &mut Instance,
    _asg: &asg::Asg,
    body_id: &String,
    index: usize,
    stmnt: &asg::Statement,
) -> String {
    let node_id = format!("{}s{}", body_id, index);
    let label = match stmnt {
        asg::Statement::If(n) => {
            let mut branches = String::new();
            let mut count = 0;

            let mut it = n.branches.iter().peekable();
            while let Some(branch) = it.next() {
                let local_expr_from_id = format!("b{}e0", count);
                let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);
                let expr_to_id = get_expression_node_id(&branch.0);

                let local_stmnt_from_id = format!("b{}s0", count);
                let stmnt_from_id = format!("{}:{}", node_id, local_stmnt_from_id);
                let stmnt_to_id = get_statementbody_node_id(&branch.1);

                branches.push_str(
                    format!("<{}> expr |<{}> then", expr_from_id, stmnt_from_id).as_str(),
                );
                if it.peek().is_some() {
                    branches.push_str(" |");
                }

                // Edges
                writer
                    .write_all(format!("{} -> {}\n", expr_from_id, expr_to_id))
                    .unwrap();
                writer
                    .write_all(format!("{} -> {}\n", stmnt_from_id, stmnt_to_id))
                    .unwrap();

                count += 1;
            }

            // Else
            if let Some(elsebranch) = n.elsebranch {
                let local_stmnt_from_id = format!("b{}s0", count);
                let stmnt_from_id = format!("{}:{}", node_id, local_stmnt_from_id);
                let stmnt_to_id = get_statementbody_node_id(&elsebranch);

                if !n.branches.is_empty() {
                    branches.push_str(" |");
                }
                branches.push_str(format!("else |<{}> stmnt", local_stmnt_from_id).as_str());

                // Edges
                writer
                    .write_all(format!("{} -> {}\n", stmnt_from_id, stmnt_to_id))
                    .unwrap();
            }

            format!("if | {{ {} }}", branches)
        }
        asg::Statement::Return(n) => {
            if let Some(expr) = n.expr {
                let local_expr_from_id = "e0";
                let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);
                let expr_to_id = get_expression_node_id(&expr);

                // Edges
                writer
                    .write_all(format!("{} -> {}\n", expr_from_id, expr_to_id))
                    .unwrap();

                format!("return | <{}> expr", local_expr_from_id)
            } else {
                format!("return")
            }
        }
        asg::Statement::Assign(n) => {
            let local_lhs_from_id = "e0";
            let lhs_from_id = format!("{}:{}", node_id, local_lhs_from_id);
            let local_rhs_from_id = "e1";
            let rhs_from_id = format!("{}:{}", node_id, local_rhs_from_id);
            let lhs_to_id = get_expression_node_id(&n.lhs);
            let rhs_to_id = get_expression_node_id(&n.rhs);

            // Edges
            writer
                .write_all(format!("{} -> {}\n", lhs_from_id, lhs_to_id))
                .unwrap();
            writer
                .write_all(format!("{} -> {}\n", rhs_from_id, rhs_to_id))
                .unwrap();

            format!(
                "assign |<{}> lhs | = |<{}> rhs",
                local_lhs_from_id, local_rhs_from_id
            )
        }
        asg::Statement::Initialize(n) => {
            let local_expr_from_id = "e0";
            let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);
            let expr_to_id = get_expression_node_id(&n.expr);

            // Edges
            writer
                .write_all(format!("{} -> {}\n", expr_from_id, expr_to_id))
                .unwrap();

            format!("initalize | {} | <{}> expr", n.symbol, local_expr_from_id)
        }
        asg::Statement::ExpressionWrapper(n) => {
            let local_expr_from_id = "e0";
            let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);
            let expr_to_id = get_expression_node_id(&n.expr);

            // Edges
            writer
                .write_all(format!("{} -> {}\n", expr_from_id, expr_to_id))
                .unwrap();

            format!("expression wrapper | <{}> expr", local_expr_from_id)
        }
    };

    // Node
    let shape = format!("record");
    let style = format!("");
    instance.writer.write(node_id).unwrap();
    writer
        .write_all(format!(
            " [shape=\"{}\", style=\"{}\", label=\"{}\", group=\"{}\"]\n",
            shape, style, label, body_id
        ))
        .unwrap();

    node_id
}

fn write_statementbody(instance: &mut Instance, asg: &asg::Asg, key: &asg::FunctionKey) {
    let body = asg.store.statementbodies.get(&key);

    let node_id = get_statementbody_node_id(&key);

    // Scope Edge
    writer
        .write_all(format!(
            "{} -> {}\n",
            node_id,
            get_symbolscope_node_id(&body.symbolscope)
        ))
        .unwrap();

    // Statements
    let mut it = body.statements.iter().peekable();
    let mut count = 0;
    while let Some(stmnt) = it.next() {
        let stmnt_id = write_statement(writer, &asg, &node_id, count, &stmnt);

        // Edges
        writer
            .write_all(format!(
                "{} -> {} [label=\"[{}]\"]\n",
                node_id, stmnt_id, count
            ))
            .unwrap();
        count += 1;
    }

    // Node
    let label = format!("Statement Body");
    let shape = format!("box");
    let style = format!("");
    instance.writer.write(node_id).unwrap();
    writer
        .write_all(format!(
            " [shape=\"{}\", style=\"{}\", label=\"{}\"]\n",
            shape, style, label
        ))
        .unwrap();
}
*/
fn escape_string(string: &String) -> String {
    // TODO: This is horribly inefficient
    string
        .replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace("{", "\\{")
        .replace("}", "\\}")
}

fn write_structfield(
    instance: &mut Instance,
    expr_id: &String,
    field: &asg::misc::StructField,
    index: usize,
) -> String {
    let node_id = format!("{}f{}", expr_id, index);

    let local_expr_from_id = format!("t");
    let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);
    let expr_to_id = instance.state.get_expression_node_id(&field.typeexpr);

    // Edges
    instance.writer.queueline(format!(
        "{} -> {} [label=\"type\"]",
        expr_from_id, expr_to_id
    ));

    let label = format!("Field: {} | <{}> type", field.name, local_expr_from_id);

    // Node
    let shape = format!("record");
    let style = format!("");
    instance.writer.writeline(format!(
        "{} [shape=\"{}\", style=\"{}\", label=\"{}\"]",
        node_id, shape, style, label
    ));

    node_id
}

fn write_simple_edge(instance: &mut Instance, from_id: &String, to_id: &String) {
    instance
        .writer
        .queueline(format!("{} -> {}", from_id, to_id));
}

struct NodeConfig {
    shape: String,
    color: String,
    xlabel: String,
    style: String,
}

fn write_node_with_config(
    instance: &mut Instance,
    config: &NodeConfig,
    node_id: &String,
    label: &String,
) {
    instance.writer.writeline(format!(
        "{} [shape=\"{}\", style=\"{}\", label=\"{}\", xlabel=\"{}\", fillcolor=\"{}\"]",
        node_id, config.shape, config.style, label, config.xlabel, config.color
    ));
}

fn write_simple_node(instance: &mut Instance, node_id: &String, label: &String) {
    let config = NodeConfig {
        shape: format!("Mrecord"),
        color: "".to_string(),
        style: "".to_string(),
        xlabel: "".to_string(),
    };

    write_node_with_config(instance, &config, node_id, label);
}

fn write_colored_expression_node(
    instance: &mut Instance,
    node_id: &String,
    label: &String,
    color: &String,
) {
    let config = NodeConfig {
        shape: format!("Mrecord"),
        color: color.clone(),
        style: if color.is_empty() {
            "".to_string()
        } else {
            "filled".to_string()
        },
        xlabel: format!("Expr: {}", node_id),
    };

    write_node_with_config(instance, &config, node_id, label);
}

fn write_expression_node(instance: &mut Instance, node_id: &String, label: &String) {
    write_colored_expression_node(instance, node_id, label, &"".to_string());
}

fn write_expression(instance: &mut Instance, asg: &asg::Asg, key: &asg::scope::ExpressionKey) {
    let node_id = instance.state.get_expression_node_id(&key);

    macro_rules! quick_node {
        ($label:expr) => {
            write_expression_node(instance, &node_id, &$label)
        };
    }

    match &instance.state.get_expression(key, asg).object {
        asg::ExpressionObject::Literal(n) => match n {
            asg::expressions::Literal::StringLiteral(n) => {
                quick_node!(format!("String Literal({})", escape_string(&n.string)))
            }
            asg::expressions::Literal::BoolLiteral(n) => {
                quick_node!(format!("Bool Literal({})", n.value))
            }
            asg::expressions::Literal::IntegerLiteral(n) => {
                quick_node!(format!("Integer Literal({})", n.data))
            } // TODO: Cast data to correct integer
            asg::expressions::Literal::StructLiteral(n) => {
                let mut count = 0;
                for field in &n.fields {
                    let field_id = write_structfield(instance, &node_id, field, count);

                    // Edges
                    instance.writer.queueline(format!(
                        "{} -> {} [label=\"field {}\"]",
                        node_id, field_id, count
                    ));

                    count += 1;
                }

                quick_node!(format!("Struct Literal"))
            }
            asg::expressions::Literal::FunctionLiteral(n) => {
                let function = instance.state.get_function(&n.functionkey, asg);
                let name = format!("Function: {}", function.name);

                // Edges
                instance.writer.queueline(format!(
                    "{} -> {}",
                    node_id,
                    instance.state.get_function_node_id(&n.functionkey)
                ));

                quick_node!(name);

                write_function(instance, asg, &n.functionkey);
            }
            asg::expressions::Literal::ModuleLiteral(n) => {
                // TODO: write module directly, or expression literal in between?
                let module = instance.state.get_module(&n.modulekey, asg);
                let name = format!("Module Literal: {}", module.name);

                // Edges
                instance.writer.queueline(format!(
                    "{} -> {}",
                    node_id,
                    instance.state.get_module_node_id(&n.modulekey)
                ));

                quick_node!(name);

                write_module(instance, asg, &n.modulekey);
            }
        },
        asg::ExpressionObject::BuiltInFunction(n) => {
            quick_node!(format!("Builtin({:?})", n.function))
        }
        asg::ExpressionObject::PrimitiveType(n) => quick_node!(format!("Primitive({:?})", n.ptype)),
        asg::ExpressionObject::SymbolReference(n) => {
            /*let scope = asg.store.symbolscopes.get(&n.symbolref.scope);

            let symref = scope.references.get(&n.symbolref.refkey);

            match symref {
                asg::SymbolReference::ResolvedReference(n) => {
                    let scope = asg.store.symbolscopes.get(&n.scope);
                    let symdecl = scope.declarations.get(&n.symbol);

                    // Edge
                    let symbolscope_id = get_symbolscope_node_id(&n.scope);
                    instance.writer.write(format!(
                        "{} -> {} [style=dashed constraint=false]\n",
                        node_id, symbolscope_id
                    ));

                    write_expression_node(
                        instance,
                        &node_id,
                        &format!("Resolved SymRef: {}", symdecl.symbol),
                    );
                }
                asg::SymbolReference::UnresolvedReference(n) => {
                    write_colored_expression_node(
                        instance,
                        &node_id,
                        &format!("Unresolved SymRef: {}", n.symbol),
                        &"red".to_string(),
                    );
                }
            };*/
        }
        asg::ExpressionObject::If(n) => {
            /*
            let mut branches = String::new();
            let mut count = 0;

            let mut it = n.branches.iter().peekable();
            while let Some(branch) = it.next() {
                let local_cond_from_id = format!("b{}e0", count);
                let cond_from_id = format!("{}:{}", node_id, local_cond_from_id);
                let cond_to_id = get_expression_node_id(&branch.0);

                let local_expr_from_id = format!("b{}e1", count);
                let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);
                let expr_to_id = get_expression_node_id(&branch.1);

                branches.push_str(
                    format!(
                        "<{}> expr |<{}> then",
                        local_cond_from_id, local_expr_from_id
                    )
                    .as_str(),
                );
                if it.peek().is_some() {
                    branches.push_str(" |");
                }

                // Edges
                instance
                    .writer
                    .write(format!("{} -> {}\n", cond_from_id, cond_to_id));
                instance
                    .writer
                    .write(format!("{} -> {}\n", expr_from_id, expr_to_id));

                count += 1;
            }

            // Else
            if let Some(elsebranch) = n.elsebranch {
                let local_expr_from_id = format!("b{}s0", count);
                let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);
                let expr_to_id = get_expression_node_id(&elsebranch);

                if !n.branches.is_empty() {
                    branches.push_str(" |");
                }
                branches.push_str(format!("else |<{}> expr", local_expr_from_id).as_str());

                // Edges
                instance
                    .writer
                    .write(format!("{} -> {}\n", expr_from_id, expr_to_id));
            }

            quick_node!(format!("if | {{ {} }}", branches))
            */
        }
        asg::ExpressionObject::Call(n) => {
            /*
            let mut label = String::new();

            label.push_str("Call |");

            let local_expr_from_id = format!("e0");
            let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);

            label.push_str(format!("<{}> callable", local_expr_from_id).as_str());

            // Edges
            instance.writer.write(format!(
                "{} -> {}\n",
                expr_from_id,
                get_expression_node_id(&n.callable)
            ));

            let mut count = 0;

            let mut it = n.args.iter().peekable();
            while let Some(arg) = it.next() {
                let local_expr_from_id = format!("a{}", count);
                let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);

                label.push_str(format!("|<{}> arg", local_expr_from_id).as_str());

                // Edges
                instance.writer.write(format!(
                    "{} -> {}\n",
                    expr_from_id,
                    get_expression_node_id(&arg)
                ));

                count += 1;
            }

            quick_node!(label)
            */
        }
        asg::ExpressionObject::BinOp(n) => {
            /*
            let local_lhs_from_id = "e0";
            let lhs_from_id = format!("{}:{}", node_id, local_lhs_from_id);
            let local_rhs_from_id = "e1";
            let rhs_from_id = format!("{}:{}", node_id, local_rhs_from_id);
            let lhs_to_id = get_expression_node_id(&n.lhs);
            let rhs_to_id = get_expression_node_id(&n.rhs);

            // Edges
            instance
                .writer
                .write(format!("{} -> {}\n", lhs_from_id, lhs_to_id));
            instance
                .writer
                .write(format!("{} -> {}\n", rhs_from_id, rhs_to_id));

            quick_node!(format!(
                "Binop |<{}> lhs | {:?} |<{}> rhs",
                local_lhs_from_id, n.op, local_rhs_from_id
            ))
            */
        }
        asg::ExpressionObject::Subscript(n) => {
            /*
            let local_expr_from_id = format!("e0");
            let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);

            // Edges
            instance.writer.write(format!(
                "{} -> {}\n",
                expr_from_id,
                get_expression_node_id(&n.expr)
            ));

            quick_node!(format!(
                "Subscript |<{}> expr | symbol: {}",
                local_expr_from_id, n.symbol
            ))
            */
        }
    };
}

pub fn generate_dotfile(asg: &asg::Asg, filename: &str) {
    let file = File::create(format!("target/{}.dot", filename)).expect("Could not create dotfile");
    let mut instance = Instance::new(file);

    // Header
    instance.writer.writeline(format!("digraph {}{{", filename));
    instance.writer.indent();

    // Disable splines
    instance.writer.writeline(format!("splines=false"));
    // Graph label
    instance.writer.writeline(format!(
        "label=\"Abstract Semantic Graph for {}\"",
        filename
    ));

    // Layout engine
    instance.writer.writeline(format!("layout=\"dot\""));

    // Content
    instance.with_module(&asg.global_module, asg, |instance| {
        let module = instance.state.get_current_module(asg);

        // Main
        write_function(instance, asg, &asg.main);

        // Global scope
        write_scope(instance, asg, &module.scope)
    });

    // Queued edges
    instance.writer.writeline(format!("// Edges"));
    instance.writer.flushqueue();

    // Write modules
    /*for key in asg.store.modules.keys() {
        write_module(&mut writer, &asg, &key);
    }

    // Write symbolscopes
    for key in asg.store.symbolscopes.keys() {
        write_symbolscope(&mut writer, &asg, &key);
    }

    // Write functions
    for key in asg.store.functions.keys() {
        write_function(&mut writer, &asg, &key);
    }

    // Write statemens bodies
    for key in asg.store.statementbodies.keys() {
        write_statementbody(&mut writer, &asg, &key);
    }

    // Write expressions
    for key in asg.store.expressions.keys() {
        write_expression(&mut writer, &asg, &key);
    }*/

    // Tail
    instance.writer.unindent();
    instance.writer.writeline("}".into());

    // Flush the buffer to ensure all data is written to the file
    //instance.state.flush();
}
