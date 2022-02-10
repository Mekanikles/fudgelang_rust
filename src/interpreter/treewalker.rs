use crate::parser::ast;
use std::collections::HashMap;

use dyn_fmt::AsStrFormatExt;

pub struct TreeWalker<'a> {
    ast : &'a ast::Ast,
    globals: HashMap<ast::SymbolKey, Value>,
    functions: Vec<Function>,
    strings: Vec<String>,
    stackframes: Vec<StackFrame>,
}

struct StackFrame
{
    variables: HashMap<ast::SymbolKey, Value>,
    returnvalue: Option<Value>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionSignature
{
    inputparams: Vec<(ast::SymbolKey, TypeId)>,
    outputparams: Vec<TypeId>,
}

pub struct Function
{
    signature: FunctionSignature,
    body: ast::NodeRef,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveType {
    Utf8StaticString,
    U32,
}

use u64 as FunctionRef;

#[derive(Debug, Clone, PartialEq)]
pub enum TypeId {
    Null,
    Type,
    Primitive(PrimitiveType),
    BuiltInFunction(ast::BuiltInFunction),
    Function(FunctionSignature),
}

#[derive(Debug, Clone)]
pub enum Data {
    Null,
    Type(TypeId),
    BuiltInFunction(ast::BuiltInFunction),
    Id(u64),
    Chunk(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct Value {
    typeid: TypeId,
    data: Data,
}

impl Value
{
    fn new_id(typeid: TypeId, id: u64) -> Value {
        return Value {
            typeid: typeid,
            data: Data::Id(id),
        };
    }

    fn as_id(&self) -> u64 {
        return match self.data {
            Data::Id(n) => n,
            _ => panic!("Data was not an id! {:?}", self.data)
        };
    }

    fn match_type(&self, o: &Value) -> bool {
        return self.typeid == o.typeid;
    }

    fn is_type(&self, t: &TypeId) -> bool {
        return self.typeid == *t;
    }
}

fn create_primitive_type(t: &ast::BuiltInPrimitiveType) -> Value {
    return Value {
        typeid: TypeId::Type,
        data: Data::Type(TypeId::Primitive(match t {
            ast::BuiltInPrimitiveType::U32 => PrimitiveType::U32
        })),
    };
}

fn create_builtin_function(f: &ast::BuiltInFunction) -> Value {
    return Value {
        typeid: TypeId::BuiltInFunction(f.clone()),
        data: Data::BuiltInFunction(f.clone()),
    };
}

fn create_null_value() -> Value {
    return Value {
        typeid: TypeId::Null,
        data: Data::Null,
    };
}

macro_rules! as_node {
    ($self:ident, $nodet:ident, $noderef:expr) => {
        match $self.ast.get_node($noderef) {
            ast::Node::$nodet(n) => n,
            n => panic!("Could not cast node {:?} to {}!", n, stringify!($nodet))
        }
    }
}

impl<'a> TreeWalker<'a> {
    fn evaluate_integerliteral(&mut self, intlit: &ast::IntegerLiteral) -> Value {
        return Value::new_id(TypeId::Primitive(PrimitiveType::U32), intlit.value);
    }

    fn evaluate_stringliteral(&mut self, strlit: &ast::StringLiteral) -> Value {
        let id = self.strings.len() as u64;
        self.strings.push(strlit.text.clone());
        return Value::new_id(TypeId::Primitive(PrimitiveType::Utf8StaticString), id);
    }

    fn evaluate_builtinref(&mut self, builtin: &ast::BuiltInObjectReference) -> Value {
        match &builtin.object {
            ast::BuiltInObject::Function(o) => return create_builtin_function(o),
            ast::BuiltInObject::PrimitiveType(o) => return create_primitive_type(o),
        };
    }

    fn evaluate_returnstatement(&mut self, retstmt: &ast::ReturnStatement) {
        let exprval = self.evaluate_expression(&retstmt.expr);
        self.stackframes.last_mut().unwrap().returnvalue = Some(exprval);
    }

    fn evaluate_binaryoperation(&mut self, binop: &ast::BinaryOperation) -> Value {
        let lhsval = self.evaluate_expression(&binop.lhs);
        let rhsval = self.evaluate_expression(&binop.rhs);

        // TODO: Only add of integers is supported atm
        assert!(lhsval.match_type(&rhsval), "Mismatching types! {:?} vs {:?}", lhsval, rhsval);
        assert!(lhsval.is_type(&TypeId::Primitive(PrimitiveType::U32)), "Can only handle integer values!, was: {:?}", lhsval);

        return Value::new_id(TypeId::Primitive(
            PrimitiveType::U32), 
            lhsval.as_id() + rhsval.as_id());
    }

    fn evaluate_calloperation(&mut self, callop: &ast::CallOperation) -> Value {
        let callable = self.evaluate_expression(&callop.expr);

        // Build arguments
        let arglist = as_node!(self, ArgumentList, &callop.arglist);
        let mut args = Vec::new();
        for arg in &arglist.args {
            let val = self.evaluate_expression(&arg);
            args.push(val);
        }

        let returnvalue = match &callable.typeid {
            TypeId::Function(n) => {
                let funcindex = callable.as_id() as usize;

                let inputparams = &self.functions[funcindex].signature.inputparams;  
                assert!(args.len() == inputparams.len());
        
                // Check signature and build frames
                let mut frame = StackFrame { variables: HashMap::new(), returnvalue: None };
                for (i, arg) in args.iter().enumerate() {
                    assert!(arg.typeid == inputparams[i].1, "Type mismatch!");
                    frame.variables.insert(inputparams[i].0, arg.clone());
                }
        
                // Call
                self.stackframes.push(frame);
                self.evaluate_statementbody(as_node!(self, StatementBody, &self.functions[funcindex].body));
                self.stackframes.pop().unwrap().returnvalue.clone()
            },
            TypeId::BuiltInFunction(n) => {
                // TODO
                match n {
                    ast::BuiltInFunction::PrintFormat => {
                        assert!(args.len() > 0);
                        assert!(args[0].typeid == TypeId::Primitive(PrimitiveType::Utf8StaticString), "Type mismatch!");

                        // TODO: Parse format string, for now, assume 1 int
                        let mut format_types = Vec::new();
                        format_types.push(TypeId::Primitive(PrimitiveType::U32));

                        // Check signature and build frames
                        let mut strargs = Vec::new();
                        for (i, arg) in args[1..].iter().enumerate() {
                            assert!(arg.typeid == format_types[i], "Type mismatch!");
                            strargs.push(arg.as_id().to_string());
                        }
                        
                        let fmt = &self.strings[args[0].as_id() as usize];

                        // Print
                        println!("{}", fmt.format(&strargs));
                    }
                };

                None
            }
            _ => panic!("Expression was not a function: {:?}", callable)
        };

        if let Some(v) = returnvalue {
            return v;
        }
        else
        {
            return create_null_value();
        }
    }

    fn evaluate_functionliteral(&mut self, fnliteral: &ast::FunctionLiteral) -> Value {
        let mut signature = FunctionSignature {
            inputparams: Vec::new(),
            outputparams: Vec::new(),
        };
        
        for inparam in &fnliteral.inputparams {
            let n = as_node!(self, InputParameter, inparam);

            // Skip symbol for now

            let typeval: Value = self.evaluate_expression(&n.typeexpr);
            let typeid = match typeval.data {
                Data::Type(n) => n,
                _ => panic!("Expected Type expression for input parameter, got {:?}", typeval)
            };

            signature.inputparams.push((n.symbol.key, typeid));
        }

        for outparam in &fnliteral.outputparams {
            let n = as_node!(self, OutputParameter, outparam);

            let typeval: Value = self.evaluate_expression(&n.typeexpr);
            let typeid = match typeval.data {
                Data::Type(n) => n,
                _ => panic!("Expected Type expression for output parameter, got {:?}", typeval)
            };

            signature.outputparams.push(typeid);
        }

        let funcid: FunctionRef = self.functions.len() as u64;
        self.functions.push(Function { 
            signature: signature.clone(),
            body: fnliteral.body,
        });
       
        return Value::new_id(TypeId::Function(signature), funcid);
    }

    fn evaluate_statementbody(&mut self, body: &ast::StatementBody) {
        for s in &body.statements {
            self.evaluate_statement(s);
        }
    }

    fn evaluate_symbolreference(&mut self, symref: &ast::SymbolReference) -> Value {
        if let Some(v) = self.stackframes.last().unwrap().variables.get(&symref.symbol.key) {
            return v.clone();
        }
        return self.globals[&symref.symbol.key].clone();
    }

    fn evaluate_symboldeclaration(&mut self, symdecl: &ast::SymbolDeclaration) {
        assert!(!self.globals.contains_key(&symdecl.symbol.key), "Symbol {} is already defined!", self.ast.get_symbol(&symdecl.symbol).unwrap());

        let typeval: Option<Value> = match(&symdecl.typeexpr) {
            Some(n) => Some(self.evaluate_expression(n)),
            _ => None
        };

        assert!(!symdecl.typeexpr.is_some(), "Type definitions in declarations not yet supported!");

        let initval = self.evaluate_expression(&symdecl.initexpr);
        self.globals.insert(symdecl.symbol.key, initval);
    }
}

impl<'a> TreeWalker<'a> {
    pub fn new(ast : &'a ast::Ast) -> Self {
        // Make sure we always have a stackframe
        let frame = StackFrame { variables: HashMap::new(), returnvalue: None };

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
            ast::Node::StringLiteral(n) => self.evaluate_stringliteral(n),
            ast::Node::FunctionLiteral(n) => self.evaluate_functionliteral(n),
            ast::Node::SymbolReference(n) => self.evaluate_symbolreference(n),
            ast::Node::CallOperation(n) => self.evaluate_calloperation(n),
            ast::Node::BinaryOperation(n) => self.evaluate_binaryoperation(n),
            n => { panic!("Not an expression! Node: {:?}", ast::NamedNode::name(n));}
        }
    }

    pub fn evaluate_statement(&mut self, node : &ast::NodeRef) {
        return match self.ast.get_node(node) {
            ast::Node::ModuleFragment(n) => self.evaluate_statementbody(as_node!(self, StatementBody, &n.statementbody)),
            ast::Node::StatementBody(n) => self.evaluate_statementbody(n),
            ast::Node::SymbolDeclaration(n) => self.evaluate_symboldeclaration(n),
            ast::Node::ReturnStatement(n) => self.evaluate_returnstatement(n),
            _ => { self.evaluate_expression(node); },
        }
    }

    pub fn interpret(&mut self) {
        if let Some(n) = self.ast.get_root() {
            self.evaluate_statement(&n);
        }
    }
}