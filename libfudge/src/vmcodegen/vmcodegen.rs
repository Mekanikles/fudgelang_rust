use crate::asg;
use crate::asg::expressions::BuiltInFunction;
use crate::vm;

use std::collections::HashMap;

use crate::utils::objectstore::ObjectStore;

fn get_scope<'a>(asg: &'a asg::Asg, scoperef: &asg::ScopeRef) -> &'a asg::Scope {
    &asg.modulestore
        .get(&scoperef.module)
        .scopestore
        .get(&scoperef.scope)
}

fn create_constdata_utf8_static_string(
    builder: &mut vm::ProgramBuilder,
    str: &str,
) -> vm::ConstDataHandle {
    let block = builder.alloc_constdata(str.len() + 8);
    let data = builder.edit_constdata(&block);

    let str = str.as_bytes();

    // String is [bytelen: u64, data: &[u8]]
    // TODO: We should not need to store the length alongside the data
    //  the string array length should be known, or packed in the string-ref type
    data[0..8].copy_from_slice(&str.len().to_be_bytes());
    data[8..].copy_from_slice(str);

    block
}

struct RegisterAllocator {
    pub registers_used: [bool; 256],
}

impl RegisterAllocator {
    fn new() -> Self {
        Self {
            registers_used: [false; 256],
        }
    }

    fn acquire_param(&mut self, index : usize) -> vm::Register {
        // Params are passed in registers 0+
        assert!(!self.registers_used[index], "Register {} already in use!", index);
        self.registers_used[index] = true;
        index as u8
    }

    pub fn acquire(&mut self) -> vm::Register {
        // Allocate temp registers from the bottom to avoid
        //  collisions with call arguments/return values
        // TODO: Bleh, but I don't know how to deal with reverse iterators
        let index = vm::RETURN_REGISTER as usize - self
            .registers_used[0..vm::RETURN_REGISTER as usize]
            .iter().rev()
            .position(|&x| !x)
            .expect("Out of registers!") - 1;

        self.registers_used[index] = true;
        index as u8
    }

    pub fn acquire_return_reg(&mut self) -> vm::Register {
        assert!(!self.registers_used[vm::RETURN_REGISTER as usize]);
        self.registers_used[vm::RETURN_REGISTER as usize] = true;
        vm::RETURN_REGISTER
    }

    pub fn release(&mut self, reg: vm::Register) {
        assert!(self.registers_used[reg as usize]);
        self.registers_used[reg as usize] = false;
    }
}

struct StackAllocator {
    current_stack_offset: u64,
    symbol_lookup: HashMap<asg::SymbolKey, u64>,
}

impl StackAllocator {
    fn new() -> Self {
        Self {
            current_stack_offset: 0,
            symbol_lookup: HashMap::new(),
        }
    }

    fn allocate(&mut self, size : u64) -> u64 {
        let offset = self.current_stack_offset;
        self.current_stack_offset += size;
        offset
    }

    fn allocate_symbol(&mut self, symbol: asg::SymbolKey, size: u64) {
        assert!(!self.symbol_lookup.contains_key(&symbol));
        
        let offset = self.allocate(size);
        self.symbol_lookup.insert(symbol, offset);
    }

    fn get_symbol_offset(&self, symbol: &asg::SymbolKey) -> u64 {
        *self.symbol_lookup.get(symbol).unwrap()
    }

    fn reset(&mut self) {
        self.current_stack_offset = 0;
    }
}

struct CodeGenContext {
    pub module_init_lookup: HashMap<asg::ModuleKey, vm::InstrAddr>,
    pub function_lookup: HashMap<asg::FunctionRef, vm::InstrAddr>,
    pub stack_allocator: StackAllocator,

    pub register_allocator: RegisterAllocator,
}

impl CodeGenContext {
    pub fn new() -> Self {
        Self {
            module_init_lookup: HashMap::new(),
            function_lookup: HashMap::new(),
            stack_allocator: StackAllocator::new(),
            register_allocator: RegisterAllocator::new(),
        }
    }
}

fn generate_expression(
    builder: &mut vm::ProgramBuilder,
    context: &mut CodeGenContext,
    asg: &asg::Asg,
    scoperef: &asg::ScopeRef,
    expressionkey: &asg::ExpressionKey,
    islhs: bool, // TODO: This is not pretty, bu we need to know when to deref symbol refs
) -> vm::Register {
    let scope = get_scope(asg, scoperef);
    match &scope.expressions.get(expressionkey).object {
        asg::ExpressionObject::Literal(n) => match n {
            asg::expressions::Literal::StringLiteral(n) => {
                let reg = context.register_allocator.acquire();
                let lit_type = scope.expressiontypes.get(&expressionkey).unwrap();

                assert!(lit_type.is_primitive(&crate::typesystem::PrimitiveType::StaticStringUtf8), "String literals assume static utf8 strings for now, was {:?}", lit_type);
                
                // Create string const data
                let strhandle = create_constdata_utf8_static_string(builder, n.string.as_str());

                builder.load_const_address(reg, strhandle.0);
                reg
            },
            asg::expressions::Literal::BoolLiteral(_) => todo!(),
            asg::expressions::Literal::IntegerLiteral(n) => {
                let reg = context.register_allocator.acquire();

                let lit_type = scope.expressiontypes.get(&expressionkey).unwrap();

                // TODO: Assume u32 for now
                assert!(lit_type.is_primitive(&crate::typesystem::PrimitiveType::U32), "Integer literals assume u32 for now, was {:?}", lit_type);

                
                builder.load_u64(reg, n.data);
                reg
            }
            asg::expressions::Literal::StructLiteral(_) => todo!(),
            asg::expressions::Literal::FunctionLiteral(_) => todo!(),
            asg::expressions::Literal::ModuleLiteral(_) => todo!(),
        },
        asg::ExpressionObject::BuiltInFunction(n) => {
            // TODO: The type really should take care of this, but store enum in reg for now
            let reg = context.register_allocator.acquire();
            builder.load_u64(reg, n.function as u64);
            reg
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
                    let stack_offset = context.stack_allocator.get_symbol_offset(&n.symbol);

                    let reg = context.register_allocator.acquire();
                    builder.load_stack_address(reg, stack_offset as u64);

                    // For right-hand-side expressions, we want to deref the ref further
                    // TODO: How does this work for structs? We need to annotate call
                    //  args with how they are passed
                    if !islhs {
                        builder.load_reg64(reg, reg);
                    }
                    reg
                }
                asg::SymbolReference::UnresolvedReference(n) => {
                    panic!("Unresolved reference! {:?}", n)
                }
            }
        }
        asg::ExpressionObject::If(_) => todo!(),
        asg::ExpressionObject::Call(n) => {
            let callablereg = generate_expression(builder, context, asg, scoperef, &n.callable, false);

            let callabletype = scope.expressiontypes.get(&n.callable).unwrap();

            let mut args : Vec<(vm::Register, &crate::typesystem::TypeId)> = Vec::new();
            for arg in &n.args {
                let reg = generate_expression(builder, context, asg, scoperef, arg, false);
                let argtype = scope.expressiontypes.get(arg).unwrap();
                args.push((reg, argtype));
            }

            match callabletype {
                crate::typesystem::TypeId::BuiltInFunction(n) => {
                    match n {
                        crate::typesystem::BuiltInFunction::PrintFormat => {
                            // Don't need callable for built ins
                            context.register_allocator.release(callablereg);

                            assert!(args.len() >= 1);

                            // Move first argument into call reg 0 and release original
                            builder.move_reg64(context.register_allocator.acquire_param(0), args[0].0);
                            context.register_allocator.release(args[0].0);

                            // Write arg count as second argument
                            builder.load_u64(context.register_allocator.acquire_param(1), (args.len() - 1) as u64);

                            // Generate typed values for rest of arguments and store in param registers
                            let mut index = 2;
                            for arg in &args[1..] {
                                // Typed value is (currently) primitive type id as u64 and register content
                                const U64SIZE : u64 = std::mem::size_of::<u64>() as u64;
                                let stack_offset = context.stack_allocator.allocate(2 * U64SIZE);

                                let argvalue = arg.0;
                                let argtype_id = arg.1.type_id();

                                // Use final reg as working register as well
                                let reg = context.register_allocator.acquire_param(index);
                                
                                // Store pair of type and value on stack, leaving reg pointing to beginning of allocation
                                builder.load_stack_address(reg, stack_offset + U64SIZE);
                                builder.store_reg64(reg, argvalue);
                                builder.load_stack_address(reg, stack_offset);
                                builder.store_u64(reg, argtype_id);
                                
                                // At this point reg is done for call

                                // Release arg register
                                context.register_allocator.release(arg.0);

                                index += 1;
                            }

                            // Issue call
                            builder.call_builtin(crate::typesystem::BuiltInFunction::PrintFormat);

                            // Release param registers
                            for i in 0..index {
                                context.register_allocator.release(i as vm::Register);
                            }
                        },
                    }         
                },
                crate::typesystem::TypeId::Function(_) => panic!("User callables not yet supported!"),
                _ => panic!("Type {:?} not supported as callable", callabletype)
            }

            let reg = context.register_allocator.acquire();

            reg
        }
        asg::ExpressionObject::BinOp(_) => todo!(),
        asg::ExpressionObject::Subscript(_) => todo!(),
    }
}

fn generate_statement_body(
    builder: &mut vm::ProgramBuilder,
    context: &mut CodeGenContext,
    asg: &asg::Asg,
    scoperef: &asg::ScopeRef,
    body: &asg::StatementBody,
) {
    for stmnt in &body.statements {
        match stmnt {
            asg::Statement::If(_) => todo!(),
            asg::Statement::Return(_) => todo!(),
            asg::Statement::Initialize(n) => {
                // Declaration has been reserved on stack earlier
                let stack_offset = context.stack_allocator.get_symbol_offset(&asg::SymbolKey::from_str(n.symbol.as_str()));

                let lhsreg = context.register_allocator.acquire();
                builder.load_stack_address(lhsreg, stack_offset);

                // TODO: For now, assume register-wide value
                let rhsreg = generate_expression(builder, context, asg, scoperef, &n.expr, true);

                builder.store_reg64(lhsreg, rhsreg);

                context.register_allocator.release(lhsreg);
                context.register_allocator.release(rhsreg);
            }
            asg::Statement::Assign(n) => todo!(),
            asg::Statement::ExpressionWrapper(n) => {
                let reg = generate_expression(builder, context, asg, scoperef, &n.expr, false);

                // We are not interested in a return value here
                context.register_allocator.release(reg);
            }
        }
    }
}

pub fn generate_program(asg: &asg::Asg) -> vm::Program {
    let mut builder = vm::ProgramBuilder::new();
    let mut context = CodeGenContext::new();

    let entrypoint = {
        let builder = &mut builder;
        let context = &mut context;

        for modulekey in asg.modulestore.keys() {
            let module = asg.modulestore.get(&modulekey);

            // Module init
            if let Some(body) = &module.body {
                context.stack_allocator.reset();
                    
                let scoperef = &asg::ScopeRef {
                    module: modulekey,
                    scope: body.scope_nonowned,
                };
                let scope = get_scope(asg, &scoperef);

                for declkey in scope.symboltable.declarations.keys() {
                    let decl = scope.symboltable.declarations.get(&declkey);
                    let decltype = scope.declarationtypes.get(&declkey).unwrap();
                    context.stack_allocator.allocate_symbol(declkey.clone(), decltype.size());
                }

                let module_init_address = builder.get_current_instruction_address();

                generate_statement_body(
                    builder,
                    context,
                    asg,
                    &asg::ScopeRef {
                        module: modulekey,
                        scope: body.scope_nonowned,
                    },
                    &body,
                );

                // Module inits are called like functions, return at the end
                builder.do_return();

                context.module_init_lookup.insert(modulekey, module_init_address);
            }

            // Generate all functions
            for functionkey in module.functionstore.keys() {
                let function = module.functionstore.get(&functionkey);

                if let Some(body) = &function.body {
                    context.stack_allocator.reset();
                    
                    let scoperef = &asg::ScopeRef {
                        module: modulekey,
                        scope: body.scope_nonowned,
                    };
                    let scope = get_scope(asg, &scoperef);

                    for declkey in scope.symboltable.declarations.keys() {
                        let decl = scope.symboltable.declarations.get(&declkey);
                        let decltype = scope.declarationtypes.get(&declkey).unwrap();
                        context.stack_allocator.allocate_symbol(declkey.clone(), decltype.size());
                    }

                    let function_address = builder.get_current_instruction_address();
    
                    generate_statement_body(
                        builder,
                        context,
                        asg,
                        &scoperef,
                        &body,
                    );
    
                    // Functions return at the end
                    builder.do_return();
    
                    context.function_lookup.insert(asg::FunctionRef {module: modulekey, function: functionkey}, function_address);
                }
            }
        }

        // Generate program initialization
        let program_init_address = builder.get_current_instruction_address();
        {        
            let callable_reg = context.register_allocator.acquire();

            // Call modules inits
            // TODO: Not in random order...
            for module_init_address in context.module_init_lookup.values() {
                // TODO: should be u64
                
                builder.load_u64(callable_reg, *module_init_address as u64);
                builder.call(callable_reg);
            }

            // Finally, call main
            builder.load_u64(callable_reg, context.function_lookup[&asg::FunctionRef { module: asg.global_module, function: asg.main }] as u64);
            builder.call(callable_reg);

            context.register_allocator.release(callable_reg);
        }

        program_init_address
    };

    builder.finish(entrypoint)

    /*
    // TODO: Hard-code generation for now!
    let mut builder = vm::ProgramBuilder::new();

    // Create const data
    let fmtstr = create_constdata_utf8_static_string(&mut builder, "Value of x: {}\n");

    // Store typed values on stack
    {
        //  Typed value is [typeid: u64, data: u64]

        // u32 literal "5"
        builder.load_stack_address(0, 0);
        builder.store_u64(0, crate::typesystem::PrimitiveType::U32 as u64);
        builder.load_stack_address(0, 8);
        builder.store_u64(0, 5u32 as u64);
    }

    // Prepare call #print_format(static_string, argcount, typed_value...)
    {
        // Load format string address into register 0
        builder.load_const_address(0, fmtstr.0);

        // Load arg count, located at stack address 0, into register 1
        builder.load_u8(1, 1);

        // Load typed value 5 address, located at stack address 0, into register 2
        builder.load_stack_address(2, 0);
    }

    // Issue call
    builder.call_builtin(crate::typesystem::BuiltInFunction::PrintFormat);

    builder.finish()
    */
}
