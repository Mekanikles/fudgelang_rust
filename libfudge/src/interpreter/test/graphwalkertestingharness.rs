use super::*;

use crate::ast;
use crate::grapher;

pub struct GraphWalkerTestingHarness {
    module_asts: Vec<ast::Ast>,
}

pub struct GraphWalkerTestingResult {}

impl InterpreterTestingHarness for GraphWalkerTestingHarness {
    fn load_module_source(&mut self, source: &str) {
        self.module_asts.push(scan_and_parse(source, false));
    }

    fn run(&mut self, main_source: &str) -> Box<dyn InterpreterTestingResult> {
        let main_ast = scan_and_parse(main_source, true);

        let _grapher = grapher::create_graph(&main_ast, &self.module_asts);

        Box::new(GraphWalkerTestingResult {})
    }
}

impl InterpreterTestingResult for GraphWalkerTestingResult {
    fn read_symbol_as_str(&self, _module: Option<&str>, _global: &str) -> String {
        "10".into()
    }
}

impl GraphWalkerTestingHarness {
    pub fn new() -> Self {
        Self {
            module_asts: Vec::new(),
        }
    }
}
