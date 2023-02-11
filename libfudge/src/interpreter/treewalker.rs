use crate::parser::ast;
use crate::parser::stringstore::*;
use crate::typesystem::*;

use std::collections::HashMap;
use std::mem;

use dyn_fmt::AsStrFormatExt;

use StringKey as SymbolKey;

#[derive(Debug, Copy, Clone, PartialEq)]
struct AstRef {
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

struct Context<'a> {
    asts: HashMap<ast::AstKey, &'a ast::Ast>,
}

#[derive(Debug)]
struct SymbolEnvironment {
    pub variables: HashMap<SymbolKey, Value>,
}

impl SymbolEnvironment {
    fn new() -> SymbolEnvironment {
        SymbolEnvironment {
            variables: HashMap::new(),
        }
    }

    fn insert(&mut self, s: SymbolKey, v: Value) -> Option<Value> {
        self.variables.insert(s, v)
    }

    fn get(&self, s: &SymbolKey) -> Option<&Value> {
        self.variables.get(s)
    }
    fn contains_key(&self, s: &SymbolKey) -> bool {
        self.variables.contains_key(s)
    }
}

struct Module {
    pub name: String,
    pub parent: Option<u64>,
    pub globals: SymbolEnvironment,
    pub functions: Vec<Function>,
    pub modules: Vec<u64>,
}

impl Module {
    fn new(name: String, parent: Option<u64>) -> Module {
        Module {
            name: name,
            parent: parent,
            globals: SymbolEnvironment::new(),
            functions: Vec::new(),
            modules: Vec::new(),
        }
    }
}

impl<'a> Context<'a> {
    fn new() -> Context<'a> {
        Context {
            asts: HashMap::new(),
        }
    }

    fn get_ast(&self, astref: &AstRef) -> &ast::Ast {
        return self.asts[&astref.ast_key];
    }

    fn get_node(&self, astref: &AstRef) -> &ast::Node {
        return self.asts[&astref.ast_key].get_node(&astref.noderef);
    }
}

pub struct TreeWalker<'a> {
    all_modules: HashMap<u64, Module>,
    global_module: u64,
    strings: Vec<String>,
    stackframes: Vec<StackFrame>,
    current_module: Option<u64>,
    context: &'a Context<'a>,
}

#[derive(Debug)]
struct StackFrame {
    variables: SymbolEnvironment,
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

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Type(TypeId),
    Primitive(PrimitiveValue),
    BuiltInFunction(BuiltInFunction),
    Function(FunctionRef),
    Module(StringRef),
}

impl Value {
    fn get_type(&self, tw: &TreeWalker) -> TypeId {
        match self {
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
                tw.get_module(&fref.module).functions[fref.index as usize]
                    .signature
                    .clone(),
            ),
            Value::Module(_) => TypeId::Module,
        }
    }

    fn to_string(&self, tw: &TreeWalker) -> String {
        match self {
            Value::Primitive(p) => match p {
                PrimitiveValue::Utf8StaticString(v) => tw.strings[v.0 as usize].clone(),
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

    fn match_type(&self, o: &Value, tw: &TreeWalker) -> bool {
        return self.is_type(&o.get_type(tw), tw);
    }

    fn is_type(&self, t: &TypeId, tw: &TreeWalker) -> bool {
        return self.get_type(tw) == *t;
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
        let id = self.strings.len() as u64;
        self.strings.push(strlit.text.clone());
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

        return match exprvalue {
            Value::Null => panic!("Null value in subscript {:?}!", subscript),
            Value::Type(_) => panic!("Type subscripts not yet supported"),
            Value::Module(m) => {
                // Push the referenced module around the evaluation
                let old_module = self.current_module;
                self.current_module = Some(m.key);
                let result = self.evaluate_symbol(astref, &subscript.field);
                self.current_module = old_module;
                result
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
        self.stackframes.last_mut().unwrap().returnvalue = match retstmt.expr {
            Some(expr) => Some(self.evaluate_expression(&from_astref(&astref, &expr))),
            _ => None,
        };
    }

    fn evaluate_binaryoperation(
        &mut self,
        astref: &AstRef,
        binop: &ast::nodes::BinaryOperation,
    ) -> Value {
        let lhsval = self.evaluate_expression(&from_astref(&astref, &binop.lhs));
        let rhsval = self.evaluate_expression(&from_astref(&astref, &binop.rhs));

        assert!(
            lhsval.match_type(&rhsval, &self),
            "Mismatching types! {:?} vs {:?}",
            lhsval,
            rhsval
        );

        return match (&lhsval, &rhsval) {
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

        // Build arguments
        let ast = self.context.get_ast(&astref);
        let arglist = as_node!(ast, ArgumentList, &callop.arglist);
        let mut args = Vec::new();
        for arg in &arglist.args {
            let val = self.evaluate_expression(&from_astref(&astref, &arg));
            args.push(val);
        }

        let returnvalue = match &callable {
            Value::Function(fref) => {
                let function = &self.get_module(&fref.module).functions[fref.index as usize];

                let inputparams = &function.signature.inputparams;
                assert!(args.len() == inputparams.len());

                // Check signature and build frames
                let mut frame = StackFrame {
                    variables: SymbolEnvironment::new(),
                    returnvalue: None,
                };
                for (i, arg) in args.iter().enumerate() {
                    assert!(
                        arg.get_type(&self) == inputparams[i].1,
                        "Callable argument type mismatch! Arg: {:?}, Param: {:?}",
                        arg.get_type(&self),
                        inputparams[i].1
                    );
                    frame.variables.insert(inputparams[i].0, arg.clone());
                }

                // Call
                let fnastref = function.body;
                let fnast = self.context.get_ast(&fnastref);
                let node = as_node!(fnast, StatementBody, &fnastref.noderef);

                // TODO: This is pretty hacky, but push the module of the function before calling
                let old_module = self.current_module;
                self.current_module = Some(function.module);

                self.stackframes.push(frame);
                self.evaluate_statementbody(astref, node);
                let result = self.stackframes.pop().unwrap().returnvalue.clone();

                self.current_module = old_module;

                result
            }
            Value::BuiltInFunction(n) => {
                // TODO
                match n {
                    BuiltInFunction::PrintFormat => {
                        assert!(args.len() > 0);
                        assert!(
                            args[0].get_type(&self)
                                == TypeId::Primitive(PrimitiveType::StaticStringUtf8),
                            "Built-in call argument type mismatch! Arg: {:?}, Param: {:?}",
                            args[0].get_type(&self),
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
                            assert!(arg.get_type(&self) == format_types[i], "Type mismatch!");
                            strargs.push(arg.to_string(self));
                        }

                        let fmt = args[0].to_string(self);

                        // Print
                        println!("{}", fmt.format(&strargs));
                    }
                };

                None
            }
            _ => panic!("Expression was not a function: {:?}", callable),
        };

        if let Some(v) = returnvalue {
            return v;
        } else {
            return create_null_value();
        }
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
            let key = n.symbol.key;

            // Skip symbol for now

            let typeval: Value = self.evaluate_expression(&from_astref(&astref, &n.typeexpr));
            let typeid = match typeval {
                Value::Type(n) => n,
                _ => panic!(
                    "Expected Type expression for input parameter, got {:?}",
                    typeval
                ),
            };

            signature.inputparams.push((key, typeid));
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

        let module = self.current_module.unwrap();
        let funcref = FunctionRef {
            index: self.get_current_module().functions.len() as u64,
            module: module,
        };
        self.get_current_module_mut().functions.push(Function {
            module: module,
            signature: signature.clone(),
            body: from_astref(&astref, &fnliteral.body),
        });

        return Value::Function(funcref);
    }

    fn evaluate_module(&mut self, astref: &AstRef, module_node: &ast::nodes::Module) {
        let ast = self.context.get_ast(astref);

        // TODO: Fallback for unnamed modules
        let key = module_node.symbol.key;

        // Register module globally
        let module = Module::new(
            ast.get_symbol(&module_node.symbol).unwrap().clone(),
            self.current_module,
        );
        self.all_modules.insert(key, module);

        // And locally
        self.get_current_module_mut().modules.push(key);

        let old_module = self.current_module;
        self.current_module = Some(key);

        // Gah, this sucks. But a new module has no stack.
        let mut old_stackframes: Vec<StackFrame> = Vec::new();
        mem::swap(&mut old_stackframes, &mut self.stackframes);

        let body = as_node!(ast, StatementBody, &module_node.statementbody);
        self.evaluate_statementbody(astref, body);

        // Restore stackframes
        mem::swap(&mut old_stackframes, &mut self.stackframes);

        self.current_module = old_module;
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

    fn evaluate_symbol(&mut self, astref: &AstRef, symbol: &StringRef) -> Value {
        // Check stack frame first, if any
        if let Some(frame) = self.stackframes.last() {
            if let Some(v) = frame.variables.get(&symbol.key) {
                return v.clone();
            }
        }

        let mut module_key = self.current_module;

        // Check all modules up including the global module
        while let Some(module) = module_key {
            let module = self.get_module(&module);

            // Module globals
            if let Some(v) = module.globals.get(&symbol.key) {
                return v.clone();
            }

            // Submodules
            if let Some(_) = module.modules.iter().find(|&module| *module == symbol.key) {
                return create_module_value(symbol);
            }

            module_key = module.parent;
        }

        panic!(
            "Could not find symbol {:?} in module {:?}",
            self.context.get_ast(astref).get_symbol(symbol).unwrap(),
            self.get_current_module().name
        );
    }

    fn evaluate_symboldeclaration(
        &mut self,
        astref: &AstRef,
        symdecl: &ast::nodes::SymbolDeclaration,
    ) {
        // TODO: check type
        let _typeval: Option<Value> = match &symdecl.typeexpr {
            Some(n) => Some(self.evaluate_expression(&from_astref(&astref, n))),
            _ => None,
        };

        assert!(
            !symdecl.typeexpr.is_some(),
            "Type definitions in declarations not yet supported!"
        );

        let initval = self.evaluate_expression(&from_astref(astref, &symdecl.initexpr));

        let symenv = if !self.stackframes.is_empty() {
            &mut self.stackframes.last_mut().unwrap().variables
        } else {
            &mut self.get_current_module_mut().globals
        };

        assert!(
            !symenv.contains_key(&symdecl.symbol.key),
            "Symbol {} is already defined!",
            self.context
                .get_ast(astref)
                .get_symbol(&symdecl.symbol)
                .unwrap()
        );
        symenv.insert(symdecl.symbol.key, initval);
    }

    fn evaluate_expression(&mut self, astref: &AstRef) -> Value {
        match self.context.get_node(astref) {
            ast::Node::BuiltInObjectReference(n) => self.evaluate_builtinref(n),
            ast::Node::IntegerLiteral(n) => self.evaluate_integerliteral(n),
            ast::Node::BooleanLiteral(n) => self.evaluate_booleanliteral(n),
            ast::Node::StringLiteral(n) => self.evaluate_stringliteral(n),
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
            _ => {
                self.evaluate_expression(astref);
            }
        };
    }
}

impl<'a> TreeWalker<'a> {
    fn new(context: &'a Context) -> Self {
        TreeWalker {
            all_modules: HashMap::new(),
            global_module: 0,
            strings: Vec::new(),
            stackframes: Vec::new(),
            context: context,
            current_module: None,
        }
    }

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

    pub fn interpret(&mut self) {
        let mut main: Option<AstRef> = None;

        // Register main module so other modules can use it to register themselves
        // TODO: Use 0 as main module key for now
        let module = Module::new("global".into(), None);
        self.all_modules.insert(0, module);
        self.global_module = 0;
        self.current_module = Some(self.global_module);

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
                }
                n => panic!("Invalid ast root node: {:?}", n),
            }
        }

        // Finally run main module
        assert!(self.current_module == Some(self.global_module));

        // Main is treated like a function, push a new stack frame
        self.stackframes.push(StackFrame {
            variables: SymbolEnvironment::new(),
            returnvalue: None,
        });

        self.evaluate_statement(&main.unwrap());
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
