use crate::asg;
use crate::ir;
use crate::ir::*;

use std::collections::HashMap;

use crate::typesystem::*;
use crate::utils::objectstore::ObjectStore;
use crate::utils::StringKey;

struct CodeGenContext {
    pub function_map: HashMap<asg::FunctionRef, FunctionKey>,
}

impl CodeGenContext {
    pub fn new() -> Self {
        Self {
            function_map: HashMap::new(),
        }
    }
}

fn generate_expression(
    context: &mut CodeGenContext,
    programbuilder: &mut ProgramBuilder,
    functionbuilder: &mut FunctionBuilder,
    current_block: &BasicBlockKey,
    asg: &asg::Asg,
    scoperef: &asg::ScopeRef,
    expressionkey: &asg::ExpressionKey,
) -> Expression {
    let scope = asg.get_scope(scoperef);
    let etype = scope.expressiontypes.get(expressionkey).unwrap();
    match &scope.expressions.get(expressionkey).object {
        asg::ExpressionObject::Literal(n) => match n {
            asg::expressions::Literal::StringLiteral(n) => {
                let data = create_constant_staticstringutf8(&*n.string);

                let constdata = programbuilder.add_constantdata(data);

                // TODO: This is awkward, a Value should be able to hold its
                //  own value.
                // If it did, we would not need to refcount constdata.
                // On the other hand, staticstringutf8 is a reftype, we are not passing the bytes around
                Expression::Constant(Value::Primitive {
                    ptype: PrimitiveType::StaticStringUtf8,
                    data: constdata as u64,
                })
            }
            asg::expressions::Literal::BoolLiteral(_) => todo!(),
            asg::expressions::Literal::IntegerLiteral(n) => {
                let ptype = match etype {
                    TypeId::Primitive(n) => n,
                    _ => panic!("Unsupported integer literal type: {:?}", etype),
                };
                let value = Value::Primitive {
                    ptype: *ptype,
                    data: n.data,
                };
                Expression::Constant(value)
            }
            asg::expressions::Literal::StructLiteral(_) => todo!(),
            asg::expressions::Literal::FunctionLiteral(_) => todo!(),
            asg::expressions::Literal::ModuleLiteral(_) => todo!(),
        },
        asg::ExpressionObject::BuiltInFunction(n) => {
            let value = Value::BuiltInFunction {
                builtin: n.function,
            };
            Expression::Constant(value)
        }
        asg::ExpressionObject::PrimitiveType(_) => todo!(),
        asg::ExpressionObject::SymbolReference(n) => {
            let sref = scope.symboltable.references.get(&n.symbolref);
            match sref {
                asg::SymbolReference::ResolvedReference(n) => {
                    // TODO: Not sure how to deal with difference between closed variables and globals
                    assert!(
                        n.scope == *scoperef,
                        "Only local scope references supported for now!"
                    );

                    // Here, we search through blocks to find last assigned value
                    let variable = functionbuilder
                        .find_last_variable_for_symbol(current_block, &n.symbol)
                        .expect(
                            format!("Cannot find assigned variable for symbol {:?}", n.symbol)
                                .as_str(),
                        );
                    Expression::Variable(variable)
                }
                asg::SymbolReference::UnresolvedReference(n) => {
                    panic!("Unresolved reference! {:?}", n)
                }
            }
        }
        asg::ExpressionObject::If(_) => todo!(),
        asg::ExpressionObject::Call(n) => {
            // Generate callable
            let _callable = generate_expression(
                context,
                programbuilder,
                functionbuilder,
                current_block,
                asg,
                scoperef,
                &n.callable,
            );

            // Generate arguments
            let mut args = Vec::new();
            for arg in &n.args {
                let expr = generate_expression(
                    context,
                    programbuilder,
                    functionbuilder,
                    current_block,
                    asg,
                    scoperef,
                    arg,
                );
                args.push(expr);
            }

            let callabletype = scope.expressiontypes.get(&n.callable).unwrap();
            let returnvar = match callabletype {
                TypeId::BuiltInFunction(n) => {
                    assert!(args.len() >= 1);
                    let dynarglen_const = Expression::Constant(Value::Primitive {
                        ptype: PrimitiveType::U64,
                        data: args.len() as u64 - 1, // First arg is string
                    });

                    // Inject dynamic value count after first arg
                    // TODO: Avoid clones
                    let mut arg_vars = Vec::new();
                    arg_vars.push(args[0].clone());
                    arg_vars.push(dynarglen_const);

                    // Create dynamic value wrappers
                    for arg in &args[1..] {
                        let typeid = arg.get_type(functionbuilder);

                        // Store value, so we can wrap it
                        let variable = functionbuilder.add_unnamed_variable(typeid.clone());
                        functionbuilder
                            .edit_block(current_block)
                            .assign(variable, arg.clone());

                        // Wrap the variable
                        let dynvalexpr =
                            Expression::Constant(Value::TypedValue { typeid, variable });

                        arg_vars.push(dynvalexpr)
                    }

                    let returnvalue = functionbuilder.add_unnamed_variable(TypeId::Null);
                    functionbuilder.edit_block(current_block).call_builtin(
                        returnvalue,
                        *n,
                        arg_vars,
                    );
                    returnvalue
                }
                TypeId::Function(_) => panic!("User callables not yet supported!"),
                _ => panic!("Type {:?} not supported as callable", callabletype),
            };

            Expression::Variable(returnvar)
        }
        asg::ExpressionObject::BinOp(_) => todo!(),
        asg::ExpressionObject::Subscript(_) => todo!(),
    }
}

fn generate_statement_body(
    context: &mut CodeGenContext,
    programbuilder: &mut ProgramBuilder,
    functionbuilder: &mut FunctionBuilder,
    asg: &asg::Asg,
    scoperef: &asg::ScopeRef,
    body: &asg::StatementBody,
) -> BasicBlockKey {
    let blockkey = functionbuilder.create_block();
    for stmnt in &body.statements {
        match stmnt {
            asg::Statement::If(_) => todo!(),
            asg::Statement::Return(_) => todo!(),
            asg::Statement::Initialize(n) => {
                let symbolkey = asg::SymbolKey::from_str(&*n.symbol);
                let scope = asg.get_scope(&scoperef);
                let decltype = scope.declarationtypes.get(&symbolkey).unwrap();

                let sourceexpr = generate_expression(
                    context,
                    programbuilder,
                    functionbuilder,
                    &blockkey,
                    asg,
                    scoperef,
                    &n.expr,
                );

                let assignee = functionbuilder.add_named_variable(symbolkey, decltype.clone());

                let mut block = functionbuilder.edit_block(&blockkey);
                block.assign(assignee, sourceexpr);
            }
            asg::Statement::Assign(n) => todo!(),
            asg::Statement::ExpressionWrapper(n) => {
                generate_expression(
                    context,
                    programbuilder,
                    functionbuilder,
                    &blockkey,
                    asg,
                    scoperef,
                    &n.expr,
                );
            }
        }
    }
    blockkey
}

pub fn generate_program(asg: &asg::Asg) -> ir::Program {
    let mut context = CodeGenContext::new();
    let mut programbuilder = ir::ProgramBuilder::new();

    let init_function = {
        let mut module_inits = Vec::new();

        // Process all modules
        for modulekey in asg.modulekeys() {
            let module = asg.get_module(&modulekey);

            // Module init
            if let Some(body) = &module.body {
                let scoperef = &asg::ScopeRef {
                    module: modulekey,
                    scope: body.scope_nonowned,
                };

                let name = format!("__{}__init", module.name);
                let mut functionbuilder = FunctionBuilder::new(name);

                let entry = generate_statement_body(
                    &mut context,
                    &mut programbuilder,
                    &mut functionbuilder,
                    asg,
                    &scoperef,
                    &body,
                );

                let function = functionbuilder.finish(entry);
                module_inits.push(programbuilder.add_function(function));
            }

            // Generate all functions in module
            for functionkey in module.functionstore.keys() {
                let function = module.functionstore.get(&functionkey);

                if let Some(body) = &function.body {
                    let scoperef = &asg::ScopeRef {
                        module: modulekey,
                        scope: body.scope_nonowned,
                    };

                    let name = format!("{}.{}", module.name, function.name);
                    let mut functionbuilder = FunctionBuilder::new(name);

                    let entry = generate_statement_body(
                        &mut context,
                        &mut programbuilder,
                        &mut functionbuilder,
                        asg,
                        &scoperef,
                        &body,
                    );

                    let function = functionbuilder.finish(entry);

                    // TODO: This function_map relies on functions being generated before they can be referenced.
                    //  Add an indirect function map, or patch up references afterwards.
                    //  Possibly reserve functionkey in programbuilder before adding/editing the function.
                    context.function_map.insert(
                        asg::FunctionRef {
                            module: modulekey,
                            function: functionkey,
                        },
                        programbuilder.add_function(function),
                    );
                }
            }
        }

        // Add program global preamble
        let global_init_function = {
            let mut functionbuilder = ir::FunctionBuilder::new("_global_init".into());

            let entry = functionbuilder.create_block();
            {
                // All init functions return null
                let dummy_var = functionbuilder.add_unnamed_variable(TypeId::Null);

                let mut block = functionbuilder.edit_block(&entry);

                // Call all module inits
                // TODO: Figure out order here
                for function in module_inits {
                    block.call_static(dummy_var, function, Vec::new());
                }

                // Finally call main
                // TODO: Should main be called with cmd args?
                let mainfuncref = asg::FunctionRef {
                    module: asg.global_module,
                    function: asg.main,
                };
                block.call_static(dummy_var, context.function_map[&mainfuncref], Vec::new());
            }
            functionbuilder.finish(entry)
        };

        programbuilder.add_function(global_init_function)
    };

    programbuilder.finish(init_function)
}
