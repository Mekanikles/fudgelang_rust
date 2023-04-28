use std::collections::HashMap;

use crate::typesystem::*;
use crate::utils::objectstore::*;
pub use crate::utils::*;

use crate::shared::BinaryOperationType;

pub type ModuleStore = IndexedObjectStore<Module>;
pub type FunctionStore = IndexedObjectStore<Function>;
pub type ExpressionStore = IndexedObjectStore<Expression>;
pub type SymbolScopeStore = IndexedObjectStore<SymbolScope>;
pub type StatementBodyStore = IndexedObjectStore<StatementBody>;
// Ffs rust, cannot use ModuleStore::Key here...
//  Also, these are weak type aliases
pub type ModuleKey = usize;
pub type FunctionKey = usize;
pub type ExpressionKey = usize;
pub type SymbolScopeKey = usize;
pub type StatementBodyKey = usize;

#[derive(PartialEq, Eq, Debug)]
pub struct SymbolDeclaration {
    pub symbol: String,
    pub typeexpr: Option<ExpressionKey>,
}

impl SymbolDeclaration {
    pub fn new(symbol: String, typeexpr: Option<ExpressionKey>) -> Self {
        Self { symbol, typeexpr }
    }
}

pub type SymbolDeclarationStore = HashedObjectStore<StringKey, SymbolDeclaration>;
pub type SymbolKey = StringKey;

impl objectstore::HashedStoreKey<SymbolDeclaration> for StringKey {
    fn from_obj(object: &SymbolDeclaration) -> Self {
        StringKey::from_str(object.symbol.as_str())
    }
}

#[derive(Debug)]
pub struct UnresolvedSymbolReference {
    pub symbol: String,
}

#[derive(Debug)]
pub struct ResolvedSymbolReference {
    pub scope: SymbolScopeKey,
    pub symbol: SymbolKey,
}

#[derive(Debug)]
pub enum SymbolReference {
    ResolvedReference(ResolvedSymbolReference),
    UnresolvedReference(UnresolvedSymbolReference),
}

pub type SymbolReferenceStore = IndexedObjectStore<SymbolReference>;
pub type SymbolReferenceKey = usize;

// TODO: This is awkward, expressions should know their local context
#[derive(Debug)]
pub struct SymbolReferenceRef {
    pub scope: SymbolScopeKey,
    pub refkey: SymbolReferenceKey,
}

#[derive(Debug)]
pub struct SymbolScope {
    pub declarations: SymbolDeclarationStore,
    pub references: SymbolReferenceStore,
    pub definitions: HashMap<SymbolKey, ExpressionKey>,
}

impl SymbolScope {
    pub fn new() -> Self {
        Self {
            declarations: SymbolDeclarationStore::new(),
            references: SymbolReferenceStore::new(),
            definitions: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Store {
    pub modules: ModuleStore,
    pub functions: FunctionStore,
    pub expressions: ExpressionStore,
    pub symbolscopes: SymbolScopeStore,
    pub statementbodies: StatementBodyStore,
}

impl Store {
    pub fn new() -> Self {
        Self {
            modules: ModuleStore::new(),
            functions: FunctionStore::new(),
            expressions: ExpressionStore::new(),
            symbolscopes: SymbolScopeStore::new(),
            statementbodies: StatementBodyStore::new(),
        }
    }
}

#[derive(Debug)]
pub struct Asg {
    pub store: Store,
    pub global_module: ModuleKey,
    pub main: FunctionKey,
}

impl Asg {
    pub fn new() -> Self {
        let mut store = Store::new();

        let module = Module::new(&mut store, "global".into(), None);
        let global_module = store.modules.add(module);
        // Note: main should not be available for symbol lookup, so don't add it to any module
        let main_func = Function::new("main".into(), global_module);
        let main = store.functions.add(main_func);

        Asg {
            store,
            global_module,
            main,
        }
    }
}

pub enum SymbolOwner {
    Module(ModuleKey),
    Function(FunctionKey),
}

pub struct SymbolRef {
    // TODO: This needs to be able to point to symbol in local function as well
    pub owner: SymbolOwner,
    pub symbol: SymbolKey,
}

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub parent: Option<ModuleKey>,
    pub symbolscope: SymbolScopeKey,
    pub initalizer: Option<StatementBodyKey>,
}

impl Module {
    pub fn new(store: &mut Store, name: String, parent: Option<ModuleKey>) -> Self {
        Self {
            name: name,
            parent: parent,
            symbolscope: store.symbolscopes.add(SymbolScope::new()),
            initalizer: None,
        }
    }
}

#[derive(Debug)]
pub struct FunctionParameter {
    pub name: String,
    pub typeexpr: ExpressionKey,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub module: ModuleKey,
    pub inparams: Vec<FunctionParameter>,
    pub body: Option<StatementBodyKey>,
}

impl Function {
    pub fn new(name: String, module: ModuleKey) -> Self {
        Self {
            name,
            module,
            inparams: Vec::new(),
            body: None,
        }
    }
}

pub mod statements {
    use super::*;

    #[derive(Debug)]
    pub struct If {
        pub branches: Vec<(ExpressionKey, StatementBodyKey)>,
        pub elsebranch: Option<StatementBodyKey>,
    }

    #[derive(Debug)]
    pub struct Return {
        pub expr: Option<ExpressionKey>,
    }

    #[derive(Debug)]
    pub struct Assign {
        pub lhs: ExpressionKey,
        pub rhs: ExpressionKey,
    }

    #[derive(Debug)]
    pub struct Initialize {
        pub symbol: String,
        pub expr: ExpressionKey,
    }

    #[derive(Debug)]
    pub struct ExpressionWrapper {
        pub expr: ExpressionKey,
    }
}

#[derive(Debug)]
pub enum Statement {
    If(statements::If),
    Return(statements::Return),
    Initialize(statements::Initialize),
    Assign(statements::Assign),
    ExpressionWrapper(statements::ExpressionWrapper),
}

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
        pub symbolref: SymbolReferenceRef,
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
pub enum Expression {
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
pub struct StatementBody {
    pub statements: Vec<Statement>,
}

impl StatementBody {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    }
}
