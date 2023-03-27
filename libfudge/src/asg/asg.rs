use std::collections::HashMap;
use std::collections::HashSet;

use crate::typesystem::*;
use crate::utils::*;

use crate::shared::BinaryOperationType;

pub struct Asg {
    pub modules: HashMap<ModuleKey, Module>,
    pub main: Main,
}

use StringKey as ModuleKey;
use StringKey as SymbolKey;

pub struct SymbolRef {
    pub module: ModuleKey,
    pub symbol: SymbolKey,
}

pub struct Main {
    pub body: StatementBody,
}

pub struct Module {
    pub name: String,
    pub parent: Option<ModuleKey>,
    pub globals: HashMap<SymbolKey, TypeId>,
    pub functions: HashMap<SymbolKey, Function>,
    pub submodules: HashSet<SymbolKey>,
    pub initalizer: StatementBody,
}

pub struct Function {
    pub name: String,
    pub signature: FunctionSignature,
    pub body: StatementBody,
}

mod statements {
    pub struct Return {
        expr: super::ExpressionRef,
    }
}

pub enum Statement {
    Return(statements::Return),
    Assign(statements::Return),
}

mod expressions {
    mod literals {
        pub struct StringLiteral {
            pub string: String,
        }
        pub struct BoolLiteral {
            pub value: bool,
        }
        pub struct IntegerLiteral {
            pub data: u64,
            pub signed: bool,
        }
        pub struct Function {
            pub key: super::super::SymbolKey,
        }
        pub struct Module {
            pub key: super::super::ModuleKey,
        }
    }

    pub enum Literal {
        StringLiteral(literals::StringLiteral),
    }

    pub struct Call {
        pub callable: super::SymbolRef,
        pub arguments: Vec<super::ExpressionRef>,
    }

    pub struct BinOp {
        pub op: super::BinaryOperationType,
        pub lhs: super::ExpressionRef,
        pub rhs: super::ExpressionRef,
    }
}

pub enum Expression {
    Literal(expressions::Literal),
    Call(expressions::Call),
    BinOp(expressions::BinOp),
}

use u64 as ExpressionRef;

pub struct StatementBody {
    pub statements: Vec<Statement>,
    pub expressions: Vec<Expression>,
}
