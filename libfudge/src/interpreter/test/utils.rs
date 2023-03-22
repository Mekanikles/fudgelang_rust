use super::*;

pub fn test_interpreters_with_modules(
    source: &str,
    modules: &[&str],
    test: &dyn Fn(&dyn InterpreterTestingResult) -> (),
) {
    let mut twth = TreeWalkerTestingHarness::new();
    for module in modules {
        twth.load_module_source(module);
    }
    test(&twth.run(source));
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
