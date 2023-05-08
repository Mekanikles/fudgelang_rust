use std::collections::HashMap;

use crate::{asg::ResolvedSymbolReference, utils::objectstore::ObjectStore};

use crate::asg::{self, SymbolKey};

fn lookup_symbol(
    asg: &asg::Asg,
    scope: &asg::SymbolScopeKey,
    key: &SymbolKey,
    symbol: &String,
) -> Option<ResolvedSymbolReference> {
    if let Some(decl) = asg
        .store
        .symbolscopes
        .get(&scope)
        .declarations
        .try_get(&key)
    {
        if decl.symbol == *symbol {
            return Some(ResolvedSymbolReference {
                scope: *scope,
                symbol: key.clone(),
            });
        }
    };

    None
}

fn lookup_symbol_recursively(
    asg: &asg::Asg,
    from_scope: &asg::SymbolScopeKey,
    symbol: &String,
) -> Option<ResolvedSymbolReference> {
    let key = SymbolKey::from_str(symbol);

    let mut iter = Some(*from_scope);
    while let Some(scope) = iter {
        if let Some(resref) = lookup_symbol(&asg, &scope, &key, &symbol) {
            return Some(resref);
        }

        iter = asg.store.symbolscopes.get(&scope).parent;
    }

    return None;
}

pub fn resolve_symbols(mut asg: asg::Asg) -> asg::Asg {
    for symbolscope in asg.store.symbolscopes.keys() {
        // This is to avoid a mut ref to references, while needing a ref to asg for lookups
        // TODO: This clones a bunch of strings :(
        let mut lookup_map = HashMap::new();

        // Do lookups
        for reference in asg.store.symbolscopes.get(&symbolscope).references.values() {
            match reference {
                asg::SymbolReference::UnresolvedReference(n) => {
                    // TODO: This lookup is duplicated for repeated references
                    if let Some(resref) = lookup_symbol_recursively(&asg, &symbolscope, &n.symbol) {
                        lookup_map.insert(
                            n.symbol.clone(),
                            asg::SymbolReference::ResolvedReference(resref),
                        );
                    }
                }
                _ => {}
            }
        }

        // Update references
        for reference in asg
            .store
            .symbolscopes
            .get_mut(&symbolscope)
            .references
            .values_mut()
        {
            match reference {
                asg::SymbolReference::UnresolvedReference(n) => {
                    if let Some(resref) = lookup_map.get(&n.symbol) {
                        *reference = resref.clone();
                    }
                }
                _ => {}
            }
        }
    }

    asg
}
