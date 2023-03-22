pub mod arithmetics;
pub mod basic;
pub mod modules;
pub mod structs;
pub mod utils;

use crate::interpreter::*;
use crate::parser;
use crate::parser::ast;
use crate::parser::stringstore::StringStore;
use crate::parser::tokenstream::TokenStream;
use crate::scanner;
use crate::source;

pub trait InterpreterTestingHarness<ResultT: InterpreterTestingResult> {
    fn load_module_source(&mut self, module_source: &str);
    fn run(&mut self, main_source: &str) -> ResultT;
}

pub trait InterpreterTestingResult {
    fn read_symbol_as_str(&self, module: Option<&str>, global: &str) -> String;
}

struct TreeWalkerTestingHarness {
    module_asts: Vec<parser::ast::Ast>,
}

struct TreeWalkerTestingResult {
    state: State,
}

impl InterpreterTestingHarness<TreeWalkerTestingResult> for TreeWalkerTestingHarness {
    fn load_module_source(&mut self, source: &str) {
        self.module_asts
            .push(TreeWalkerTestingHarness::scan_and_parse(source, false));
    }

    fn run(&mut self, main_source: &str) -> TreeWalkerTestingResult {
        let mut context = Context::new();

        let main_ast = TreeWalkerTestingHarness::scan_and_parse(main_source, true);

        context.asts.insert(main_ast.key, &main_ast);
        for module_ast in &self.module_asts {
            context.asts.insert(module_ast.key, module_ast);
        }

        let mut walker = TreeWalker::new(&context);
        walker.interpret();
        TreeWalkerTestingResult {
            state: walker.take_state(),
        }
    }
}

impl InterpreterTestingResult for TreeWalkerTestingResult {
    fn read_symbol_as_str(&self, module: Option<&str>, global: &str) -> String {
        let symbolref = ast::SymbolRef::from_str(global);
        let valref = if let Some(module) = module {
            self.state
                .lookup_symbol_from_module(StringStore::get_key(module), &symbolref)
                .unwrap()
        } else {
            self.state.lookup_symbol_from_stack(&symbolref).unwrap()
        };
        let val = self.state.full_deref_valueref(&valref);

        val.to_string(&self.state)
    }
}

impl TreeWalkerTestingHarness {
    fn new() -> TreeWalkerTestingHarness {
        TreeWalkerTestingHarness {
            module_asts: Vec::new(),
        }
    }

    fn scan_and_parse(source: &str, ismain: bool) -> parser::ast::Ast {
        let source = source::Source::from_str(&source);
        let scanner_result = scanner::tokenize(&source);
        let parser_result = parser::parse(
            &mut TokenStream::new(&scanner_result.tokens, &source),
            ismain,
        );
        parser_result.ast
    }
}
