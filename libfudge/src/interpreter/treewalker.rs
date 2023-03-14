use crate::parser::ast;
use crate::parser::stringstore::*;
use crate::typesystem::*;

use std::collections::HashMap;
use std::fmt;
use std::mem;

use dyn_fmt::AsStrFormatExt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct AstRef {
    ast_key: ast::AstKey,
    noderef: ast::NodeRef,
}

fn from_astref(astref: &AstRef, noderef: &ast::NodeRef) -> AstRef {
    return AstRef {
        ast_key: astref.ast_key,
        noderef: *noderef,
    };
}

fn from_ast(ast: &ast::Ast) -> AstRef {
    return AstRef {
        ast_key: ast.key,
        noderef: ast.get_root().unwrap(),
    };
}

fn from_ast_and_node(ast: &ast::Ast, noderef: &ast::NodeRef) -> AstRef {
    return AstRef {
        ast_key: ast.key,
        noderef: *noderef,
    };
}

pub struct Context<'a> {
    pub asts: HashMap<ast::AstKey, &'a ast::Ast>,
}

#[derive(Debug)]
pub struct VariableEnvironment {
    symbol_storage_lookup: HashMap<ast::SymbolRef, usize>,
    storage: Vec<Value>,
}

impl VariableEnvironment {
    fn new() -> VariableEnvironment {
        VariableEnvironment {
            symbol_storage_lookup: HashMap::new(),
            storage: Vec::new(),
        }
    }

    fn add(&mut self, v: Value) -> usize {
        let index = self.storage.len();
        self.storage.push(v);
        index
    }

    fn get(&self, index: usize) -> &Value {
        &self.storage[index]
    }

    fn get_mut(&mut self, index: usize) -> &mut Value {
        self.storage.get_mut(index).unwrap()
    }

    fn add_with_symbol(&mut self, s: ast::SymbolRef, v: Value) -> usize {
        let index = self.add(v);
        self.symbol_storage_lookup.insert(s, index);
        index
    }

    pub fn get_from_symbol(&self, s: &ast::SymbolRef) -> Option<&Value> {
        if let Some(index) = self.symbol_storage_lookup.get(s) {
            return Some(self.get(*index));
        }
        None
    }

    fn get_from_symbol_mut(&mut self, s: &ast::SymbolRef) -> Option<&mut Value> {
        if let Some(index) = self.symbol_storage_lookup.get(s) {
            return self.storage.get_mut(*index);
        }
        None
    }

    fn has_symbol(&self, s: &ast::SymbolRef) -> bool {
        self.symbol_storage_lookup.contains_key(s)
    }
}

pub struct Module {
    pub name: String,
    pub astref: Option<AstRef>,
    pub parent: Option<u64>,
    pub globals: VariableEnvironment,
    pub functions: Vec<Function>,
    pub modules: VariableEnvironment,
}

impl Module {
    fn new(name: String, astref: Option<AstRef>, parent: Option<u64>) -> Module {
        Module {
            name: name,
            astref: astref,
            parent: parent,
            globals: VariableEnvironment::new(),
            functions: Vec::new(),
            modules: VariableEnvironment::new(),
        }
    }
}

impl<'a> Context<'a> {
    pub fn new() -> Context<'a> {
        Context {
            asts: HashMap::new(),
        }
    }

    pub fn get_ast(&self, astref: &AstRef) -> &ast::Ast {
        return self.asts[&astref.ast_key];
    }

    pub fn get_node(&self, astref: &AstRef) -> &ast::Node {
        return self.asts[&astref.ast_key].get_node(&astref.noderef);
    }
}

pub struct State {
    pub all_modules: HashMap<u64, Module>,
    pub global_module: u64,
    pub strings: Vec<String>,
    pub stackframes: Vec<StackFrame>,
    pub current_module: Option<u64>,
}

pub struct TreeWalker<'a> {
    state: State,
    context: &'a Context<'a>,
}

#[derive(Debug)]
pub struct StackFrame {
    index: usize,
    variables: VariableEnvironment,
    returnvalue: Option<Value>,
}

pub struct Function {
    module: u64,
    signature: FunctionSignature,
    body: AstRef,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionRef {
    index: u64,
    module: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructInstance {
    pub definition: StructDefinition,
    pub fields: HashMap<ast::SymbolRef, Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Utf8StaticString(u64);
#[derive(Debug, Clone, PartialEq)]
pub struct Bool(bool);
#[derive(Debug, Clone, PartialEq)]
pub struct U8(u8);
#[derive(Debug, Clone, PartialEq)]
pub struct U16(u16);
#[derive(Debug, Clone, PartialEq)]
pub struct U32(u32);
#[derive(Debug, Clone, PartialEq)]
pub struct U64(u64);
#[derive(Debug, Clone, PartialEq)]
pub struct S8(i16);
#[derive(Debug, Clone, PartialEq)]
pub struct S16(i16);
#[derive(Debug, Clone, PartialEq)]
pub struct S32(i32);
#[derive(Debug, Clone, PartialEq)]
pub struct S64(i64);
#[derive(Debug, Clone, PartialEq)]
pub struct F32(f32);
#[derive(Debug, Clone, PartialEq)]
pub struct F64(f64);

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveValue {
    Utf8StaticString(Utf8StaticString),
    Bool(Bool),
    U8(U8),
    U16(U16),
    U32(U32),
    U64(U64),
    S8(S8),
    S16(S16),
    S32(S32),
    S64(S64),
    F32(F32),
    F64(F64),
}

// This is ass, but I need to be able to specialize traits on the enum "variants"
pub struct Add;
pub struct Mul;
pub struct Sub;
pub struct Div;
pub struct Equals;
pub struct LessThan;
pub struct LessThanOrEq;
pub struct GeaterThan;
pub struct GreaterThanOrEq;

trait BinOp<Op, Rhs = Self> {
    fn perform(&self, rhs: &Rhs) -> Value;
}

fn perform_binop<
    T: BinOp<Add>
        + BinOp<Sub>
        + BinOp<Mul>
        + BinOp<Div>
        + BinOp<Equals>
        + BinOp<LessThan>
        + BinOp<LessThanOrEq>
        + BinOp<GeaterThan>
        + BinOp<GreaterThanOrEq>,
>(
    op: &ast::BinaryOperationType,
    lhs: &T,
    rhs: &T,
) -> Value {
    return match op {
        ast::BinaryOperationType::Add => BinOp::<Add>::perform(lhs, rhs),
        ast::BinaryOperationType::Sub => BinOp::<Sub>::perform(lhs, rhs),
        ast::BinaryOperationType::Mul => BinOp::<Mul>::perform(lhs, rhs),
        ast::BinaryOperationType::Div => BinOp::<Div>::perform(lhs, rhs),
        ast::BinaryOperationType::Equals => BinOp::<Equals>::perform(lhs, rhs),
        ast::BinaryOperationType::LessThan => BinOp::<LessThan>::perform(lhs, rhs),
        ast::BinaryOperationType::LessThanOrEq => BinOp::<LessThanOrEq>::perform(lhs, rhs),
        ast::BinaryOperationType::GreaterThan => BinOp::<GeaterThan>::perform(lhs, rhs),
        ast::BinaryOperationType::GreaterThanOrEq => BinOp::<GreaterThanOrEq>::perform(lhs, rhs),
    };
}

macro_rules! primitive_binop_impl {
    ($optrait:ty, $op:tt, $($t:tt,)*) => ($(
        impl $optrait for $t {
            #[inline]
            fn perform(&self, rhs: &$t) -> Value {
                Value::Primitive(PrimitiveValue::$t($t(self.0 $op rhs.0)))
            }
        }
    )*)
}

macro_rules! primitive_comparison_impl {
    ($optrait:ty, $op:tt, $($t:tt,)*) => ($(
        impl $optrait for $t {
            #[inline]
            fn perform(&self, rhs: &$t) -> Value {
                Value::Primitive(PrimitiveValue::Bool(Bool(self.0 $op rhs.0)))
            }
        }
    )*)
}

macro_rules! primitive_binop_unsupported {
    ($optrait:ty, $($t:ty,)*) => ($(
        impl $optrait for $t {
            #[inline]
            fn perform(&self, _: &$t) -> Value {
                panic!("Binary operation {}, not supported for {}", stringify!($optrait), stringify!($t))
            }
        }
    )*)
}

primitive_binop_impl!(BinOp<Add>, +, U8, U16, U32, U64, S8, S16, S32, S64, F32, F64,);
primitive_binop_unsupported!(BinOp<Add>, Utf8StaticString,);

primitive_binop_impl!(BinOp<Sub>, -, U8, U16, U32, U64, S8, S16, S32, S64, F32, F64,);
primitive_binop_unsupported!(BinOp<Sub>, Utf8StaticString,);

primitive_binop_impl!(BinOp<Mul>, *, U8, U16, U32, U64, S8, S16, S32, S64, F32, F64,);
primitive_binop_unsupported!(BinOp<Mul>, Utf8StaticString,);

primitive_binop_impl!(BinOp<Div>, /, U8, U16, U32, U64, S8, S16, S32, S64, F32, F64,);
primitive_binop_unsupported!(BinOp<Div>, Utf8StaticString,);

primitive_comparison_impl!(BinOp<Equals>, ==, U8, U16, U32, U64, S8, S16, S32, S64, F32, F64,);
primitive_binop_unsupported!(BinOp<Equals>, Utf8StaticString,);

primitive_comparison_impl!(BinOp<LessThan>, <, U8, U16, U32, U64, S8, S16, S32, S64, F32, F64,);
primitive_binop_unsupported!(BinOp<LessThan>, Utf8StaticString,);

primitive_comparison_impl!(BinOp<LessThanOrEq>, <=, U8, U16, U32, U64, S8, S16, S32, S64, F32, F64,);
primitive_binop_unsupported!(BinOp<LessThanOrEq>, Utf8StaticString,);

primitive_comparison_impl!(BinOp<GeaterThan>, >, U8, U16, U32, U64, S8, S16, S32, S64, F32, F64,);
primitive_binop_unsupported!(BinOp<GeaterThan>, Utf8StaticString,);

primitive_comparison_impl!(BinOp<GreaterThanOrEq>, >=, U8, U16, U32, U64, S8, S16, S32, S64, F32, F64,);
primitive_binop_unsupported!(BinOp<GreaterThanOrEq>, Utf8StaticString,);

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Type(TypeId),
    Primitive(PrimitiveValue),
    BuiltInFunction(BuiltInFunction),
    Function(FunctionRef),
    StructInstance(StructInstance),
    Module(StringRef),
    ValueRef(ValueRef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexedStackValueRef {
    frame: usize,
    index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedStackValueRef {
    frame: usize,
    symbol: ast::SymbolRef,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IndexedGlobalValueRef {
    module: u64,
    index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NamedGlobalValueRef {
    module: u64,
    symbol: ast::SymbolRef,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubModuleValueRef {
    module: u64,
    submodule: ast::SymbolRef,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SimpleValueRef {
    NamedStackValueRef(NamedStackValueRef),
    IndexedStackValueRef(IndexedStackValueRef),
    NamedGlobalValueRef(NamedGlobalValueRef),
    IndexedGlobalValueRef(IndexedGlobalValueRef),
    SubModuleValueRef(SubModuleValueRef),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SubscriptedValueRef {
    vref: SimpleValueRef,
    field: ast::SymbolRef,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueRef {
    SimpleValueRef(SimpleValueRef),
    SubscriptedValueRef(SubscriptedValueRef),
}

pub struct ValueRefDisplay<'a> {
    pub vref: &'a ValueRef,
    pub state: &'a State,
}

fn get_simple_valueref_debug_name<'a>(svref: &SimpleValueRef, state: &State) -> String {
    match svref {
        SimpleValueRef::NamedStackValueRef(r) => format!("NamedStackRef({})", r.symbol),
        SimpleValueRef::IndexedStackValueRef(r) => format!("IndexedStackRef({})", r.index),
        SimpleValueRef::NamedGlobalValueRef(r) => {
            let module = state.get_module(&r.module);
            format!("NamedGlobal({}.{})", module.name, r.symbol)
        }
        SimpleValueRef::IndexedGlobalValueRef(r) => {
            let module = state.get_module(&r.module);
            format!("IndexedGlobal({}.{})", module.name, r.index)
        }
        SimpleValueRef::SubModuleValueRef(r) => {
            let module = state.get_module(&r.module);
            let submodule = match module.modules.get_from_symbol(&r.submodule).unwrap() {
                Value::Module(m) => state.get_module(&m.key),
                _ => panic!("Found submodule value what was not a module"),
            };
            format!("SubModule({}.{})", module.name, submodule.name,)
        }
    }
}

// Debug formatter
impl<'a> fmt::Debug for ValueRefDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.vref {
            ValueRef::SimpleValueRef(r) => {
                write!(
                    f,
                    "SimpleValueRef({})",
                    get_simple_valueref_debug_name(r, self.state)
                )
            }
            ValueRef::SubscriptedValueRef(r) => {
                write!(
                    f,
                    "SubscriptedValueRef({}.{})",
                    get_simple_valueref_debug_name(&r.vref, self.state),
                    r.field
                )
            }
        }
    }
}

pub struct ValueDisplay<'a> {
    pub v: &'a Value,
    pub state: &'a State,
}

// Debug formatter
impl<'a> fmt::Debug for ValueDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.v {
            Value::ValueRef(vref) => ValueRefDisplay {
                vref,
                state: self.state,
            }
            .fmt(f),
            Value::Module(m) => f
                .debug_tuple("Module")
                .field(&self.state.get_module(&m.key).name)
                .finish(),
            n => n.fmt(f),
        }
    }
}

impl Value {
    fn get_type(&self, state: &State) -> TypeId {
        match self.get_inner_ref(state) {
            Value::Null => TypeId::Null,
            Value::Type(_) => TypeId::Type,
            Value::Primitive(p) => match p {
                PrimitiveValue::Utf8StaticString(_) => {
                    TypeId::Primitive(PrimitiveType::StaticStringUtf8)
                }
                PrimitiveValue::Bool(_) => TypeId::Primitive(PrimitiveType::Bool),
                PrimitiveValue::U8(_) => TypeId::Primitive(PrimitiveType::U8),
                PrimitiveValue::U16(_) => TypeId::Primitive(PrimitiveType::U16),
                PrimitiveValue::U32(_) => TypeId::Primitive(PrimitiveType::U32),
                PrimitiveValue::U64(_) => TypeId::Primitive(PrimitiveType::U64),
                PrimitiveValue::S8(_) => TypeId::Primitive(PrimitiveType::S8),
                PrimitiveValue::S16(_) => TypeId::Primitive(PrimitiveType::S16),
                PrimitiveValue::S32(_) => TypeId::Primitive(PrimitiveType::S32),
                PrimitiveValue::S64(_) => TypeId::Primitive(PrimitiveType::S64),
                PrimitiveValue::F32(_) => TypeId::Primitive(PrimitiveType::F32),
                PrimitiveValue::F64(_) => TypeId::Primitive(PrimitiveType::F64),
            },
            Value::BuiltInFunction(v) => TypeId::BuiltInFunction(v.clone()),
            Value::Function(fref) => TypeId::Function(
                state.get_module(&fref.module).functions[fref.index as usize]
                    .signature
                    .clone(),
            ),
            Value::StructInstance(instance) => TypeId::Struct(instance.definition.clone()),
            Value::Module(_) => TypeId::Module,
            _ => panic!("Value type cannot be found: {:?}", &self),
        }
    }

    pub fn to_string(&self, state: &State) -> String {
        match self.get_inner_ref(state) {
            Value::Primitive(p) => match p {
                PrimitiveValue::Utf8StaticString(v) => state.strings[v.0 as usize].clone(),
                PrimitiveValue::Bool(v) => v.0.to_string(),
                PrimitiveValue::U8(v) => v.0.to_string(),
                PrimitiveValue::U16(v) => v.0.to_string(),
                PrimitiveValue::U32(v) => v.0.to_string(),
                PrimitiveValue::U64(v) => v.0.to_string(),
                PrimitiveValue::S8(v) => v.0.to_string(),
                PrimitiveValue::S16(v) => v.0.to_string(),
                PrimitiveValue::S32(v) => v.0.to_string(),
                PrimitiveValue::S64(v) => v.0.to_string(),
                PrimitiveValue::F32(v) => v.0.to_string(),
                PrimitiveValue::F64(v) => v.0.to_string(),
            },
            _ => panic!("Value not convertible to string: {:?}", &self),
        }
    }

    fn match_type(&self, o: &Value, state: &State) -> bool {
        return self.is_type(&o.get_type(state), state);
    }

    fn is_type(&self, t: &TypeId, state: &State) -> bool {
        return self.get_type(state) == *t;
    }

    fn get_inner_ref_mut<'a>(&'a mut self, state: &'a mut State) -> &'a mut Value {
        match self {
            Value::ValueRef(vref) => state.full_deref_valueref_mut(vref.clone()),
            _ => self,
        }
    }

    fn get_inner_ref<'a>(&'a self, state: &'a State) -> &'a Value {
        match self {
            Value::ValueRef(vref) => state.full_deref_valueref(vref),
            _ => self,
        }
    }

    fn clone_or_move_inner(self, state: &State) -> Value {
        match self {
            Value::ValueRef(vref) => state.full_deref_valueref(&vref).clone(),
            _ => self,
        }
    }
}

fn create_primitive_type(t: &PrimitiveType) -> Value {
    return Value::Type(TypeId::Primitive(*t));
}

fn create_builtin_function(f: &BuiltInFunction) -> Value {
    return Value::BuiltInFunction(f.clone());
}

fn create_null_value() -> Value {
    return Value::Null;
}

fn create_module_value(name: &StringRef) -> Value {
    return Value::Module(name.clone());
}

fn create_stackframe_ref_from_value(frame: &mut StackFrame, value: Value) -> SimpleValueRef {
    let index = frame.variables.add(value);
    SimpleValueRef::IndexedStackValueRef(IndexedStackValueRef {
        frame: frame.index,
        index,
    })
}

fn create_default_value(typeid: &TypeId) -> Value {
    match typeid {
        TypeId::Primitive(p) => match p {
            PrimitiveType::StaticStringUtf8 => {
                // TODO: Bad default value for string
                Value::Primitive(PrimitiveValue::Utf8StaticString(Utf8StaticString(0)))
            }
            PrimitiveType::Bool => Value::Primitive(PrimitiveValue::Bool(Bool(false))),
            PrimitiveType::U8 => Value::Primitive(PrimitiveValue::U8(U8(0))),
            PrimitiveType::U16 => Value::Primitive(PrimitiveValue::U16(U16(0))),
            PrimitiveType::U32 => Value::Primitive(PrimitiveValue::U32(U32(0))),
            PrimitiveType::U64 => Value::Primitive(PrimitiveValue::U64(U64(0))),
            PrimitiveType::S8 => Value::Primitive(PrimitiveValue::S8(S8(0))),
            PrimitiveType::S16 => Value::Primitive(PrimitiveValue::S16(S16(0))),
            PrimitiveType::S32 => Value::Primitive(PrimitiveValue::S32(S32(0))),
            PrimitiveType::S64 => Value::Primitive(PrimitiveValue::S64(S64(0))),
            PrimitiveType::F32 => Value::Primitive(PrimitiveValue::F32(F32(0.0))),
            PrimitiveType::F64 => Value::Primitive(PrimitiveValue::F64(F64(0.0))),
        },
        TypeId::Struct(definition) => {
            let mut fields = HashMap::new();

            for field in &definition.fields {
                fields.insert(field.0.clone(), create_default_value(&field.1));
            }

            Value::StructInstance(StructInstance {
                definition: definition.clone(),
                fields: fields,
            })
        }
        _ => panic!("No default value for typeid {:?}", typeid),
    }
}

macro_rules! as_node {
    ($ast:ident, $nodet:ident, $noderef:expr) => {
        match $ast.get_node($noderef) {
            ast::Node::$nodet(n) => n,
            n => panic!("Could not cast node {:?} to {}!", n, stringify!($nodet)),
        }
    };
}

impl<'a> TreeWalker<'a> {
    fn evaluate_integerliteral(&mut self, intlit: &ast::nodes::IntegerLiteral) -> Value {
        return Value::Primitive(PrimitiveValue::U32(U32(intlit.value as u32)));
    }

    fn evaluate_booleanliteral(&mut self, intlit: &ast::nodes::BooleanLiteral) -> Value {
        return Value::Primitive(PrimitiveValue::Bool(Bool(intlit.value)));
    }

    fn evaluate_stringliteral(&mut self, strlit: &ast::nodes::StringLiteral) -> Value {
        let id = self.state.strings.len() as u64;
        self.state.strings.push(strlit.text.clone());
        return Value::Primitive(PrimitiveValue::Utf8StaticString(Utf8StaticString(id)));
    }

    fn evaluate_builtinref(&mut self, builtin: &ast::nodes::BuiltInObjectReference) -> Value {
        match &builtin.object {
            ast::BuiltInObject::Function(o) => return create_builtin_function(o),
            ast::BuiltInObject::PrimitiveType(o) => return create_primitive_type(o),
        };
    }

    fn evaluate_subscript(&mut self, astref: &AstRef, subscript: &ast::nodes::SubScript) -> Value {
        let exprvalue = self.evaluate_expression(&from_astref(&astref, &subscript.expr));

        return match &exprvalue {
            Value::Null => panic!("Null value in subscript {:?}!", subscript),
            Value::Type(_) => panic!("Type subscripts not yet supported"),
            Value::Module(m) => {
                // TODO: What happens with the stackframe here, it should not be valid for module lookups
                // Push the referenced module around the evaluation
                let old_module = self.state.current_module;
                self.state.current_module = Some(m.key);
                let result = self.evaluate_symbol(astref, &subscript.field);
                self.state.current_module = old_module;
                result
            }
            Value::StructInstance(instance) => {
                // This is a weird scenario where the incoming is not a ref, thus an r-value
                //  This should very rarely happen, and the left side of the subscript should
                //  likely be discarded
                instance.fields[&subscript.field].clone()
            }
            Value::ValueRef(vref) => {
                match vref {
                    ValueRef::SimpleValueRef(vref) => {
                        Value::ValueRef(ValueRef::SubscriptedValueRef(SubscriptedValueRef {
                            vref: vref.clone(),
                            field: subscript.field.clone(),
                        }))
                    }
                    ValueRef::SubscriptedValueRef(_) => {
                        // To support chained subscripts, store the inner subscripted ref
                        //  on the stack so we can reference it as a simple value ref
                        Value::ValueRef(ValueRef::SubscriptedValueRef(SubscriptedValueRef {
                            vref: create_stackframe_ref_from_value(
                                self.state.stackframes.last_mut().unwrap(),
                                exprvalue,
                            ),
                            field: subscript.field.clone(),
                        }))
                    }
                }
            }
            _ => panic!("Unsupported subscript"),
        };
    }

    fn evaluate_ifexpression(
        &mut self,
        astref: &AstRef,
        ifexpr: &ast::nodes::IfExpression,
    ) -> Value {
        for branch in &ifexpr.branches {
            let condition = branch.0;
            let expr = branch.1;

            let condvalue = self.evaluate_expression(&from_astref(&astref, &condition));
            let boolvalue = match condvalue {
                Value::Primitive(PrimitiveValue::Bool(n)) => Some(n.0),
                _ => None,
            };

            assert!(
                boolvalue.is_some(),
                "if conditional expression was not a bool value",
            );

            if boolvalue.unwrap() {
                return self.evaluate_expression(&from_astref(&astref, &expr));
            }
        }

        if ifexpr.elsebranch.is_some() {
            return self.evaluate_expression(&from_astref(&astref, &ifexpr.elsebranch.unwrap()));
        };

        return create_null_value();
    }

    fn evaluate_ifstatement(&mut self, astref: &AstRef, ifstmt: &ast::nodes::IfStatement) {
        for branch in &ifstmt.branches {
            let condition = branch.0;
            let body = branch.1;

            let condvalue = self.evaluate_expression(&from_astref(&astref, &condition));
            let boolvalue = match condvalue {
                Value::Primitive(PrimitiveValue::Bool(n)) => Some(n.0),
                _ => None,
            };

            assert!(
                boolvalue.is_some(),
                "if conditional expression was not a bool value",
            );

            if boolvalue.unwrap() {
                let node = self.context.get_node(&from_astref(astref, &body));
                match node {
                    ast::Node::StatementBody(n) => {
                        return self.evaluate_statementbody(astref, &n);
                    }
                    _ => {
                        panic!("Expected statement body");
                    }
                }
            }
        }

        if ifstmt.elsebranch.is_some() {
            match self
                .context
                .get_node(&from_astref(astref, &ifstmt.elsebranch.unwrap()))
            {
                ast::Node::StatementBody(n) => {
                    self.evaluate_statementbody(astref, &n);
                }
                _ => {
                    panic!("Expected statement body");
                }
            };
        }
    }

    fn evaluate_returnstatement(&mut self, astref: &AstRef, retstmt: &ast::nodes::ReturnStatement) {
        self.state.stackframes.last_mut().unwrap().returnvalue = match retstmt.expr {
            Some(expr) => Some(self.evaluate_expression(&from_astref(&astref, &expr))),
            _ => None,
        };
    }

    fn evaluate_assignstatement(
        &mut self,
        astref: &AstRef,
        assignstmt: &ast::nodes::AssignStatement,
    ) {
        let mut lhs = self.evaluate_expression(&from_astref(&astref, &assignstmt.lhs));
        let rhs = self.evaluate_expression(&from_astref(&astref, &assignstmt.rhs));

        assert_eq!(
            lhs.get_type(&self.state),
            rhs.get_type(&self.state),
            "Mismatching types for assignment",
        );

        let rvalue = rhs.clone_or_move_inner(&self.state);
        let lvalue = lhs.get_inner_ref_mut(&mut self.state);

        *lvalue = rvalue;
    }

    fn evaluate_binaryoperation(
        &mut self,
        astref: &AstRef,
        binop: &ast::nodes::BinaryOperation,
    ) -> Value {
        let lhsval = self.evaluate_expression(&from_astref(&astref, &binop.lhs));
        let rhsval = self.evaluate_expression(&from_astref(&astref, &binop.rhs));

        assert!(
            lhsval.match_type(&rhsval, &self.state),
            "Mismatching types! {:?} vs {:?}",
            lhsval,
            rhsval
        );

        return match (
            lhsval.get_inner_ref(&self.state),
            rhsval.get_inner_ref(&self.state),
        ) {
            (Value::Primitive(l), Value::Primitive(r)) => match (l, r) {
                (PrimitiveValue::U32(l2), PrimitiveValue::U32(r2)) => {
                    perform_binop(&binop.optype, l2, r2)
                }
                (PrimitiveValue::U32(_), _) => unreachable!(),

                // TODO: Remove
                _ => unreachable!(),
            },
            _ => panic!(
                "Binary operation {:?} not supported for {:?}",
                binop.optype, lhsval
            ),
        };
    }

    fn evaluate_calloperation(
        &mut self,
        astref: &AstRef,
        callop: &ast::nodes::CallOperation,
    ) -> Value {
        let callable = self.evaluate_expression(&from_astref(&astref, &callop.expr));

        /*println!(
            "Calling: {:?}...",
            ValueDisplay {
                v: &callable,
                tw: self
            }
        );*/

        // Build arguments
        let ast = self.context.get_ast(&astref);
        let arglist = as_node!(ast, ArgumentList, &callop.arglist);
        let mut args = Vec::new();
        for arg in &arglist.args {
            let val = self.evaluate_expression(&from_astref(&astref, &arg));

            /*println!(
                "Call argument {}: {:?}",
                args.len(),
                ValueDisplay { v: &val, tw: self }
            );*/

            args.push(val);
        }

        let actual = callable.get_inner_ref(&self.state);
        let returnvalue = match actual {
            Value::Function(fref) => {
                let function = &self.state.get_module(&fref.module).functions[fref.index as usize];

                let inputparams = &function.signature.inputparams;
                assert!(args.len() == inputparams.len());

                // Check signature and build frames
                let mut frame = StackFrame {
                    index: self.state.stackframes.len(),
                    variables: VariableEnvironment::new(),
                    returnvalue: None,
                };
                for (i, arg) in args.into_iter().enumerate() {
                    assert!(
                        arg.get_type(&self.state) == inputparams[i].1,
                        "Callable argument type mismatch! Arg: {:?}, Param: {:?}",
                        arg.get_type(&self.state),
                        inputparams[i].1
                    );

                    // Copy inner value to not automatically reference other stacks
                    let arg = arg.clone_or_move_inner(&self.state);

                    /*println!(
                        "Function frame param {}: {:?}",
                        i,
                        ValueDisplay { v: &arg, tw: self }
                    );*/

                    // TODO: Does not need inner clone, probably
                    frame
                        .variables
                        .add_with_symbol(inputparams[i].0.clone(), arg);
                }

                // Call
                let fnastref = function.body;
                let fnast = self.context.get_ast(&fnastref);
                let node = as_node!(fnast, StatementBody, &fnastref.noderef);

                // TODO: This is pretty hacky, but push the module of the function before calling
                let old_module = self.state.current_module;
                self.state.current_module = Some(function.module);

                self.state.stackframes.push(frame);
                self.evaluate_statementbody(&fnastref, node);
                let result = if let Some(v) = self.state.stackframes.pop().unwrap().returnvalue {
                    Some(v.clone_or_move_inner(&self.state))
                } else {
                    None
                };

                self.state.current_module = old_module;

                result
            }
            Value::BuiltInFunction(n) => {
                // TODO
                match n {
                    BuiltInFunction::PrintFormat => {
                        assert!(args.len() > 0);
                        assert!(
                            args[0].get_type(&self.state)
                                == TypeId::Primitive(PrimitiveType::StaticStringUtf8),
                            "Built-in call argument type mismatch! Arg: {:?}, Param: {:?}",
                            args[0].get_type(&self.state),
                            TypeId::Primitive(PrimitiveType::StaticStringUtf8)
                        );

                        // TODO: Parse format string, for now, assume all args are u32s
                        let mut format_types = Vec::new();
                        for _i in 0..args.len() {
                            format_types.push(TypeId::Primitive(PrimitiveType::U32));
                        }

                        // Check signature and build frames
                        let mut strargs = Vec::new();
                        for (i, arg) in args[1..].iter().enumerate() {
                            assert!(
                                arg.get_type(&self.state) == format_types[i],
                                "Type mismatch!"
                            );
                            strargs.push(arg.to_string(&self.state));
                        }

                        let fmt = args[0].to_string(&self.state);

                        // Print
                        println!("{}", fmt.format(&strargs));
                    }
                };

                None
            }
            _ => panic!("Expression was not a function: {:?}", actual),
        };

        if let Some(v) = returnvalue {
            return v;
        } else {
            return create_null_value();
        }
    }

    fn evaluate_structliteral(
        &mut self,
        astref: &AstRef,
        sliteral: &ast::nodes::StructLiteral,
    ) -> Value {
        let mut definition = StructDefinition { fields: Vec::new() };

        for field in &sliteral.fields {
            let ast = self.context.get_ast(&astref);
            let n = as_node!(ast, StructField, field);

            let typeval: Value = self.evaluate_expression(&from_astref(&astref, &n.typeexpr));
            let typeid = match typeval.get_inner_ref(&self.state) {
                Value::Type(n) => n,
                _ => panic!(
                    "Expected Type expression for input parameter, got {:?}",
                    typeval
                ),
            };

            definition.fields.push((n.symbol.clone(), typeid.clone()));
        }

        return Value::Type(TypeId::Struct(definition));
    }

    fn evaluate_functionliteral(
        &mut self,
        astref: &AstRef,
        fnliteral: &ast::nodes::FunctionLiteral,
    ) -> Value {
        let mut signature = FunctionSignature {
            inputparams: Vec::new(),
            outputparams: Vec::new(),
        };

        for inparam in &fnliteral.inputparams {
            let ast = self.context.get_ast(&astref);
            let n = as_node!(ast, InputParameter, inparam);

            // Skip symbol for now

            let typeval: Value = self.evaluate_expression(&from_astref(&astref, &n.typeexpr));
            let typeid = match typeval.get_inner_ref(&self.state) {
                Value::Type(n) => n,
                _ => panic!(
                    "Expected Type expression for input parameter, got {:?}",
                    typeval
                ),
            };

            signature
                .inputparams
                .push((n.symbol.clone(), typeid.clone()));
        }

        for outparam in &fnliteral.outputparams {
            let ast = self.context.get_ast(&astref);
            let n = as_node!(ast, OutputParameter, outparam);

            let typeval: Value = self.evaluate_expression(&from_astref(&astref, &n.typeexpr));
            let typeid = match typeval {
                Value::Type(n) => n,
                _ => panic!(
                    "Expected Type expression for output parameter, got {:?}",
                    typeval
                ),
            };

            signature.outputparams.push(typeid);
        }

        let module = self.state.current_module.unwrap();
        let funcref = FunctionRef {
            index: self.state.get_current_module().functions.len() as u64,
            module: module,
        };
        self.state
            .get_current_module_mut()
            .functions
            .push(Function {
                module: module,
                signature: signature.clone(),
                body: from_astref(&astref, &fnliteral.body),
            });

        return Value::Function(funcref);
    }

    fn evaluate_module(&mut self, astref: &AstRef, module_node: &ast::nodes::Module) {
        let ast = self.context.get_ast(astref);

        // TODO: Fallback for unnamed modules
        // TODO: Stringref :(
        let key = module_node.symbol.stringref.key;

        // Register module globally
        let module = Module::new(
            ast.get_symbol(&module_node.symbol).unwrap().clone(),
            Some(*astref),
            self.state.current_module,
        );
        self.state.all_modules.insert(key, module);

        // And locally
        self.state.get_current_module_mut().modules.add_with_symbol(
            module_node.symbol.clone(),
            create_module_value(&StringRef { key: key }),
        );

        let old_module = self.state.current_module;
        self.state.current_module = Some(key);

        // Gah, this sucks. But a new module has no stack.
        let mut old_stackframes: Vec<StackFrame> = Vec::new();
        mem::swap(&mut old_stackframes, &mut self.state.stackframes);

        let body = as_node!(ast, StatementBody, &module_node.statementbody);
        self.evaluate_statementbody(astref, body);

        // Restore stackframes
        mem::swap(&mut old_stackframes, &mut self.state.stackframes);

        self.state.current_module = old_module;
    }

    fn evaluate_statementbody(&mut self, astref: &AstRef, body: &ast::nodes::StatementBody) {
        for s in &body.statements {
            self.evaluate_statement(&from_astref(&astref, s));
        }
    }

    fn evaluate_symbolreference(
        &mut self,
        astref: &AstRef,
        symref: &ast::nodes::SymbolReference,
    ) -> Value {
        return self.evaluate_symbol(astref, &symref.symbol);
    }

    fn evaluate_symbol(&mut self, astref: &AstRef, symbol: &ast::SymbolRef) -> Value {
        if let Some(vref) = self.state.lookup_symbol_from_stack(symbol) {
            return Value::ValueRef(vref);
        }

        panic!(
            "Could not find symbol {:?} in module {:?}",
            self.context.get_ast(astref).get_symbol(symbol).unwrap(),
            self.state.get_current_module().name
        );
    }

    fn evaluate_symboldeclaration(
        &mut self,
        astref: &AstRef,
        symdecl: &ast::nodes::SymbolDeclaration,
    ) {
        let typeval = match &symdecl.typeexpr {
            Some(n) => Some(self.evaluate_expression(&from_astref(&astref, n))),
            _ => None,
        };

        let initval = if let Some(initexpr) = &symdecl.initexpr {
            Some(self.evaluate_expression(&from_astref(astref, initexpr)))
        } else {
            None
        };

        let typevaltype = if let Some(typeval) = &typeval {
            match typeval.get_inner_ref(&self.state) {
                Value::Type(n) => Some(n),
                _ => panic!("Type expression is not a type!"),
            }
        } else {
            None
        };

        let actual_initval = initval.unwrap_or_else(|| {
            assert!(
                typeval.is_some(),
                "Cannot initialize default value without known type for symbol declaration {}",
                self.context
                    .get_ast(astref)
                    .get_symbol(&symdecl.symbol)
                    .unwrap()
            );
            create_default_value(&typevaltype.as_ref().unwrap())
        });

        if let Some(typevaltype) = typevaltype {
            let inittype = actual_initval.get_type(&self.state);
            assert_eq!(
                *typevaltype,
                inittype,
                "Mismatching types for symbol declaration {}",
                self.context
                    .get_ast(astref)
                    .get_symbol(&symdecl.symbol)
                    .unwrap()
            )
        }

        let actual_initval = actual_initval.clone_or_move_inner(&self.state);

        let symenv = if !self.state.stackframes.is_empty() {
            &mut self.state.stackframes.last_mut().unwrap().variables
        } else {
            &mut self.state.get_current_module_mut().globals
        };

        assert!(
            !symenv.has_symbol(&symdecl.symbol),
            "Symbol {} is already defined!",
            self.context
                .get_ast(astref)
                .get_symbol(&symdecl.symbol)
                .unwrap()
        );
        symenv.add_with_symbol(symdecl.symbol.clone(), actual_initval);
    }

    fn evaluate_expression(&mut self, astref: &AstRef) -> Value {
        match self.context.get_node(astref) {
            ast::Node::BuiltInObjectReference(n) => self.evaluate_builtinref(n),
            ast::Node::IntegerLiteral(n) => self.evaluate_integerliteral(n),
            ast::Node::BooleanLiteral(n) => self.evaluate_booleanliteral(n),
            ast::Node::StringLiteral(n) => self.evaluate_stringliteral(n),
            ast::Node::StructLiteral(n) => self.evaluate_structliteral(astref, n),
            ast::Node::FunctionLiteral(n) => self.evaluate_functionliteral(astref, n),
            ast::Node::SymbolReference(n) => self.evaluate_symbolreference(astref, n),
            ast::Node::CallOperation(n) => self.evaluate_calloperation(astref, n),
            ast::Node::BinaryOperation(n) => self.evaluate_binaryoperation(astref, n),
            ast::Node::IfExpression(n) => self.evaluate_ifexpression(astref, n),
            ast::Node::SubScript(n) => self.evaluate_subscript(astref, n),
            n => {
                panic!("Not an expression! Node: {:?}", ast::NodeInfo::name(n));
            }
        }
    }

    fn evaluate_statement(&mut self, astref: &AstRef) {
        return match self.context.get_node(astref) {
            ast::Node::ModuleSelfDeclaration(_) => {
                /* TODO: This should be pruned before any intepretation step */
            }
            ast::Node::Module(n) => self.evaluate_module(astref, n),
            ast::Node::StatementBody(n) => self.evaluate_statementbody(astref, n),
            ast::Node::SymbolDeclaration(n) => self.evaluate_symboldeclaration(astref, n),
            ast::Node::IfStatement(n) => self.evaluate_ifstatement(astref, n),
            ast::Node::ReturnStatement(n) => self.evaluate_returnstatement(astref, n),
            ast::Node::AssignStatement(n) => self.evaluate_assignstatement(astref, n),
            _ => {
                self.evaluate_expression(astref);
            }
        };
    }
}

impl State {
    fn get_module(&self, key: &u64) -> &Module {
        return self.all_modules.get(key).unwrap();
    }

    fn get_module_mut(&mut self, key: &u64) -> &mut Module {
        return self.all_modules.get_mut(key).unwrap();
    }

    fn get_current_module(&self) -> &Module {
        return self.get_module(&self.current_module.unwrap());
    }

    fn get_current_module_mut(&mut self) -> &mut Module {
        return self.get_module_mut(&self.current_module.unwrap());
    }

    fn get_indexed_stack_value(&self, sref: &IndexedStackValueRef) -> &Value {
        self.stackframes.last().unwrap().variables.get(sref.index)
    }

    fn get_indexed_stack_value_mut(&mut self, sref: &IndexedStackValueRef) -> &mut Value {
        self.stackframes
            .last_mut()
            .unwrap()
            .variables
            .get_mut(sref.index)
    }

    fn get_named_stack_value(&self, sref: &NamedStackValueRef) -> &Value {
        self.stackframes[sref.frame]
            .variables
            .get_from_symbol(&sref.symbol)
            .unwrap()
    }

    fn get_named_stack_value_mut(&mut self, sref: &NamedStackValueRef) -> &mut Value {
        self.stackframes
            .get_mut(sref.frame)
            .unwrap()
            .variables
            .get_from_symbol_mut(&sref.symbol)
            .unwrap()
    }

    fn get_indexed_global_value(&self, gref: &IndexedGlobalValueRef) -> &Value {
        self.get_module(&gref.module).globals.get(gref.index)
    }

    fn get_indexed_global_value_mut(&mut self, gref: &IndexedGlobalValueRef) -> &mut Value {
        self.get_module_mut(&gref.module)
            .globals
            .get_mut(gref.index)
    }

    fn get_named_global_value(&self, gref: &NamedGlobalValueRef) -> &Value {
        self.get_module(&gref.module)
            .globals
            .get_from_symbol(&gref.symbol)
            .unwrap()
    }

    fn get_named_global_value_mut(&mut self, gref: &NamedGlobalValueRef) -> &mut Value {
        self.get_module_mut(&gref.module)
            .globals
            .get_from_symbol_mut(&gref.symbol)
            .unwrap()
    }

    fn get_submodule_value(&self, smref: &SubModuleValueRef) -> &Value {
        self.get_module(&smref.module)
            .modules
            .get_from_symbol(&smref.submodule)
            .unwrap()
    }

    fn get_submodule_value_mut(&mut self, smref: &SubModuleValueRef) -> &mut Value {
        self.get_module_mut(&smref.module)
            .modules
            .get_from_symbol_mut(&smref.submodule)
            .unwrap()
    }

    fn resolve_simple_valueref(&self, vref: &SimpleValueRef) -> &Value {
        match vref {
            SimpleValueRef::IndexedStackValueRef(r) => self.get_indexed_stack_value(r),
            SimpleValueRef::NamedStackValueRef(r) => self.get_named_stack_value(r),
            SimpleValueRef::IndexedGlobalValueRef(r) => self.get_indexed_global_value(r),
            SimpleValueRef::NamedGlobalValueRef(r) => self.get_named_global_value(r),
            SimpleValueRef::SubModuleValueRef(r) => self.get_submodule_value(r),
        }
    }

    fn resolve_subscripted_valueref(&self, svref: &SubscriptedValueRef) -> &Value {
        let v = match self.resolve_simple_valueref(&svref.vref) {
            Value::ValueRef(vref) => self.full_deref_valueref(vref),
            n => n,
        };

        match v {
            Value::StructInstance(instance) => instance.fields.get(&svref.field).unwrap(),
            Value::Module(m) => {
                let vref = self.lookup_symbol_from_module(m.key, &svref.field);
                self.resolve_valueref(&vref.unwrap())
            }
            n => panic!("SubscriptedValueRef not supported on value {:?}", n),
        }
    }

    fn resolve_valueref(&self, vref: &ValueRef) -> &Value {
        match vref {
            ValueRef::SimpleValueRef(r) => self.resolve_simple_valueref(r),
            ValueRef::SubscriptedValueRef(r) => self.resolve_subscripted_valueref(r),
        }
    }

    fn resolve_simple_valueref_mut(&mut self, vref: SimpleValueRef) -> &mut Value {
        match vref {
            SimpleValueRef::IndexedStackValueRef(r) => self.get_indexed_stack_value_mut(&r),
            SimpleValueRef::NamedStackValueRef(r) => self.get_named_stack_value_mut(&r),
            SimpleValueRef::IndexedGlobalValueRef(r) => self.get_indexed_global_value_mut(&r),
            SimpleValueRef::NamedGlobalValueRef(r) => self.get_named_global_value_mut(&r),
            SimpleValueRef::SubModuleValueRef(r) => self.get_submodule_value_mut(&r),
        }
    }

    fn resolve_subscripted_valueref_mut(&mut self, svref: SubscriptedValueRef) -> &mut Value {
        // Find final ref non-mutably
        let leaf_ref = self.find_ref_to_leaf_value(&ValueRef::SimpleValueRef(svref.vref));

        // Rust seems to not be able to deal with conditionally ending lifetimes
        //  so for module lookups which requires &self, do this non-mut resolve separately
        if let Some(module_key) = match self.resolve_valueref(&leaf_ref) {
            Value::Module(m) => Some(m.key),
            _ => None,
        } {
            let vref = self.lookup_symbol_from_module(module_key, &svref.field);
            return self.resolve_valueref_mut(vref.unwrap());
        }

        // Deref it mutably
        return match self.resolve_valueref_mut(leaf_ref) {
            Value::StructInstance(instance) => instance.fields.get_mut(&svref.field).unwrap(),
            n => panic!("SubscriptedValueRef not supported on value {:?}", n),
        };
    }

    fn resolve_valueref_mut(&mut self, vref: ValueRef) -> &mut Value {
        match vref {
            ValueRef::SimpleValueRef(r) => self.resolve_simple_valueref_mut(r),
            ValueRef::SubscriptedValueRef(r) => self.resolve_subscripted_valueref_mut(r),
        }
    }

    pub fn full_deref_valueref(&self, vref: &ValueRef) -> &Value {
        /*
        {
            let x = ValueRefDisplay { vref, tw: self };
            println!("Dereffing: {:?}", x);
        }
        */

        let v = self.resolve_valueref(&vref);

        /*
        {
            let x = ValueDisplay { v, tw: self };
            println!("Got: {:?}", x);
        }
        */

        if let Value::ValueRef(vref) = v {
            self.full_deref_valueref(vref)
        } else {
            v
        }
    }

    // Needed for really awkward mut-traversials of ref-chains
    //  returns the value ref to the non-ref value at the end of the chain
    fn find_ref_to_leaf_value(&self, vref: &ValueRef) -> ValueRef {
        match self.resolve_valueref(vref) {
            Value::ValueRef(vref) => self.find_ref_to_leaf_value(vref),
            _ => vref.clone(),
        }
    }

    // Non-ref vref here because of borrow checking sadness
    fn full_deref_valueref_mut(&mut self, vref: ValueRef) -> &mut Value {
        // Find final ref non-mutably
        let leaf_ref = self.find_ref_to_leaf_value(&vref);

        return self.resolve_valueref_mut(leaf_ref);
    }

    pub fn lookup_symbol_from_module(
        &self,
        module_key: u64,
        symbol: &ast::SymbolRef,
    ) -> Option<ValueRef> {
        let mut module_key_iter = Some(module_key);

        // Check all modules up including the global module
        while let Some(module_key) = module_key_iter {
            let module = self.get_module(&module_key);

            // Module globals
            if module.globals.get_from_symbol(&symbol).is_some() {
                return Some(ValueRef::SimpleValueRef(
                    SimpleValueRef::NamedGlobalValueRef(NamedGlobalValueRef {
                        module: module_key,
                        symbol: symbol.clone(),
                    }),
                ));
            }

            // Submodules
            if module.modules.get_from_symbol(&symbol).is_some() {
                return Some(ValueRef::SimpleValueRef(SimpleValueRef::SubModuleValueRef(
                    SubModuleValueRef {
                        module: module_key,
                        submodule: symbol.clone(),
                    },
                )));
            }

            module_key_iter = module.parent;
        }
        None
    }

    pub fn lookup_symbol_from_stack(&self, symbol: &ast::SymbolRef) -> Option<ValueRef> {
        // Check stack frame first, if any
        if let Some(frame) = self.stackframes.last() {
            if frame.variables.get_from_symbol(&symbol).is_some() {
                return Some(ValueRef::SimpleValueRef(
                    SimpleValueRef::NamedStackValueRef(NamedStackValueRef {
                        frame: frame.index,
                        symbol: symbol.clone(),
                    }),
                ));
            }
        }

        self.lookup_symbol_from_module(self.current_module.unwrap(), symbol)
    }
}

impl<'a> TreeWalker<'a> {
    pub fn new(context: &'a Context) -> Self {
        TreeWalker {
            state: State {
                all_modules: HashMap::new(),
                global_module: 0,
                strings: Vec::new(),
                stackframes: Vec::new(),
                current_module: None,
            },
            context: context,
        }
    }

    pub fn interpret(&mut self) {
        let mut main: Option<AstRef> = None;

        // Register main module so other modules can use it to register themselves
        // TODO: Use 0 as main module key for now
        let module = Module::new("global".into(), None, None);
        self.state.all_modules.insert(0, module);
        self.state.global_module = 0;
        self.state.current_module = Some(self.state.global_module);

        // Evaluate all asts
        // TODO: This needs to happen in some specific order
        for ast in self.context.asts.values() {
            match ast.get_root_node().unwrap() {
                ast::Node::Module(n) => {
                    self.evaluate_module(&from_ast(&ast), &n);
                }
                ast::Node::EntryPoint(n) => {
                    // We save main for later and evaluate all modules first
                    main = Some(from_ast_and_node(&ast, &n.statementbody));
                    self.state.get_module_mut(&0).astref = main;
                }
                n => panic!("Invalid ast root node: {:?}", n),
            }
        }

        // Finally run main module
        assert!(self.state.current_module == Some(self.state.global_module));

        // Main is treated like a function, push a new stack frame
        self.state.stackframes.push(StackFrame {
            index: 0,
            variables: VariableEnvironment::new(),
            returnvalue: None,
        });

        self.evaluate_statement(&main.unwrap());
    }

    pub fn take_state(self) -> State {
        self.state
    }
}

pub fn run<'a>(main_ast: &'a ast::Ast, module_asts: &'a Vec<ast::Ast>) {
    let mut context = Context::new();

    context.asts.insert(main_ast.key, main_ast);
    for module_ast in module_asts {
        context.asts.insert(module_ast.key, module_ast);
    }

    let mut walker = TreeWalker::new(&context);
    walker.interpret();
}
