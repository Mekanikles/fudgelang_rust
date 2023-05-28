use std::collections::HashMap;

use super::{objectstore::*, scope::ExpressionKey, *};

pub type SymbolDeclarationStore = HashedObjectStore<StringKey, SymbolDeclaration>;
pub type SymbolKey = StringKey;

pub type SymbolReferenceStore = IndexedObjectStore<SymbolReference>;
pub type SymbolReferenceKey = usize;

#[derive(Debug)]
pub struct SymbolTable {
    pub declarations: SymbolDeclarationStore,
    pub references: SymbolReferenceStore,
    pub definitions: HashMap<SymbolKey, ExpressionKey>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            declarations: SymbolDeclarationStore::new(),
            references: SymbolReferenceStore::new(),
            definitions: HashMap::new(),
        }
    }
}

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

impl objectstore::HashedStoreKey<SymbolDeclaration> for StringKey {
    fn from_obj(object: &SymbolDeclaration) -> Self {
        StringKey::from_str(object.symbol.as_str())
    }
}

#[derive(Debug, Clone)]
pub enum SymbolReference {
    ResolvedReference(ResolvedSymbolReference),
    UnresolvedReference(UnresolvedSymbolReference),
}
