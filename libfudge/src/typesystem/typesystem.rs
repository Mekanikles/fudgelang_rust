use phf::phf_map;

use crate::utils::*;

use StringKey as SymbolKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    // TODO: Should static string be a primitive? It cannot fit into a register
    // without being some kind of ref-only type, maybe differentiate between
    // built-ins and primitives
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

impl PrimitiveType {
    pub fn to_str(&self) -> &str {
        match self {
            PrimitiveType::StaticStringUtf8 => "str",
            PrimitiveType::Bool => "bool",
            PrimitiveType::U8 => "u8",
            PrimitiveType::U16 => "u16",
            PrimitiveType::U32 => "u32",
            PrimitiveType::U64 => "u64",
            PrimitiveType::S8 => "s8",
            PrimitiveType::S16 => "s16",
            PrimitiveType::S32 => "s32",
            PrimitiveType::S64 => "s64",
            PrimitiveType::F32 => "f32",
            PrimitiveType::F64 => "f64",
        }
    }

    pub fn data_to_string(&self, data: u64) -> String {
        match self {
            PrimitiveType::StaticStringUtf8 => format!("@c{}", data as u64),
            PrimitiveType::Bool => format!("{}", data != 0),
            PrimitiveType::U8 => format!("{}", data as u8),
            PrimitiveType::U16 => format!("{}", data as u16),
            PrimitiveType::U32 => format!("{}", data as u32),
            PrimitiveType::U64 => format!("{}", data as u64),
            PrimitiveType::S8 => format!("{}", data as i8),
            PrimitiveType::S16 => format!("{}", data as i16),
            PrimitiveType::S32 => format!("{}", data as i32),
            PrimitiveType::S64 => format!("{}", data as i64),
            PrimitiveType::F32 => format!("{}", data as f32),
            PrimitiveType::F64 => format!("{}", data as f64),
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            PrimitiveType::StaticStringUtf8 => 8, // u64 address, length is stored at address
            PrimitiveType::Bool => 1,
            PrimitiveType::U8 => 1,
            PrimitiveType::U16 => 2,
            PrimitiveType::U32 => 4,
            PrimitiveType::U64 => 8,
            PrimitiveType::S8 => 1,
            PrimitiveType::S16 => 2,
            PrimitiveType::S32 => 4,
            PrimitiveType::S64 => 8,
            PrimitiveType::F32 => 4,
            PrimitiveType::F64 => 8,
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructDefinition {
    pub fields: Vec<(SymbolKey, TypeId)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum BuiltInFunction {
    PrintFormat,
}

impl BuiltInFunction {
    pub fn to_str(&self) -> &str {
        match self {
            BuiltInFunction::PrintFormat => "#output.print_format",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeId {
    Null,
    Type,
    Primitive(PrimitiveType),
    // Cheat for complex built in signatures, until we have a competent type system for argument-dependent function signatures
    BuiltInFunction(BuiltInFunction),
    Function(FunctionSignature),
    Struct(StructDefinition),
    Module,
    // Hm, this is a bit awkward, perhaps this can be a core struct instead?
    TypedValue,
}

impl TypeId {
    pub fn new_primitive(ptype: PrimitiveType) -> Self {
        TypeId::Primitive(ptype)
    }

    pub fn type_id(&self) -> u64 {
        match self {
            TypeId::Primitive(n) => return *n as u64,
            _ => panic!(
                "Type id is only supported for primitives currently, not {:?}",
                self
            ),
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            TypeId::Primitive(n) => return n.size(),
            TypeId::TypedValue => return 2 * 8, // u64 typeid, u64 value
            _ => panic!(
                "Size is only supported for primitives currently, not {:?}",
                self
            ),
        }
    }

    pub fn is_primitive(&self, ptype: &PrimitiveType) -> bool {
        match &self {
            TypeId::Primitive(n) => *n == *ptype,
            _ => false,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            TypeId::Null => format!("null"),
            TypeId::Type => format!("type"),
            TypeId::Primitive(n) => n.to_str().into(),
            TypeId::BuiltInFunction(n) => n.to_str().into(),
            TypeId::Function(_) => format!("func"),
            TypeId::Struct(_) => format!("struct"),
            TypeId::Module => format!("module"),
            TypeId::TypedValue => format!("typedval"),
        }
    }
}
