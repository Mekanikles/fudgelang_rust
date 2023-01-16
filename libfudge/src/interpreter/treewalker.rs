use crate::parser::ast;
use crate::parser::stringstore::*;
use crate::typesystem::*;

use std::collections::HashMap;

use dyn_fmt::AsStrFormatExt;

use StringKey as SymbolKey;

pub struct TreeWalker<'a> {
    ast: &'a ast::Ast,
    globals: HashMap<SymbolKey, Value>,
    functions: Vec<Function>,
    strings: Vec<String>,
    stackframes: Vec<StackFrame>,
}

struct StackFrame {
    variables: HashMap<SymbolKey, Value>,
    returnvalue: Option<Value>,
}

pub struct Function {
    signature: FunctionSignature,
    body: ast::NodeRef,
}

use u64 as FunctionRef;

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

/*
return match (&lhsval, &rhsval) {
    (Value::Primitive(l), Value::Primitive(r)) => match (l, r) {
        (PrimitiveValue::U32(l2), PrimitiveValue::U32(r2)) => perform_binop(&binop.optype, l2, r2),
        (PrimitiveValue::U32(_), _) => unreachable!(),

        // TODO: Remove
        _ => unreachable!(),
    }
    _ => panic!("Binary operation {:?} not supported for {:?}", binop.optype, lhsval),
};*/

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Type(TypeId),
    Primitive(PrimitiveValue),
    BuiltInFunction(BuiltInFunction),
    Function(u64),
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
            Value::Function(v) => TypeId::Function(tw.functions[*v as usize].signature.clone()),
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

macro_rules! as_node {
    ($self:ident, $nodet:ident, $noderef:expr) => {
        match $self.ast.get_node($noderef) {
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

    fn evaluate_ifexpression(&mut self, ifexpr: &ast::nodes::IfExpression) -> Value {
        let condition = &ifexpr.condexpr;
        let trueexpr = &ifexpr.trueexpr;
        let falseexpr = &ifexpr.falseexpr;

        let condvalue = self.evaluate_expression(&condition);
        let boolvalue = match condvalue {
            Value::Primitive(PrimitiveValue::Bool(n)) => Some(n.0),
            _ => None,
        };

        assert!(
            boolvalue.is_some(),
            "if conditional expression was not a bool value",
        );

        return match boolvalue.unwrap() {
            true => self.evaluate_expression(trueexpr),
            false => match falseexpr {
                Some(n) => self.evaluate_expression(n),
                None => {
                    assert!(false, "Need to implement 'none'");
                    self.evaluate_expression(trueexpr)
                }
            },
        };
    }

    fn evaluate_ifstatement(&mut self, ifstmt: &ast::nodes::IfStatement) {
        for branch in &ifstmt.branches {
            let condition = branch.0;
            let body = branch.1;

            let condvalue = self.evaluate_expression(&condition);
            let boolvalue = match condvalue {
                Value::Primitive(PrimitiveValue::Bool(n)) => Some(n.0),
                _ => None,
            };

            assert!(
                boolvalue.is_some(),
                "if conditional expression was not a bool value",
            );

            if boolvalue.unwrap() {
                match self.ast.get_node(&body) {
                    ast::Node::StatementBody(n) => {
                        return self.evaluate_statementbody(&n);
                    }
                    _ => {
                        panic!("Expected statement body");
                    }
                }
            }
        }

        if ifstmt.elsebranch.is_some() {
            match self.ast.get_node(&ifstmt.elsebranch.unwrap()) {
                ast::Node::StatementBody(n) => {
                    self.evaluate_statementbody(&n);
                }
                _ => {
                    panic!("Expected statement body");
                }
            };
        }
    }

    fn evaluate_returnstatement(&mut self, retstmt: &ast::nodes::ReturnStatement) {
        self.stackframes.last_mut().unwrap().returnvalue = match retstmt.expr {
            Some(expr) => Some(self.evaluate_expression(&expr)),
            _ => None,
        };
    }

    fn evaluate_binaryoperation(&mut self, binop: &ast::nodes::BinaryOperation) -> Value {
        let lhsval = self.evaluate_expression(&binop.lhs);
        let rhsval = self.evaluate_expression(&binop.rhs);

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

    fn evaluate_calloperation(&mut self, callop: &ast::nodes::CallOperation) -> Value {
        let callable = self.evaluate_expression(&callop.expr);

        // Build arguments
        let arglist = as_node!(self, ArgumentList, &callop.arglist);
        let mut args = Vec::new();
        for arg in &arglist.args {
            let val = self.evaluate_expression(&arg);
            args.push(val);
        }

        let returnvalue = match &callable {
            Value::Function(n) => {
                let funcindex = *n as usize;

                let inputparams = &self.functions[funcindex].signature.inputparams;
                assert!(args.len() == inputparams.len());

                // Check signature and build frames
                let mut frame = StackFrame {
                    variables: HashMap::new(),
                    returnvalue: None,
                };
                for (i, arg) in args.iter().enumerate() {
                    assert!(arg.get_type(&self) == inputparams[i].1, "Type mismatch!");
                    frame.variables.insert(inputparams[i].0, arg.clone());
                }

                // Call
                self.stackframes.push(frame);
                self.evaluate_statementbody(as_node!(
                    self,
                    StatementBody,
                    &self.functions[funcindex].body
                ));
                self.stackframes.pop().unwrap().returnvalue.clone()
            }
            Value::BuiltInFunction(n) => {
                // TODO
                match n {
                    BuiltInFunction::PrintFormat => {
                        assert!(args.len() > 0);
                        assert!(
                            args[0].get_type(&self)
                                == TypeId::Primitive(PrimitiveType::StaticStringUtf8),
                            "Type mismatch!"
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

    fn evaluate_functionliteral(&mut self, fnliteral: &ast::nodes::FunctionLiteral) -> Value {
        let mut signature = FunctionSignature {
            inputparams: Vec::new(),
            outputparams: Vec::new(),
        };

        for inparam in &fnliteral.inputparams {
            let n = as_node!(self, InputParameter, inparam);

            // Skip symbol for now

            let typeval: Value = self.evaluate_expression(&n.typeexpr);
            let typeid = match typeval {
                Value::Type(n) => n,
                _ => panic!(
                    "Expected Type expression for input parameter, got {:?}",
                    typeval
                ),
            };

            signature.inputparams.push((n.symbol.key, typeid));
        }

        for outparam in &fnliteral.outputparams {
            let n = as_node!(self, OutputParameter, outparam);

            let typeval: Value = self.evaluate_expression(&n.typeexpr);
            let typeid = match typeval {
                Value::Type(n) => n,
                _ => panic!(
                    "Expected Type expression for output parameter, got {:?}",
                    typeval
                ),
            };

            signature.outputparams.push(typeid);
        }

        let funcid: FunctionRef = self.functions.len() as u64;
        self.functions.push(Function {
            signature: signature.clone(),
            body: fnliteral.body,
        });

        return Value::Function(funcid);
    }

    fn evaluate_statementbody(&mut self, body: &ast::nodes::StatementBody) {
        for s in &body.statements {
            self.evaluate_statement(s);
        }
    }

    fn evaluate_symbolreference(&mut self, symref: &ast::nodes::SymbolReference) -> Value {
        if let Some(v) = self
            .stackframes
            .last()
            .unwrap()
            .variables
            .get(&symref.symbol.key)
        {
            return v.clone();
        }
        return self.globals[&symref.symbol.key].clone();
    }

    fn evaluate_symboldeclaration(&mut self, symdecl: &ast::nodes::SymbolDeclaration) {
        assert!(
            !self.globals.contains_key(&symdecl.symbol.key),
            "Symbol {} is already defined!",
            self.ast.get_symbol(&symdecl.symbol).unwrap()
        );

        // TODO: check type
        let _typeval: Option<Value> = match &symdecl.typeexpr {
            Some(n) => Some(self.evaluate_expression(n)),
            _ => None,
        };

        assert!(
            !symdecl.typeexpr.is_some(),
            "Type definitions in declarations not yet supported!"
        );

        let initval = self.evaluate_expression(&symdecl.initexpr);
        self.globals.insert(symdecl.symbol.key, initval);
    }
}

impl<'a> TreeWalker<'a> {
    pub fn new(ast: &'a ast::Ast) -> Self {
        // Make sure we always have a stackframe
        let frame = StackFrame {
            variables: HashMap::new(),
            returnvalue: None,
        };

        TreeWalker {
            ast: ast,
            globals: HashMap::new(),
            functions: Vec::new(),
            strings: Vec::new(),
            stackframes: vec![frame],
        }
    }

    pub fn evaluate_expression(&mut self, node: &ast::NodeRef) -> Value {
        match self.ast.get_node(node) {
            ast::Node::BuiltInObjectReference(n) => self.evaluate_builtinref(n),
            ast::Node::IntegerLiteral(n) => self.evaluate_integerliteral(n),
            ast::Node::BooleanLiteral(n) => self.evaluate_booleanliteral(n),
            ast::Node::StringLiteral(n) => self.evaluate_stringliteral(n),
            ast::Node::FunctionLiteral(n) => self.evaluate_functionliteral(n),
            ast::Node::SymbolReference(n) => self.evaluate_symbolreference(n),
            ast::Node::CallOperation(n) => self.evaluate_calloperation(n),
            ast::Node::BinaryOperation(n) => self.evaluate_binaryoperation(n),
            ast::Node::IfExpression(n) => self.evaluate_ifexpression(n),
            n => {
                panic!("Not an expression! Node: {:?}", ast::NodeInfo::name(n));
            }
        }
    }

    pub fn evaluate_statement(&mut self, node: &ast::NodeRef) {
        return match self.ast.get_node(node) {
            ast::Node::ModuleFragment(n) => {
                self.evaluate_statementbody(as_node!(self, StatementBody, &n.statementbody))
            }
            ast::Node::StatementBody(n) => self.evaluate_statementbody(n),
            ast::Node::SymbolDeclaration(n) => self.evaluate_symboldeclaration(n),
            ast::Node::IfStatement(n) => self.evaluate_ifstatement(n),
            ast::Node::ReturnStatement(n) => self.evaluate_returnstatement(n),
            _ => {
                self.evaluate_expression(node);
            }
        };
    }

    pub fn interpret(&mut self) {
        if let Some(n) = self.ast.get_root() {
            self.evaluate_statement(&n);
        }
    }
}