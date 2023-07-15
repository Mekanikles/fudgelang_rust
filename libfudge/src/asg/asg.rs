use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::typesystem::*;
use crate::utils::objectstore::*;
pub use crate::utils::*;

use super::scope::ExpressionKey;
pub use super::*;

pub type ModuleStore = IndexedObjectStore<Module>;
pub type FunctionStore = IndexedObjectStore<Function>;
pub type ScopeStore = IndexedObjectStore<scope::Scope>;
pub type TypeStore = HashedObjectStore<TypeKey, typesystem::TypeId>;
// Ffs rust, cannot use ModuleStore::Key here...
//  Also, these are weak type aliases
pub type ModuleKey = usize;
pub type FunctionKey = usize;
pub type ScopeKey = usize;
pub type TypeKey = u64;

impl objectstore::HashedStoreKey<typesystem::TypeId> for u64 {
    fn from_obj(object: &typesystem::TypeId) -> Self {
        let mut hasher = DefaultHasher::new();
        object.hash(&mut hasher);
        return hasher.finish();
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct ScopeRef {
    pub module: ModuleKey,
    pub scope: ScopeKey,
}

impl ScopeRef {
    pub fn new(module: ModuleKey, scope: ScopeKey) -> Self {
        Self { module, scope }
    }
}

#[derive(Debug)]
pub struct Asg {
    pub global_module: ModuleKey,
    pub main: FunctionKey,
    pub modulestore: ModuleStore,
}

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub scope: ScopeKey,
    pub typestore: TypeStore,
    pub scopestore: ScopeStore,
    pub functionstore: FunctionStore,
    pub body: Option<StatementBody>,
}

impl Module {
    pub fn new(name: String, parentscope: Option<ScopeRef>) -> Self {
        let mut scopestore = ScopeStore::new();
        let scope = scopestore.add(scope::Scope::new(parentscope));

        Self {
            name: name,
            scope: scope,
            scopestore,
            typestore: TypeStore::new(),
            functionstore: FunctionStore::new(),
            body: None,
        }
    }
}

#[derive(Debug)]
pub struct FunctionParameter {
    // This is a bit weird, but since all symbols are added to the
    //  function's scope for lookup, we just reference it here
    pub symref: symboltable::ResolvedSymbolReference,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub scope: ScopeKey,
    pub inparams: Vec<FunctionParameter>,
    pub body: Option<StatementBody>,
}

impl Function {
    pub fn new(
        name: String,
        scope: ScopeKey,
        inparams: Vec<FunctionParameter>,
        body: Option<StatementBody>,
    ) -> Self {
        Self {
            name,
            scope,
            inparams,
            body,
        }
    }
}

#[derive(Debug)]
pub struct StatementBody {
    pub scope_nonowned: ScopeKey,
    pub statements: Vec<Statement>,
}

impl StatementBody {
    pub fn new(scope: ScopeKey) -> Self {
        Self {
            scope_nonowned: scope,
            statements: Vec::new(),
        }
    }
}
