use super::utils::*;

use phf::phf_map;

static ARIT_BINOPS: phf::Map<&'static str, fn(u32, u32) -> u32> = phf_map! {
    "+" => |a, b| a + b,
    "-" => |a, b| a - b,
    "*" => |a, b| a * b,
    "/" => |a, b| a / b,
};

static COMP_BINOPS: phf::Map<&'static str, fn(u32, u32) -> bool> = phf_map! {
    "==" => |a, b| a == b,
    ">" => |a, b| a > b,
    ">=" => |a, b| a >= b,
    "<" => |a, b| a < b,
    "<=" => |a, b| a <= b,
};

#[test]
fn test_int_arit_binop_expressions() {
    for op in ARIT_BINOPS.keys() {
        let a = 10;
        let b = 5;
        let exp = format!("{}{}{}", a, op, b);
        let x: fn(u32, u32) -> u32 = ARIT_BINOPS[op];
        let expected = format!("{}", x(a, b));
        assert_expression_as_str(exp.as_str(), expected.as_str());
    }
}

#[test]
fn test_int_comp_binop_expressions() {
    for op in COMP_BINOPS.keys() {
        let a = 10;
        let b = 5;
        let exp = format!("{}{}{}", a, op, b);
        let x: fn(u32, u32) -> bool = COMP_BINOPS[op];
        let expected = format!("{}", x(a, b));
        assert_expression_as_str(exp.as_str(), expected.as_str());
    }
}
