use libfudgec::asg::{FunctionKey, StatementBodyKey};
use libfudgec::utils::objectstore::ObjectStore;
use libfudgec::*;

use std::fs::File;
use std::io::{BufWriter, Write};

fn get_symbolscope_node_id(key: &asg::SymbolScopeKey) -> String {
    format!("sc{}", key)
}

fn get_module_node_id(key: &asg::ModuleKey) -> String {
    format!("m{}", key)
}

fn get_symbol_node_id(key: &asg::SymbolKey) -> String {
    format!("s{}", key)
}

fn get_function_node_id(key: &asg::FunctionKey) -> String {
    format!("f{}", key)
}

fn get_expression_node_id(key: &asg::ExpressionKey) -> String {
    format!("e{}", key)
}

fn get_statementbody_node_id(key: &StatementBodyKey) -> String {
    format!("sb{}", key)
}

fn write_module(writer: &mut BufWriter<File>, asg: &asg::Asg, key: &asg::ModuleKey) {
    let module = asg.store.modules.get(&key);

    let node_id = get_module_node_id(&key);

    // Module node
    let label = format!("Module: {}", module.name);
    let shape = format!("box");
    let style = format!("rounded");
    writer.write_all(node_id.as_bytes()).unwrap();
    writer
        .write_all(
            format!(
                " [shape=\"{}\", style=\"{}\", label=\"{}\"]\n",
                shape, style, label
            )
            .as_bytes(),
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
                .as_bytes(),
            )
            .unwrap();

        // Initialiazer body
        if let Some(body) = &module.initalizer {
            writer
                .write_all(
                    format!("{} -> {}\n", node_id, get_statementbody_node_id(&body)).as_bytes(),
                )
                .unwrap();
        }
    }
}

fn write_symbolscope(writer: &mut BufWriter<File>, asg: &asg::Asg, key: &asg::SymbolScopeKey) {
    let symbolscope = asg.store.symbolscopes.get(&key);

    let node_id = get_symbolscope_node_id(&key);

    let mut symbol_labels = String::new();

    // Declarations
    let mut it = symbolscope.declarations.keys().peekable();
    while let Some(symbol_decl_key) = it.next() {
        let symbol_decl = symbolscope.declarations.get(&symbol_decl_key);

        let local_decl_from_id = format!("d{}", get_symbol_node_id(&symbol_decl_key));

        let label = format!("{}", symbol_decl.symbol);
        symbol_labels.push_str(format!("<{}> {}", local_decl_from_id, label).as_str());
        if it.peek().is_some() {
            symbol_labels.push_str("|");
        }

        if let Some(initexpr) = &symbol_decl.initexpr {
            let decl_from_id = format!("{}:{}", node_id, local_decl_from_id);
            let initexpr_to_id = get_expression_node_id(&initexpr);

            // Init expr edge
            writer
                .write_all(format!("{} -> {}\n", decl_from_id, initexpr_to_id).as_bytes())
                .unwrap();
        }

        if let Some(typeexpr) = &symbol_decl.typeexpr {
            let decl_from_id = format!("{}:{}", node_id, local_decl_from_id);
            let typeexpr_to_id = get_expression_node_id(&typeexpr);

            // Type expr edge
            writer
                .write_all(format!("{} -> {}\n", decl_from_id, typeexpr_to_id).as_bytes())
                .unwrap();
        }
    }

    let label = format!("{{ Symbols |{{ |{{ {} }}| }} }}", symbol_labels);
    let shape = format!("record");
    let style = format!("");
    writer.write_all(node_id.as_bytes()).unwrap();
    writer
        .write_all(
            format!(
                " [shape=\"{}\", style=\"{}\", label=\"{}\"]\n",
                shape, style, label
            )
            .as_bytes(),
        )
        .unwrap();
}

fn write_functionparameter(
    writer: &mut BufWriter<File>,
    asg: &asg::Asg,
    function_id: &String,
    index: usize,
    param: &asg::FunctionParameter,
) -> String {
    let node_id = format!("{}p{}", function_id, index);

    let local_expr_from_id = "t";
    let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);
    let expr_to_id = get_expression_node_id(&param.typeexpr);

    // Edges
    writer
        .write_all(format!("{} -> {}\n", expr_from_id, expr_to_id).as_bytes())
        .unwrap();

    let label = format!("in param | {} | <{}> type", param.name, local_expr_from_id);

    let shape = format!("record");
    let style = format!("");
    writer.write_all(node_id.as_bytes()).unwrap();
    writer
        .write_all(
            format!(
                " [shape=\"{}\", style=\"{}\", label=\"{}\"]\n",
                shape, style, label
            )
            .as_bytes(),
        )
        .unwrap();

    node_id
}

fn write_function(writer: &mut BufWriter<File>, asg: &asg::Asg, key: &asg::FunctionKey) {
    let function = asg.store.functions.get(&key);

    let node_id = get_function_node_id(&key);

    let label = format!("Function: {}", function.name);
    let shape = format!("component");
    let style = format!("");
    writer.write_all(node_id.as_bytes()).unwrap();
    writer
        .write_all(
            format!(
                " [shape=\"{}\", style=\"{}\", label=\"{}\"]\n",
                shape, style, label
            )
            .as_bytes(),
        )
        .unwrap();

    // Params
    let mut count = 0;
    for inparam in &function.inparams {
        let paramid = write_functionparameter(writer, &asg, &node_id, count, &inparam);

        writer
            .write_all(format!("{} -> {}\n", node_id, paramid).as_bytes())
            .unwrap();

        count += 1;
    }

    // Edges
    {
        // Module
        writer
            .write_all(
                format!("{} -> {}\n", get_module_node_id(&function.module), node_id).as_bytes(),
            )
            .unwrap();

        // Body
        if let Some(body) = &function.body {
            writer
                .write_all(
                    format!("{} -> {}\n", node_id, get_statementbody_node_id(&body)).as_bytes(),
                )
                .unwrap();
        }
    }
}

fn write_statement(
    writer: &mut BufWriter<File>,
    asg: &asg::Asg,
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
                    .write_all(format!("{} -> {}\n", expr_from_id, expr_to_id).as_bytes())
                    .unwrap();
                writer
                    .write_all(format!("{} -> {}\n", stmnt_from_id, stmnt_to_id).as_bytes())
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
                    .write_all(format!("{} -> {}\n", stmnt_from_id, stmnt_to_id).as_bytes())
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
                    .write_all(format!("{} -> {}\n", expr_from_id, expr_to_id).as_bytes())
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
                .write_all(format!("{} -> {}\n", lhs_from_id, lhs_to_id).as_bytes())
                .unwrap();
            writer
                .write_all(format!("{} -> {}\n", rhs_from_id, rhs_to_id).as_bytes())
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
                .write_all(format!("{} -> {}\n", expr_from_id, expr_to_id).as_bytes())
                .unwrap();

            format!("initalize | {} | <{}> expr", n.symbol, local_expr_from_id)
        }
        asg::Statement::ExpressionWrapper(n) => {
            let local_expr_from_id = "e0";
            let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);
            let expr_to_id = get_expression_node_id(&n.expr);

            // Edges
            writer
                .write_all(format!("{} -> {}\n", expr_from_id, expr_to_id).as_bytes())
                .unwrap();

            format!("expression wrapper | <{}> expr", local_expr_from_id)
        }
    };

    // Node
    let shape = format!("record");
    let style = format!("");
    writer.write_all(node_id.as_bytes()).unwrap();
    writer
        .write_all(
            format!(
                " [shape=\"{}\", style=\"{}\", label=\"{}\"]\n",
                shape, style, label
            )
            .as_bytes(),
        )
        .unwrap();

    node_id
}

fn write_statementbody(writer: &mut BufWriter<File>, asg: &asg::Asg, key: &asg::FunctionKey) {
    let body = asg.store.statementbodies.get(&key);

    let node_id = get_statementbody_node_id(&key);

    // Statements
    let mut it = body.statements.iter().peekable();
    let mut count = 0;
    while let Some(stmnt) = it.next() {
        let stmnt_id = write_statement(writer, &asg, &node_id, count, &stmnt);

        // Edges
        writer
            .write_all(format!("{} -> {} [label=\"[{}]\"]\n", node_id, stmnt_id, count).as_bytes())
            .unwrap();
        count += 1;
    }

    // Node
    let label = format!("Statement Body");
    let shape = format!("box");
    let style = format!("");
    writer.write_all(node_id.as_bytes()).unwrap();
    writer
        .write_all(
            format!(
                " [shape=\"{}\", style=\"{}\", label=\"{}\"]\n",
                shape, style, label
            )
            .as_bytes(),
        )
        .unwrap();
}

fn escape_string(string: &String) -> String {
    // TODO: This is horribly inefficient
    string
        .replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace("{", "\\{")
        .replace("}", "\\}")
}

fn write_expression(writer: &mut BufWriter<File>, asg: &asg::Asg, key: &asg::ExpressionKey) {
    let expr = asg.store.expressions.get(key);

    let node_id = get_expression_node_id(&key);

    let label = match expr {
        asg::Expression::Literal(n) => match n {
            asg::expressions::Literal::StringLiteral(n) => {
                format!("String Literal({})", escape_string(&n.string))
            }
            asg::expressions::Literal::BoolLiteral(n) => format!("Bool Literal({})", n.value),
            asg::expressions::Literal::IntegerLiteral(n) => format!("Integer Literal({})", n.data), // TODO: Cast data to correct integer
            asg::expressions::Literal::StructLiteral(n) => format!("Struct Literal"), // TODO
            asg::expressions::Literal::FunctionLiteral(n) => {
                let function = asg.store.modules.get(&n.functionkey);
                let name = format!("Function: {}", function.name);

                // Edges
                writer
                    .write_all(
                        format!("{} -> {}\n", node_id, get_function_node_id(&n.functionkey))
                            .as_bytes(),
                    )
                    .unwrap();

                name
            }
            asg::expressions::Literal::ModuleLiteral(n) => {
                let module = asg.store.modules.get(&n.modulekey);
                let name = format!("Module Literal: {}", module.name);

                // Edges
                writer
                    .write_all(
                        format!("{} -> {}\n", node_id, get_module_node_id(&n.modulekey)).as_bytes(),
                    )
                    .unwrap();

                name
            }
        },
        asg::Expression::BuiltInFunction(n) => format!("Builtin({:?})", n.function),
        asg::Expression::PrimitiveType(n) => format!("Primitive({:?})", n.ptype),
        asg::Expression::SymbolReference(n) => format!("Symbol Reference(TODO)"),
        asg::Expression::If(n) => {
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
                writer
                    .write_all(format!("{} -> {}\n", cond_from_id, cond_to_id).as_bytes())
                    .unwrap();
                writer
                    .write_all(format!("{} -> {}\n", expr_from_id, expr_to_id).as_bytes())
                    .unwrap();

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
                writer
                    .write_all(format!("{} -> {}\n", expr_from_id, expr_to_id).as_bytes())
                    .unwrap();
            }

            format!("if | {{ {} }}", branches)
        }
        asg::Expression::Call(n) => {
            let mut label = String::new();

            label.push_str("Call |");

            let local_expr_from_id = format!("e0");
            let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);

            label.push_str(format!("<{}> callable", local_expr_from_id).as_str());

            // Edges
            writer
                .write_all(
                    format!(
                        "{} -> {}\n",
                        expr_from_id,
                        get_expression_node_id(&n.callable)
                    )
                    .as_bytes(),
                )
                .unwrap();

            let mut count = 0;

            let mut it = n.args.iter().peekable();
            while let Some(arg) = it.next() {
                let local_expr_from_id = format!("a{}", count);
                let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);

                label.push_str(format!("|<{}> arg", local_expr_from_id).as_str());

                // Edges
                writer
                    .write_all(
                        format!("{} -> {}\n", expr_from_id, get_expression_node_id(&arg))
                            .as_bytes(),
                    )
                    .unwrap();

                count += 1;
            }

            label
        }
        asg::Expression::BinOp(n) => {
            let local_lhs_from_id = "e0";
            let lhs_from_id = format!("{}:{}", node_id, local_lhs_from_id);
            let local_rhs_from_id = "e1";
            let rhs_from_id = format!("{}:{}", node_id, local_rhs_from_id);
            let lhs_to_id = get_expression_node_id(&n.lhs);
            let rhs_to_id = get_expression_node_id(&n.rhs);

            // Edges
            writer
                .write_all(format!("{} -> {}\n", lhs_from_id, lhs_to_id).as_bytes())
                .unwrap();
            writer
                .write_all(format!("{} -> {}\n", rhs_from_id, rhs_to_id).as_bytes())
                .unwrap();

            format!(
                "Binop |<{}> lhs | {:?} |<{}> rhs",
                local_lhs_from_id, n.op, local_rhs_from_id
            )
        }
        asg::Expression::Subscript(n) => {
            let local_expr_from_id = format!("e0");
            let expr_from_id = format!("{}:{}", node_id, local_expr_from_id);

            // Edges
            writer
                .write_all(
                    format!("{} -> {}\n", expr_from_id, get_expression_node_id(&n.expr)).as_bytes(),
                )
                .unwrap();

            format!(
                "Subscript |<{}> expr | symbol: {}",
                local_expr_from_id, n.symbol
            )
        }
    };

    // Node
    let shape = format!("Mrecord");
    let style = format!("");
    writer.write_all(node_id.as_bytes()).unwrap();
    writer
        .write_all(
            format!(
                " [shape=\"{}\", style=\"{}\", label=\"{}\"]\n",
                shape, style, label
            )
            .as_bytes(),
        )
        .unwrap();
}

pub fn generate_dotfile(asg: &asg::Asg, filename: &str) {
    let file = File::create(format!("target/{}.dot", filename)).expect("Could not create dotfile");
    let mut writer = BufWriter::new(file);

    // Header
    writer
        .write_all(format!("digraph {}{{ \n", filename).as_bytes())
        .unwrap();

    // Graph label
    writer
        .write_all(format!("label=\"Abstract Semantic Graph for {}\"\n", filename).as_bytes())
        .unwrap();

    // Write modules
    for key in asg.store.modules.keys() {
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
    }

    // Tail
    writer.write_all("}".as_bytes()).unwrap();

    // Flush the buffer to ensure all data is written to the file
    writer.flush().unwrap();
}
