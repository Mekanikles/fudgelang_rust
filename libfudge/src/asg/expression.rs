use super::*;

use crate::shared::BinaryOperationType;
use crate::typesystem;

use scope::ExpressionKey;

pub mod misc {
    use super::*;

    #[derive(Debug)]
    pub struct StructField {
        pub name: String,
        pub typeexpr: ExpressionKey,
    }
}

pub mod expressions {
    use super::*;

    pub mod literals {
        use super::*;

        #[derive(Debug)]
        pub struct StringLiteral {
            pub string: String,
        }
        #[derive(Debug)]
        pub struct BoolLiteral {
            pub value: bool,
        }
        #[derive(Debug)]
        pub struct IntegerLiteral {
            pub data: u64,
            pub signed: bool,
        }
        #[derive(Debug)]
        pub struct StructLiteral {
            pub fields: Vec<misc::StructField>,
        }
        #[derive(Debug)]
        pub struct FunctionLiteral {
            pub functionkey: FunctionKey,
        }
        #[derive(Debug)]
        pub struct ModuleLiteral {
            pub modulekey: ModuleKey,
        }
    }

    #[derive(Debug)]
    pub enum Literal {
        StringLiteral(literals::StringLiteral),
        BoolLiteral(literals::BoolLiteral),
        IntegerLiteral(literals::IntegerLiteral),
        StructLiteral(literals::StructLiteral),
        FunctionLiteral(literals::FunctionLiteral),
        ModuleLiteral(literals::ModuleLiteral),
    }

    #[derive(Debug)]
    pub struct BuiltInFunction {
        pub function: typesystem::BuiltInFunction,
    }

    #[derive(Debug)]
    pub struct PrimitiveType {
        pub ptype: typesystem::PrimitiveType,
    }

    #[derive(Debug)]
    pub struct SymbolReference {
        pub symbolref: symboltable::SymbolReferenceKey,
    }

    #[derive(Debug)]
    pub struct If {
        pub branches: Vec<(ExpressionKey, ExpressionKey)>,
        pub elsebranch: Option<ExpressionKey>,
    }

    #[derive(Debug)]
    pub struct Call {
        pub callable: ExpressionKey,
        pub args: Vec<ExpressionKey>,
    }

    #[derive(Debug)]
    pub struct BinOp {
        pub op: BinaryOperationType,
        pub lhs: ExpressionKey,
        pub rhs: ExpressionKey,
    }

    #[derive(Debug)]
    pub struct Subscript {
        pub expr: ExpressionKey,
        pub symbol: String,
    }
}

#[derive(Debug)]
pub enum ExpressionObject {
    Literal(expressions::Literal),
    BuiltInFunction(expressions::BuiltInFunction),
    PrimitiveType(expressions::PrimitiveType),
    SymbolReference(expressions::SymbolReference),
    If(expressions::If),
    Call(expressions::Call),
    BinOp(expressions::BinOp),
    Subscript(expressions::Subscript),
}

#[derive(Debug)]
pub struct Expression {
    pub object: ExpressionObject,
    pub statementindex: usize,
}

impl Expression {
    pub fn new(object: ExpressionObject, statementindex: usize) -> Self {
        Expression {
            object,
            statementindex,
        }
    }
}
