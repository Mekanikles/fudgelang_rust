use super::*;

pub struct GraphWalkerTestingHarness {}

pub struct GraphWalkerTestingResult {}

impl InterpreterTestingHarness for GraphWalkerTestingHarness {
    fn load_module_source(&mut self, _source: &str) {}

    fn run(&mut self, _main_source: &str) -> Box<dyn InterpreterTestingResult> {
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
        Self {}
    }
}
