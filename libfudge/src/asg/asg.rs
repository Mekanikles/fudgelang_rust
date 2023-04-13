use std::collections::HashMap;

use crate::typesystem::*;
use crate::utils::objectstore::*;
pub use crate::utils::*;

use crate::shared::BinaryOperationType;

pub type ModuleStore = IndexedObjectStore<Module>;
pub type FunctionStore = IndexedObjectStore<Function>;
pub type ExpressionStore = IndexedObjectStore<Expression>;
pub type SymbolScopeStore = IndexedObjectStore<SymbolScope>;
// Ffs rust, cannot use ModuleStore::Key here...
pub type ModuleKey = usize;
pub type FunctionKey = usize;
pub type ExpressionKey = usize;
pub type SymbolScopeKey = usize;

#[derive(PartialEq, Eq)]
pub struct SymbolDeclaration {
    symbol: String,
    typeexpr: Option<ExpressionKey>,
    initexpr: Option<ExpressionKey>,
}

impl SymbolDeclaration {
    pub fn new(
        symbol: String,
        typeexpr: Option<ExpressionKey>,
        initexpr: Option<ExpressionKey>,
    ) -> Self {
        Self {
            symbol,
            typeexpr,
            initexpr,
        }
    }
}

pub type SymbolDeclarationStore = HashedObjectStore<StringKey, SymbolDeclaration>;
pub type SymbolKey = StringKey;

impl objectstore::HashedStoreKey<SymbolDeclaration> for StringKey {
    fn from_obj(object: &SymbolDeclaration) -> Self {
        StringKey::from_str(object.symbol.as_str())
    }
}

pub struct UnresolvedSymbolReference {
    pub symbol: String,
}

pub struct ResolvedSymbolReference {
    pub scope: SymbolScopeKey,
    pub symbol: SymbolKey,
}

pub enum SymbolReference {
    ResolvedReference(ResolvedSymbolReference),
    UnresolvedReference(UnresolvedSymbolReference),
}

pub type SymbolReferenceStore = IndexedObjectStore<SymbolReference>;
pub type SymbolReferenceKey = usize;

pub struct SymbolScope {
    pub declarations: SymbolDeclarationStore,
    pub references: SymbolReferenceStore,
}

impl SymbolScope {
    pub fn new() -> Self {
        Self {
            declarations: SymbolDeclarationStore::new(),
            references: SymbolReferenceStore::new(),
        }
    }
}

pub struct Store {
    pub modules: ModuleStore,
    pub functions: FunctionStore,
    pub expressions: ExpressionStore,
    pub symbolscopes: SymbolScopeStore,
}

impl Store {
    pub fn new() -> Self {
        Self {
            modules: ModuleStore::new(),
            functions: FunctionStore::new(),
            expressions: ExpressionStore::new(),
            symbolscopes: SymbolScopeStore::new(),
        }
    }
}

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
        let main = store.functions.add(Function::new("main".into()));

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

pub struct Module {
    pub name: String,
    pub parent: Option<ModuleKey>,
    pub symbolscope: SymbolScopeKey,
    pub initalizer: StatementBody,
}

impl Module {
    pub fn new(store: &mut Store, name: String, parent: Option<ModuleKey>) -> Self {
        Self {
            name: name,
            parent: parent,
            symbolscope: store.symbolscopes.add(SymbolScope::new()),
            initalizer: StatementBody::new(),
        }
    }
}

pub struct FunctionParameter {
    pub name: String,
    pub typeexpr: ExpressionKey,
}

pub struct Function {
    pub name: String,
    pub inparams: Vec<FunctionParameter>,
    pub body: StatementBody,
}

impl Function {
    pub fn new(name: String) -> Self {
        Self {
            name: name,
            inparams: Vec::new(),
            body: StatementBody::new(),
        }
    }
}

pub mod statements {
    use super::*;

    pub struct Return {
        pub expr: ExpressionKey,
    }

    pub struct Assign {
        pub symbol: SymbolKey,
        pub expr: ExpressionKey,
    }
}

pub enum Statement {
    Return(statements::Return),
    Assign(statements::Assign),
}

pub mod misc {
    use super::*;

    pub struct StructField {
        pub name: String,
        pub typeexpr: ExpressionKey,
    }
}

pub mod expressions {
    use super::*;

    pub mod literals {
        use super::*;

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
        pub struct StructLiteral {
            pub fields: Vec<misc::StructField>,
        }
        pub struct FunctionLiteral {
            pub functionkey: FunctionKey,
        }
        pub struct ModuleLiteral {
            pub modulekey: ModuleKey,
        }
    }

    pub enum Literal {
        StringLiteral(literals::StringLiteral),
        BoolLiteral(literals::BoolLiteral),
        IntegerLiteral(literals::IntegerLiteral),
        StructLiteral(literals::StructLiteral),
        FunctionLiteral(literals::FunctionLiteral),
        ModuleLiteral(literals::ModuleLiteral),
    }

    pub struct BuiltInFunction {
        pub function: typesystem::BuiltInFunction,
    }

    pub struct PrimitiveType {
        pub ptype: typesystem::PrimitiveType,
    }

    pub struct SymbolReference {
        pub symbolref: SymbolReferenceKey,
    }

    pub struct If {
        pub branches: Vec<(ExpressionKey, ExpressionKey)>,
        pub elsebranch: Option<ExpressionKey>,
    }

    pub struct Call {
        pub callable: ExpressionKey,
        pub args: Vec<ExpressionKey>,
    }

    pub struct BinOp {
        pub op: BinaryOperationType,
        pub lhs: ExpressionKey,
        pub rhs: ExpressionKey,
    }

    pub struct Subscript {
        pub expr: ExpressionKey,
        pub symbol: String,
    }
}

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
