use super::*;

use crate::utils::objectstore::*;
use crate::utils::*;

use crate::typesystem::*;

pub type BasicBlockStore = IndexedObjectStore<BasicBlock>;
pub type BasicBlockKey = usize;

pub type FunctionStore = IndexedObjectStore<Function>;
pub type FunctionKey = usize;

pub type VariableStore = IndexedObjectStore<Variable>;
pub type VariableKey = usize;

pub type ConstantDataStore = IndexedObjectStore<ConstantData>;
pub type ConstantDataKey = usize;

#[derive(Debug)]
pub enum Instruction {
    Assign(instructions::Assign),
    CallBuiltIn(instructions::CallBuiltIn),
    CallStatic(instructions::CallStatic),
    Return(instructions::Return),
    Halt(instructions::Halt),
}

pub mod instructions {
    use super::*;

    #[derive(Debug)]
    pub struct Assign {
        pub variable: VariableKey,
        pub expression: Expression,
    }

    #[derive(Debug)]
    pub struct CallBuiltIn {
        pub variable: VariableKey,
        pub builtin: BuiltInFunction,
        pub args: Vec<VariableKey>,
    }

    #[derive(Debug)]
    pub struct CallStatic {
        pub variable: VariableKey,
        pub function: FunctionKey,
        pub args: Vec<VariableKey>,
    }

    #[derive(Debug)]
    pub struct Return {
        pub values: Vec<VariableKey>,
    }

    #[derive(Debug)]
    pub struct Halt {}
}

#[derive(Debug, Clone)]
pub enum Expression {
    Variable(VariableKey),
    Constant(Value),
}

impl Expression {
    pub fn get_type(&self, functionbuilder: &FunctionBuilder) -> TypeId {
        match self {
            Expression::Variable(n) => functionbuilder.get_variable(n).get_type().clone(), // Ugh, clone
            Expression::Constant(n) => n.get_type(),
        }
    }
}

#[derive(Debug)]
pub enum Variable {
    Named { symbol: StringKey, typeid: TypeId },
    Unnamed { typeid: TypeId },
}

impl Variable {
    pub fn get_type(&self) -> &TypeId {
        match self {
            Variable::Named { symbol: _, typeid } => typeid,
            Variable::Unnamed { typeid } => typeid,
        }
    }
}

#[derive(Debug)]
pub struct ConstantData {
    pub typeid: TypeId,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub enum Value {
    Primitive {
        ptype: PrimitiveType,
        data: u64,
    },
    BuiltInFunction {
        builtin: BuiltInFunction,
    },
    // Hm, this is a wrapper around a ref to another variable, does this setup make sense?
    TypedValue {
        typeid: TypeId,
        variable: VariableKey,
    },
}

impl Value {
    pub fn get_type(&self) -> TypeId {
        match self {
            Value::Primitive { ptype, data: _ } => TypeId::Primitive(*ptype),
            Value::BuiltInFunction { builtin } => TypeId::BuiltInFunction(*builtin),
            Value::TypedValue {
                typeid: _,
                variable: _,
            } => TypeId::TypedValue,
        }
    }

    pub fn get_size(&self) -> u64 {
        self.get_type().size()
    }
}

pub fn create_constant_staticstringutf8(str: &str) -> ConstantData {
    // ut8staticstring is [bytelen: u64, data: &[u8]]
    let str = str.as_bytes();
    let size = 8 + str.len();
    let mut data = vec![0; size];

    // TODO: We should not need to store the length alongside the data
    //  the string array length should be known, or packed in the string-ref type
    data[0..8].copy_from_slice(&str.len().to_be_bytes());
    data[8..].copy_from_slice(str);

    ConstantData {
        typeid: TypeId::Primitive(PrimitiveType::StaticStringUtf8),
        data,
    }
}
