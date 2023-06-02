use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::typesystem::*;
use crate::utils::objectstore::*;
pub use crate::utils::*;

use super::*;

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

#[derive(Debug, Clone)]
pub struct ScopeRef {
    pub module: ModuleKey,
    pub scope: ScopeKey,
}

impl ScopeRef {
    pub fn new(module: ModuleKey, scope: ScopeKey) -> Self {
        Self { module, scope }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum TypeVariable {}

#[derive(Debug)]
pub struct Asg {
    pub global_module: ModuleKey,
    pub main: FunctionKey,
    pub modulestore: ModuleStore,
}

impl Asg {
    pub fn new() -> Self {
        let mut global_module = Module::new("global".into(), None);
        let scope = global_module.scope;

        // Note: main should not be available for symbol lookup, so don't add it to any scope
        let main_func = Function::new("main".into(), &mut global_module, ScopeRef::new(0, scope));
        let main = global_module.functionstore.add(main_func);

        let mut modulestore = ModuleStore::new();
        let global_module = modulestore.add(global_module);

        Asg {
            global_module,
            main,
            modulestore,
        }
    }
}

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub scope: ScopeKey,
    pub typestore: TypeStore,
    pub scopestore: ScopeStore,
    pub functionstore: FunctionStore,
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
        }
    }
}

#[derive(Debug)]
pub struct FunctionParameter {
    // This is a bit weird, but since all symbols are added to the
    //  functinon's scope for lookup, we just reference it here
    pub symref: symboltable::ResolvedSymbolReference,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub inparams: Vec<FunctionParameter>,
    pub scope: ScopeKey,
}

impl Function {
    pub fn new(name: String, module: &mut Module, parentscope: ScopeRef) -> Self {
        Self {
            name,
            inparams: Vec::new(),
            scope: module.scopestore.add(scope::Scope::new(Some(parentscope))),
        }
    }
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
