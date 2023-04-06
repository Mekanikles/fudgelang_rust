use phf::phf_map;

use crate::utils::*;

use StringKey as SymbolKey;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimitiveType {
    StaticStringUtf8,
    Bool,
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

// Map with all addressable primitive types
pub static PRIMITIVES: phf::Map<&'static str, PrimitiveType> = phf_map! {
    "ssutf8" => PrimitiveType::StaticStringUtf8,
    "bool" => PrimitiveType::Bool,
    "u8" => PrimitiveType::U8,
    "u16" => PrimitiveType::U16,
    "u32" => PrimitiveType::U32,
    "u64" => PrimitiveType::U64,
    "s8" => PrimitiveType::S8,
    "s16" => PrimitiveType::S16,
    "s32" => PrimitiveType::S32,
    "s64" => PrimitiveType::S64,
    "f32" => PrimitiveType::F32,
    "f64" => PrimitiveType::F64,
};

#[derive(Debug, Clone, PartialEq)]
pub struct StructDefinition {
    pub fields: Vec<(SymbolKey, TypeId)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature {
    pub inputparams: Vec<(SymbolKey, TypeId)>,
    pub outputparams: Vec<TypeId>,
}

impl FunctionSignature {
    pub fn new_simple() -> Self {
        Self {
            inputparams: Vec::new(),
            outputparams: Vec::new(),
        }
    }
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
    Struct(StructDefinition),
    Module,
}
