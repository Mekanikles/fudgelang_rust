use crate::asg;
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

    //  String is [bytelen: u64, data: &[u8]]
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

    pub fn acquire(&mut self) -> vm::Register {
        let index = self
            .registers_used
            .iter()
            .position(|&x| !x)
            .expect("Out of registers!");
        index as u8
    }

    pub fn release(&mut self, reg: vm::Register) {
        assert!(self.registers_used[reg as usize]);
        self.registers_used[reg as usize] = false;
    }
}

struct StackAllocator {
    pub current_stack_offset: usize,
    pub symbol_lookup: HashMap<asg::SymbolKey, usize>,
}

impl StackAllocator {
    fn new() -> Self {
        Self {
            current_stack_offset: 0,
            symbol_lookup: HashMap::new(),
        }
    }
}

struct CodeGenContext {
    pub module_init_lookup: HashMap<asg::ModuleKey, vm::InstrAddr>,
    pub stack_allocator: StackAllocator,

    pub register_allocator: RegisterAllocator,
}

impl CodeGenContext {
    pub fn new() -> Self {
        Self {
            module_init_lookup: HashMap::new(),
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
) -> vm::Register {
    let scope = get_scope(asg, scoperef);
    match &scope.expressions.get(expressionkey).object {
        asg::ExpressionObject::Literal(n) => match n {
            asg::expressions::Literal::StringLiteral(_) => todo!(),
            asg::expressions::Literal::BoolLiteral(_) => todo!(),
            asg::expressions::Literal::IntegerLiteral(_) => todo!(),
            asg::expressions::Literal::StructLiteral(_) => todo!(),
            asg::expressions::Literal::FunctionLiteral(_) => todo!(),
            asg::expressions::Literal::ModuleLiteral(_) => todo!(),
        },
        asg::ExpressionObject::BuiltInFunction(_) => todo!(),
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
                    let stack_offset = context.stack_allocator.symbol_lookup[&n.symbol];

                    let reg = context.register_allocator.acquire();
                    builder.load_stack_address(reg, stack_offset as u64);
                    reg
                }
                asg::SymbolReference::UnresolvedReference(n) => {
                    panic!("Unresolved reference! {:?}", n)
                }
            }
        }
        asg::ExpressionObject::If(_) => todo!(),
        asg::ExpressionObject::Call(_) => todo!(),
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
            asg::Statement::Initialize(_) => todo!(),
            asg::Statement::Assign(n) => {
                let lhsreg = generate_expression(builder, context, asg, scoperef, &n.lhs);
                let rhsreg = generate_expression(builder, context, asg, scoperef, &n.rhs);
            }
            asg::Statement::ExpressionWrapper(_) => todo!(),
        }
    }
}

pub fn generate_program(asg: &asg::Asg) -> vm::Program {
    let mut builder = vm::ProgramBuilder::new();
    let mut context = CodeGenContext::new();

    {
        let builder = &mut builder;
        let context = &mut context;

        for modulekey in asg.modulestore.keys() {
            let module = asg.modulestore.get(&modulekey);

            if let Some(body) = &module.body {
                let address = builder.get_current_instruction_address();

                let scope = module.scopestore.get(&body.scope_nonowned);
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

                context.module_init_lookup.insert(modulekey, address);
            }
        }
    }

    builder.finish()

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
