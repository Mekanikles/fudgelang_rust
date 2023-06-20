use super::{objectstore::IndexedObjectStore, *};

pub type ExpressionStore = IndexedObjectStore<Expression>;
pub type ExpressionKey = usize;

#[derive(Debug)]
pub struct Scope {
    pub parent: Option<ScopeRef>,
    pub expressions: ExpressionStore,
    pub symboltable: symboltable::SymbolTable,
    //pub exprtypemap: HashMap<ExpressionKey, TypeVariable>,
}

impl Scope {
    pub fn new(parent: Option<ScopeRef>) -> Self {
        Self {
            parent: parent,
            expressions: ExpressionStore::new(),
            symboltable: symboltable::SymbolTable::new(),
            //exprtypemap: HashMap::new(),
        }
    }
}
