use super::utils::*;

#[test]
fn test_var_default() {
    test_interpreters("var a : #primitives.u32", &|result| {
        assert_eq!(result.read_symbol_as_str(None, "a"), "0");
    });
}

#[test]
fn test_var_assign() {
    test_interpreters("var a : #primitives.u32 = 5", &|result| {
        assert_eq!(result.read_symbol_as_str(None, "a"), "5");
    });
}

#[test]
fn test_int_literal_expression() {
    assert_expression_as_str("5", "5");
}
