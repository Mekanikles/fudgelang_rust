use std::collections::HashMap;

use std::mem;

use asg::objectstore::ObjectStore;

use crate::asg;

fn get_scope<'a>(asg: &'a asg::Asg, scope: &asg::ScopeRef) -> &'a asg::scope::Scope {
    asg.modulestore
        .get(&scope.module)
        .scopestore
        .get(&scope.scope)
}

fn get_scope_mut<'a>(asg: &'a mut asg::Asg, scope: &asg::ScopeRef) -> &'a mut asg::scope::Scope {
    asg.modulestore
        .get_mut(&scope.module)
        .scopestore
        .get_mut(&scope.scope)
}

fn lookup_symbol(
    asg: &asg::Asg,
    reference: &asg::symboltable::UnresolvedSymbolReference,
    scope: asg::ScopeRef,
) -> asg::symboltable::SymbolReference {
    let key = asg::symboltable::SymbolKey::from_str(reference.symbol.as_str());

    let mut iter = Some(scope);
    while let Some(scoperef) = iter {
        let scope = get_scope(&asg, &scoperef);

        if let Some(decl) = scope.symboltable.declarations.try_get(&key) {
            // TODO: Handle hash collisions
            assert!(decl.symbol == *reference.symbol);
            if decl.symbol == *reference.symbol {
                return asg::symboltable::SymbolReference::ResolvedReference(
                    asg::symboltable::ResolvedSymbolReference {
                        scope: scoperef,
                        symbol: key.clone(),
                    },
                );
            }
        }

        iter = scope.parent.clone();
    }

    return asg::symboltable::SymbolReference::UnresolvedReference(reference.clone());
}

pub fn process_asg(mut asg: asg::Asg) -> asg::Asg {
    let mut collected_scoperefs = Vec::new();

    // Collect all scopes
    for modulekey in asg.modulestore.keys() {
        let module = asg.modulestore.get_mut(&modulekey);
        for scopekey in module.scopestore.keys() {
            collected_scoperefs.push(asg::ScopeRef::new(modulekey, scopekey));
        }
    }

    // Lookup symbols
    for scoperef in collected_scoperefs {
        // Really finagling to not iterate over references while trying to modify them
        let mut references = mem::replace(
            &mut get_scope_mut(&mut asg, &scoperef).symboltable.references,
            asg::symboltable::SymbolReferenceStore::new(),
        );

        for reference in references.values_mut() {
            match reference {
                asg::symboltable::SymbolReference::ResolvedReference(_) => {}
                asg::symboltable::SymbolReference::UnresolvedReference(n) => {
                    *reference = lookup_symbol(&asg, &n, scoperef.clone())
                }
            }
        }

        get_scope_mut(&mut asg, &scoperef).symboltable.references = references;
    }

    asg
}
