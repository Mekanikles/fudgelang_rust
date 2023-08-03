use super::{objectstore::IndexedObjectStore, *};
use std::collections::HashMap;

pub type ExpressionStore = IndexedObjectStore<Expression>;
pub type ExpressionKey = usize;

#[derive(Debug)]
pub struct Scope {
    pub parent: Option<ScopeRef>,
    pub expressions: ExpressionStore,
    pub symboltable: symboltable::SymbolTable,
    pub declarationtypes: HashMap<SymbolKey, crate::typesystem::TypeId>,
    pub expressiontypes: HashMap<ExpressionKey, crate::typesystem::TypeId>,
}

impl Scope {
    pub fn new(parent: Option<ScopeRef>) -> Self {
        Self {
            parent: parent,
            expressions: ExpressionStore::new(),
            symboltable: symboltable::SymbolTable::new(),
            declarationtypes: HashMap::new(),
            expressiontypes: HashMap::new(),
        }
    }
}
