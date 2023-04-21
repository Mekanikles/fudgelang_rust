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
        let decl_id = get_symbol_node_id(&symbol_decl_key);
        let label = format!("{}", symbol_decl.symbol);
        symbol_labels.push_str(format!("<{}> {}", decl_id, label).as_str());
        if it.peek().is_some() {
            symbol_labels.push_str("|");
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

pub fn generate_dotfile(asg: &asg::Asg, filename: &str) {
    let file = File::create(format!("{}.dot", filename)).expect("Could not create dotfile");
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

    // Tail
    writer.write_all("}".as_bytes()).unwrap();

    // Flush the buffer to ensure all data is written to the file
    writer.flush().unwrap();
}
