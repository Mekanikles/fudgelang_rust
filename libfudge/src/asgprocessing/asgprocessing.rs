use std::any::type_name;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::asg::symboltable::SymbolKey;
use crate::asg::*;
use crate::utils::objectstore::*;

use crate::typesystem::*;

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
    scope: &asg::ScopeRef,
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
                        scope: *scoperef,
                        symbol: key.clone(),
                    },
                );
            }
        }

        iter = scope.parent.as_ref();
    }

    return asg::symboltable::SymbolReference::UnresolvedReference(reference.clone());
}

#[derive(Debug)]
enum TypeVariable {
    Free,
    TypeSet(HashSet<TypeId>),
}

impl TypeVariable {
    pub fn new_typeset(types: HashSet<TypeId>) -> Self {
        Self::TypeSet(types)
    }
}

#[derive(Debug)]
enum TypeEntry {
    Id(TypeId),
    Variable(TypeVariable),
    Substituted(TypeEntryKey),
}

#[derive(Debug)]
enum TypeConstraint {
    EqualsEntry {
        lhs: TypeEntryKey,
        rhs: TypeEntryKey,
    },
    EqualsTypeId {
        entry: TypeEntryKey,
        id: TypeId,
    },
    EqualsCallParam {
        call: TypeEntryKey,
        param: usize,
        arg: TypeEntryKey,
    },
    ValueOfExpr {
        entry: TypeEntryKey,
        expr: ExpressionKey,
    },
    TypeOfSymbol {
        entry: TypeEntryKey,
        symref: SymbolReferenceKey,
    },
}

type TypeEntryStore = IndexedObjectStore<TypeEntry>;
pub type TypeEntryKey = usize;

// TODO: Make store items removable, use option for now
type TypeConstraintStore = IndexedObjectStore<Option<TypeConstraint>>;
pub type TypeConstraintKey = usize;

#[derive(Debug)]
struct TypeEnvironment {
    types: TypeEntryStore,
    constraints: TypeConstraintStore,
    symbolmap: HashMap<SymbolKey, TypeEntryKey>,
    exprmap: HashMap<ExpressionKey, TypeEntryKey>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            types: TypeEntryStore::new(),
            constraints: TypeConstraintStore::new(),
            symbolmap: HashMap::new(),
            exprmap: HashMap::new(),
        }
    }

    pub fn get_entry(&self, key: &TypeEntryKey) -> &TypeEntry {
        self.types.get(key)
    }

    pub fn get_entry_mut(&mut self, key: &TypeEntryKey) -> &mut TypeEntry {
        self.types.get_mut(key)
    }

    pub fn add_for_symbol(&mut self, symbolkey: SymbolKey, typeentry: TypeEntry) -> TypeEntryKey {
        let key = self.types.add(typeentry);
        self.symbolmap.insert(symbolkey, key.clone());
        key
    }

    pub fn get_for_symbol(&self, symbolkey: &SymbolKey) -> TypeEntryKey {
        self.symbolmap.get(symbolkey).unwrap().clone()
    }

    pub fn get_for_expression(&self, expressionkey: &ExpressionKey) -> TypeEntryKey {
        self.exprmap.get(expressionkey).unwrap().clone()
    }

    pub fn add_for_expression(
        &mut self,
        exprkey: ExpressionKey,
        typeentry: TypeEntry,
    ) -> TypeEntryKey {
        let key = self.types.add(typeentry);
        self.exprmap.insert(exprkey, key.clone());
        key
    }

    pub fn add_constraint(&mut self, constraint: TypeConstraint) -> TypeConstraintKey {
        self.constraints.add(Some(constraint))
    }

    pub fn remove_constraint(&mut self, key: &TypeConstraintKey) -> Option<TypeConstraint> {
        std::mem::replace(self.constraints.get_mut(key), None)
    }
}

fn process_expression_type(
    asg: &asg::Asg,
    scope: &asg::scope::Scope,
    exprkey: &ExpressionKey,
    typeenv: &mut TypeEnvironment,
) -> TypeEntryKey {
    let expression = scope.expressions.get(exprkey);

    match &expression.object {
        expression::ExpressionObject::Literal(n) => {
            use expression::expressions::Literal::*;
            match n {
                IntegerLiteral(_) => {
                    let mut types = HashSet::new();
                    // TODO: Limit range based on literal and support signed integer types and defaults
                    types.insert(TypeId::Primitive(PrimitiveType::U64));
                    types.insert(TypeId::Primitive(PrimitiveType::U32));
                    types.insert(TypeId::Primitive(PrimitiveType::U16));
                    types.insert(TypeId::Primitive(PrimitiveType::U8));
                    typeenv.add_for_expression(
                        exprkey.clone(),
                        TypeEntry::Variable(TypeVariable::new_typeset(types)),
                    )
                }
                StringLiteral(_) => typeenv.add_for_expression(
                    exprkey.clone(),
                    TypeEntry::Id(TypeId::Primitive(PrimitiveType::StaticStringUtf8)),
                ),
                _ => todo!(),
                /*BoolLiteral(_) => TypeVariable::new_primitive(PrimitiveType::Bool),
                IntegerLiteral(_) => {
                    // TODO: All integer literals are u64 for now
                    TypeVariable::new_primitive(PrimitiveType::U64)
                }
                StructLiteral(_) => TypeVariable::new_type(),
                FunctionLiteral(n) => {
                    let mut inputparams = Vec::new();

                    let module = asg.modulestore.get(&scoperef.module);
                    let function = module.functionstore.get(&n.functionkey);
                    for inputparam in &function.inparams {
                        let scopecache = typecache.get_mut(scoperef).unwrap();
                        let typevar = TypeVariable::new_symboltype(
                            asg::symboltable::SymbolReference::ResolvedReference(
                                inputparam.symref.clone(),
                            ),
                        );
                        let key = scopecache.insert(typevar);
                        inputparams.push((inputparam.symref.symbol.clone(), key));
                    }

                    TypeVariable::new_function(inputparams)
                }
                ModuleLiteral(_) => TypeVariable::new_module(),*/
            }
        }
        expression::ExpressionObject::PrimitiveType(n) => {
            // All primitive types are of type "Type"
            typeenv.add_for_expression(exprkey.clone(), TypeEntry::Id(TypeId::Type))
        }
        expression::ExpressionObject::BuiltInFunction(n) => {
            // TODO: Look up function types of built-ins properly
            typeenv.add_for_expression(
                exprkey.clone(),
                TypeEntry::Id(TypeId::BuiltInFunction(n.function)),
            )
        }
        expression::ExpressionObject::Call(n) => {
            // Process callable
            let callabletype = process_expression_type(asg, scope, &n.callable, typeenv);

            // Process and constraint args
            for (i, argexpr) in n.args.iter().enumerate() {
                let argtype = process_expression_type(asg, scope, argexpr, typeenv);
                let constraint = TypeConstraint::EqualsCallParam {
                    call: callabletype,
                    param: i,
                    arg: argtype,
                };
                typeenv.add_constraint(constraint);
            }

            // TODO: All calls return null for now
            typeenv.add_for_expression(exprkey.clone(), TypeEntry::Id(TypeId::Null))
        }
        expression::ExpressionObject::SymbolReference(n) => {
            // We don't know yet the type of sym refs, we do lookup later
            let tv = typeenv
                .add_for_expression(exprkey.clone(), TypeEntry::Variable(TypeVariable::Free));
            typeenv.add_constraint(TypeConstraint::TypeOfSymbol {
                entry: tv,
                symref: n.symbolref,
            });
            tv
        }
        _ => todo!(),
        /*
        expression::ExpressionObject::SymbolReference(n) => {
            let symref = scope.symboltable.references.get(&n.symbolref);
            TypeVariable::new_symboltype(symref.clone())
        }
        expression::ExpressionObject::If(n) => {
            let mut branch_expressions = Vec::new();

            for b in &n.branches {
                branch_expressions.push(b.1);
            }

            if let Some(e) = n.elsebranch {
                branch_expressions.push(e);
            }

            TypeVariable::new_expression_types(branch_expressions)
        }
        expression::ExpressionObject::BinOp(n) => {
            // TODO: For now, assume all operators have to be same type
            let mut ops = Vec::new();
            ops.push(n.lhs);
            ops.push(n.rhs);

            TypeVariable::new_expression_types(ops)
        }
        expression::ExpressionObject::Subscript(_) => todo!(),*/
    }
}

fn eval_expression_as_type(
    asg: &asg::Asg,
    scope: &asg::scope::Scope,
    exprkey: &ExpressionKey,
) -> TypeId {
    let expression = scope.expressions.get(exprkey);

    match &expression.object {
        expression::ExpressionObject::PrimitiveType(n) => TypeId::Primitive(n.ptype),
        _ => panic!("Cannot currently evaulate non-built in type literal expressions"),
    }
}

fn process_function(
    asg: &mut asg::Asg,
    modulekey: &asg::ModuleKey,
    functionkey: &asg::FunctionKey,
) {
    fn can_unify_var_id(a: &TypeVariable, b: &TypeId) -> bool {
        match a {
            TypeVariable::Free => true,
            TypeVariable::TypeSet(n) => {
                assert!(
                    n.contains(&b),
                    "Failed unify, typeSet did not contain type: {:?}, {:?}",
                    n,
                    a
                );
                true
            }
        }
    }

    fn resolve_substitutions(key: &TypeEntryKey, typeenv: &TypeEnvironment) -> TypeEntryKey {
        let mut key = key;
        loop {
            let entry = typeenv.get_entry(key);
            match entry {
                TypeEntry::Substituted(n) => key = n,
                _ => return *key,
            }
            return *key;
        }
    }

    let mut typeenv = TypeEnvironment::new();
    let typeenv = &mut typeenv;

    // Decls
    {
        let module = asg.modulestore.get(&modulekey);
        let function = module.functionstore.get(&functionkey);

        let scope = module.scopestore.get(&function.scope);
        let decls = &scope.symboltable.declarations;
        for symkey in decls.keys() {
            let decl = decls.get(symkey);
            if let Some(typeexpr) = &decl.typeexpr {
                // Process type for type expression
                let exprtype = process_expression_type(asg, &scope, &typeexpr, typeenv);

                // Constrain it to "Type"
                typeenv.add_constraint(TypeConstraint::EqualsTypeId {
                    entry: exprtype,
                    id: TypeId::Type,
                });

                // Add entry for symbol
                let entry =
                    typeenv.add_for_symbol(symkey.clone(), TypeEntry::Variable(TypeVariable::Free));

                // Constrain the type of the symbol to the _value_ of the expression
                typeenv.add_constraint(TypeConstraint::ValueOfExpr {
                    entry,
                    expr: typeexpr.clone(),
                });
            } else {
                todo!();
            }
        }
    }

    // Statements
    {
        let module = asg.modulestore.get(&modulekey);
        let function = module.functionstore.get(&functionkey);

        if let Some(body) = &function.body {
            let scope = module.scopestore.get(&body.scope_nonowned);
            for stmnt in &body.statements {
                match stmnt {
                    Statement::If(_) => todo!(),
                    Statement::Return(_) => todo!(),
                    Statement::Initialize(n) => {
                        let symkey = SymbolKey::from_str(n.symbol.as_str()); // TODO: Don't need complete symbol in n

                        // Add type for initialization expression
                        let rhs = process_expression_type(asg, &scope, &n.expr, typeenv);

                        // Symbol declaration should have been added earlier
                        let lhs = typeenv.get_for_symbol(&symkey);

                        // Add constraint for symboltype and expression
                        typeenv.add_constraint(TypeConstraint::EqualsEntry { lhs, rhs });
                    }
                    Statement::Assign(_) => todo!(),
                    Statement::ExpressionWrapper(n) => {
                        process_expression_type(asg, &scope, &n.expr, typeenv);
                    }
                }
            }
        }
    }

    // At this point we should be ready to start processing type constraints
    let mut iteration = 0;
    loop {
        let last_constraints = typeenv.constraints.keys();

        println!("Iteration {}: {:?}", iteration, typeenv);

        for constraintkey in typeenv.constraints.keys() {
            fn resolve_type<'a>(key: &TypeEntryKey, typeenv: &'a TypeEnvironment) -> &'a TypeId {
                let subst = typeenv.get_entry(&resolve_substitutions(key, typeenv));
                match subst {
                    TypeEntry::Id(n) => &n,
                    TypeEntry::Variable(n) => {
                        panic!("Cannot resolve type {:?}", n);
                    }
                    TypeEntry::Substituted(_) => panic!("Substitutions not allowed!"),
                }
            }

            // TODO: This is the worst thing ever, use a queue instead
            let c = typeenv.remove_constraint(&constraintkey);

            // Note: Make sure to resolve substitutions for keys, to avoid unifying stale variables
            match c {
                Some(n) => match n {
                    TypeConstraint::EqualsEntry {
                        lhs: lhskey,
                        rhs: rhskey,
                    } => {
                        let lhs = typeenv.get_entry(&resolve_substitutions(&lhskey, &typeenv));
                        let rhs = typeenv.get_entry(&resolve_substitutions(&rhskey, &typeenv));

                        match (lhs, rhs) {
                            (TypeEntry::Id(lhs), TypeEntry::Id(rhs)) => {
                                assert!(lhs == rhs, "Type mismatch: {:?}, {:?}", lhs, rhs);
                                // No need to substitute, only check
                            }
                            (TypeEntry::Variable(lhs), TypeEntry::Id(rhs)) => {
                                if can_unify_var_id(&lhs, &rhs) {
                                    // Typecheck passed, assign the variable to typeid
                                    // TODO: Typeid cloned here :(
                                    *typeenv.get_entry_mut(&lhskey) =
                                        TypeEntry::Substituted(rhskey);
                                }
                            }
                            (TypeEntry::Id(lhs), TypeEntry::Variable(rhs)) => {
                                if can_unify_var_id(&rhs, &lhs) {
                                    // Typecheck passed, assign the variable to typeid
                                    // TODO: Typeid cloned here :(
                                    *typeenv.get_entry_mut(&rhskey) =
                                        TypeEntry::Substituted(lhskey);
                                }
                            }
                            (TypeEntry::Variable(rhs), TypeEntry::Variable(lhs)) => {
                                match (rhs, lhs) {
                                    (TypeVariable::Free, TypeVariable::Free) => {
                                        // Can always unify free types
                                        *typeenv.get_entry_mut(&lhskey) =
                                            TypeEntry::Substituted(rhskey);
                                    }
                                    (TypeVariable::TypeSet(_), TypeVariable::Free) => {
                                        // Can always unify free types
                                        *typeenv.get_entry_mut(&rhskey) =
                                            TypeEntry::Substituted(lhskey);
                                    }
                                    (TypeVariable::Free, TypeVariable::TypeSet(_)) => {
                                        // Can always unify free types
                                        *typeenv.get_entry_mut(&lhskey) =
                                            TypeEntry::Substituted(rhskey);
                                    }
                                    (TypeVariable::TypeSet(lhs), TypeVariable::TypeSet(rhs)) => {
                                        let intersection: HashSet<_> =
                                            lhs.intersection(rhs).cloned().collect();
                                        if !intersection.is_empty() {
                                            // Set lhs to rhs
                                            *typeenv.get_entry_mut(&lhskey) =
                                                TypeEntry::Substituted(rhskey);

                                            // Reduce rhs set to intersection
                                            *typeenv.get_entry_mut(&rhskey) = TypeEntry::Variable(
                                                TypeVariable::TypeSet(intersection),
                                            );
                                        } else {
                                            panic!("No intersection between: {:?}, {:?}", lhs, rhs);
                                        }
                                    }
                                }
                            }
                            (_, TypeEntry::Substituted(_)) | (TypeEntry::Substituted(_), _) => {
                                panic!("Substitutions not allowed!")
                            }
                        }
                    }
                    TypeConstraint::EqualsTypeId { entry, id } => {
                        let lhs = typeenv.get_entry(&resolve_substitutions(&entry, &typeenv));
                        match lhs {
                            TypeEntry::Id(n) => {
                                assert!(*n == id, "Type mismatch: {:?}, {:?}", n, id);
                            }
                            TypeEntry::Variable(n) => {
                                if can_unify_var_id(&n, &id) {
                                    // Typecheck passed, assign the variable to typeid
                                    // TODO: Typeid cloned here :(
                                    *typeenv.get_entry_mut(&entry) = TypeEntry::Id(id.clone());
                                } else {
                                    panic!("Cannot unify {:?} and {:?}", n, id)
                                }
                            }
                            TypeEntry::Substituted(_) => panic!("Substitutions not allowed!"),
                        };
                    }
                    TypeConstraint::EqualsCallParam {
                        call: callentrykey,
                        param: paramindex,
                        arg: argentrykey,
                    } => {
                        let callabletype =
                            resolve_type(&resolve_substitutions(&callentrykey, &typeenv), &typeenv);

                        match callabletype {
                            TypeId::BuiltInFunction(n) => match n {
                                BuiltInFunction::PrintFormat => {
                                    // Generate new constraints for each argument
                                    // Print format first arg is string, rest "Any"
                                    let paramentry = if paramindex == 0 {
                                        Some(TypeId::Primitive(PrimitiveType::StaticStringUtf8))
                                    } else {
                                        None
                                    };
                                    match paramentry {
                                        Some(n) => {
                                            // Type known, add concrete constraint
                                            typeenv.add_constraint(TypeConstraint::EqualsTypeId {
                                                entry: argentrykey,
                                                id: n,
                                            });
                                        }
                                        None => {
                                            // Type any, no new contraints necessary
                                        }
                                    }
                                }
                            },
                            _ => panic!("Only built-in function calls supported!"),
                        }
                    }
                    TypeConstraint::ValueOfExpr { entry, expr } => {
                        let module = asg.modulestore.get(&modulekey);
                        let function = module.functionstore.get(&functionkey);
                        let scope = module.scopestore.get(&function.scope);
                        let typeid = eval_expression_as_type(asg, scope, &expr);

                        let lhs = typeenv.get_entry(&resolve_substitutions(&entry, &typeenv));

                        match lhs {
                            TypeEntry::Id(n) => {
                                assert!(*n == typeid, "Type mismatch: {:?}, {:?}", n, typeid);
                            }
                            TypeEntry::Variable(n) => {
                                if can_unify_var_id(&n, &typeid) {
                                    // Typecheck passed, assign the variable to typeid
                                    *typeenv.get_entry_mut(&entry) = TypeEntry::Id(typeid);
                                } else {
                                    panic!("Cannot unify {:?} and {:?}", n, typeid)
                                }
                            }
                            TypeEntry::Substituted(_) => panic!("Substitutions not allowed!"),
                        };
                    }
                    TypeConstraint::TypeOfSymbol {
                        entry,
                        symref: symrefkey,
                    } => {
                        // TODO: This wont work with multiple scope in a function
                        let module = asg.modulestore.get(&modulekey);
                        let function = module.functionstore.get(&functionkey);
                        let scope = module.scopestore.get(&function.scope);
                        let symref = scope.symboltable.references.get(&symrefkey);

                        match symref {
                            SymbolReference::ResolvedReference(_) => {
                                panic!("Symbol was already resolved!")
                            }
                            SymbolReference::UnresolvedReference(n) => {
                                // Lookup symbol
                                let scoperef = ScopeRef::new(*modulekey, function.scope);
                                let resolved_ref = lookup_symbol(asg, n, &scoperef);

                                match &resolved_ref {
                                    SymbolReference::ResolvedReference(n) => {
                                        // Queue a new constraint with the resolved symbol entry
                                        let rhs = typeenv.get_for_symbol(&n.symbol);
                                        typeenv.add_constraint(TypeConstraint::EqualsEntry {
                                            lhs: entry,
                                            rhs: rhs,
                                        });
                                    }
                                    SymbolReference::UnresolvedReference(_) => {
                                        panic!("Symbol could not be resolved!")
                                    }
                                }

                                // Update reference in asg
                                let modmut = asg.modulestore.get_mut(&modulekey);
                                let scopemut = modmut.scopestore.get_mut(&scoperef.scope);
                                let symrefmod = scopemut.symboltable.references.get_mut(&symrefkey);
                                *symrefmod = resolved_ref;
                            }
                        }
                    }
                },
                None => {}
            };
        }

        // TODO: Bleh, using a queue would make this check cleaner
        let constraints = typeenv.constraints.keys();
        if constraints == last_constraints {
            assert!(
                typeenv.constraints.values().iter().all(|x| x.is_none()),
                "Unresolved constraints left: {:?}",
                typeenv.constraints.values()
            );
            break;
        }

        iteration += 1;
    }

    let scopekey = {
        let module = asg.modulestore.get(&modulekey);
        let function = module.functionstore.get(&functionkey);
        function.scope
    };

    println!("Type processing done!");
    println!("  Declaration types:");
    {
        let module = asg.modulestore.get(&modulekey);
        let scope = module.scopestore.get(&scopekey);
        let decls = &scope.symboltable.declarations;
        for d in decls.keys() {
            println!(
                "    {}: {:?}",
                decls.get(d).symbol,
                typeenv.get_entry(&resolve_substitutions(&typeenv.get_for_symbol(d), typeenv))
            );
        }
    }
    println!("  Expression types:");
    {
        let module = asg.modulestore.get(&modulekey);
        let scope = module.scopestore.get(&scopekey);
        for e in scope.expressions.keys() {
            println!(
                "    e{}: {:?}",
                e,
                typeenv.get_entry(&resolve_substitutions(
                    &typeenv.get_for_expression(&e),
                    typeenv
                ))
            );
        }
    }

    // Write back result to asg
    {
        let mut decltypes = HashMap::new();
        {
            let module = asg.modulestore.get(&modulekey);
            let scope = module.scopestore.get(&scopekey);

            let decls = &scope.symboltable.declarations;
            for d in decls.keys() {
                let e =
                    typeenv.get_entry(&resolve_substitutions(&typeenv.get_for_symbol(d), typeenv));
                match e {
                    TypeEntry::Id(n) => decltypes.insert(d.clone(), n.clone()),
                    _ => panic!(
                        "Unresolved type for symbol {:?}",
                        scope.symboltable.declarations.get(&d)
                    ),
                };
            }
        }
        asg.modulestore
            .get_mut(&modulekey)
            .scopestore
            .get_mut(&scopekey)
            .declarationtypes = decltypes;

        let mut exprtypes = HashMap::new();
        {
            let module = asg.modulestore.get(&modulekey);
            let scope = module.scopestore.get(&scopekey);

            for expkey in scope.expressions.keys() {
                let e = typeenv.get_entry(&resolve_substitutions(
                    &typeenv.get_for_expression(&expkey),
                    typeenv,
                ));
                match e {
                    TypeEntry::Id(n) => exprtypes.insert(expkey, n.clone()),
                    _ => panic!(
                        "Unresolved type for expression {:?}",
                        scope.expressions.get(&expkey)
                    ),
                };
            }
        }

        asg.modulestore
            .get_mut(&modulekey)
            .scopestore
            .get_mut(&scopekey)
            .expressiontypes = exprtypes;
    }
}

pub fn process_asg(mut asg: asg::Asg) -> asg::Asg {
    let module = asg.global_module.clone();
    let function = asg.main.clone();
    process_function(&mut asg, &module, &function);

    asg
}
