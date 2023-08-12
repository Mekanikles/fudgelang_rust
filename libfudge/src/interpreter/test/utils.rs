use super::*;

use crate::ast;
use crate::parser;
use crate::parser::tokenstream::TokenStream;
use crate::scanner;
use crate::source;

pub fn scan_and_parse(source: &str, ismain: bool) -> ast::Ast {
    let source = source::Source::from_str(&source);
    let scanner_result = scanner::tokenize(&source);
    let parser_result = parser::parse(
        &mut TokenStream::new(&scanner_result.tokens, &source),
        ismain,
    );
    parser_result.ast
}

pub fn test_interpreters_with_modules(
    source: &str,
    modules: &[&str],
    test: &dyn Fn(&dyn InterpreterTestingResult) -> (),
) {
    let interpreters: Vec<Box<dyn InterpreterTestingHarness>> = vec![
        Box::new(TreeWalkerTestingHarness::new()),
        //Box::new(GraphWalkerTestingHarness::new())
    ];

    for mut i in interpreters {
        for module in modules {
            i.load_module_source(module);
        }
        test(&*i.run(source))
    }
}

pub fn test_interpreters(source: &str, test: &dyn Fn(&dyn InterpreterTestingResult) -> ()) {
    test_interpreters_with_modules(source, &[], test)
}

pub fn assert_expression_as_str_with_fixture(fixture: &str, exp: &str, expected: &str) {
    let source = format!(
        "\
            {}\n\
            def __res = {}\
        ",
        fixture, exp
    );

    test_interpreters(source.as_str(), &|result| {
        assert_eq!(result.read_symbol_as_str(None, "__res"), expected);
    });
}

pub fn assert_expression_as_str(exp: &str, expected: &str) {
    assert_expression_as_str_with_fixture("", exp, expected)
}
