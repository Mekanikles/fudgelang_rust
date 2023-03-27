#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperationType {
    Add,
    Sub,
    Mul,
    Div,
    Equals,
    LessThan,
    LessThanOrEq,
    GreaterThan,
    GreaterThanOrEq,
}
