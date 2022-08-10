use crate::parser::stringstore::*;

use StringKey as SymbolKey;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimitiveType {
    StaticStringUtf8,
    U8,
    U16,
    U32,
    U64,
    S8,
    S16,
    S32,
    S64,
    F32,
    F64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub inputparams: Vec<(SymbolKey, TypeId)>,
    pub outputparams: Vec<TypeId>,
}

// Cheat a bit and treat all built-ins as their own unique types
// TODO: We want to express the type of built-ins through the regular type system, including signatures with dependent types
#[derive(Debug, Clone, PartialEq)]
pub enum BuiltInFunction {
    PrintFormat,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeId {
    Null,
    Type,
    Primitive(PrimitiveType),
    // Cheat for complex built in signatures, until we have a competent type system for argument-dependent function signatures
    BuiltInFunction(BuiltInFunction),
    Function(FunctionSignature),
}
