use super::*;

fn test_interpreters(source: &str, test: &dyn Fn(&dyn InterpreterTestingResult) -> ()) {
    let mut twth = TreeWalkerTestingHarness::new();
    test(&twth.run(source));
}

#[test]
fn test_var_default() {
    test_interpreters(
        "\
            var a : #primitives.u32
        ",
        &|result| {
            assert_eq!(result.read_symbol_as_str(None, "a"), "0");
        },
    );
}

#[test]
fn test_var_assign() {
    test_interpreters(
        "\
            var a : #primitives.u32 = 5
        ",
        &|result| {
            assert_eq!(result.read_symbol_as_str(None, "a"), "5");
        },
    );
}
