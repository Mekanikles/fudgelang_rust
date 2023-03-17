use super::utils::*;

const STRUCT_FIXTURE: &str = "\
    def A =\n\
        \tstruct begin\n\
            \t\tvar a : #primitives.u32\n\
            \t\tvar b : #primitives.u32\n\
            \t\tvar c : #primitives.u32\n\
        \tend\n\
    def B =\n\
    \tstruct begin\n\
        \t\tvar a : A\n\
    \tend\n\
    var a : A\n\
    var b : B\n\
    ";

fn append_to_fixture(fixture: &str, appendix: &str) -> String {
    format!("{}\n{}", fixture, appendix)
}

#[test]
fn test_struct_field_default_simple() {
    assert_expression_as_str_with_fixture(STRUCT_FIXTURE, "a.a", "0");
}

#[test]
fn test_struct_field_default_multiple() {
    assert_expression_as_str_with_fixture(STRUCT_FIXTURE, "a.a + a.b + a.c", "0");
}

#[test]
fn test_struct_field_assign_simple() {
    assert_expression_as_str_with_fixture(
        append_to_fixture(
            STRUCT_FIXTURE,
            "\
                a.a = 5
            ",
        )
        .as_str(),
        "a.a",
        "5",
    );
}

#[test]
fn test_struct_field_assign_multiple() {
    assert_expression_as_str_with_fixture(
        append_to_fixture(
            STRUCT_FIXTURE,
            "\
                a.a = 3
                a.b = 5
                a.c = 8
            ",
        )
        .as_str(),
        "a.a + a.b + a.c",
        "16",
    );
}

#[test]
fn test_struct_field_nested_default_simple() {
    assert_expression_as_str_with_fixture(STRUCT_FIXTURE, "b.a.a", "0");
}

#[test]
fn test_struct_field_nested_default_multiple() {
    assert_expression_as_str_with_fixture(STRUCT_FIXTURE, "b.a.a + b.a.b + b.a.c", "0");
}

#[test]
fn test_struct_field_nested_assign_simple() {
    assert_expression_as_str_with_fixture(
        append_to_fixture(
            STRUCT_FIXTURE,
            "\
                b.a.a = 5
            ",
        )
        .as_str(),
        "b.a.a",
        "5",
    );
}

#[test]
fn test_struct_field_nested_assign_multiple() {
    assert_expression_as_str_with_fixture(
        append_to_fixture(
            STRUCT_FIXTURE,
            "\
                b.a.a = 3
                b.a.b = 5
                b.a.c = 8
            ",
        )
        .as_str(),
        "b.a.a + b.a.b + b.a.c",
        "16",
    );
}
