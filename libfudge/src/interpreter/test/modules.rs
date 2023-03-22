use super::utils::*;

#[test]
fn test_module_simple() {
    test_interpreters(
        "\
            module a begin\n\
                \tdef b = 10
            end\n\
        ",
        &|result| {
            assert_eq!(result.read_symbol_as_str(Some("a"), "b"), "10");
        },
    );
}

#[test]
fn test_module_nested() {
    test_interpreters(
        "\
            module c begin\n\
                \tmodule a begin\n\
                    \t\tdef b = 10
                \tend\n\
            end\n\
            var a = c.a.b
        ",
        &|result| {
            assert_eq!(result.read_symbol_as_str(None, "a"), "10");
        },
    );
}
