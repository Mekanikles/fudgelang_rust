use std::collections::HashMap;
use std::hash::Hash;

use crate::source;
use crate::utils::objectstore::ObjectStore;

use crate::ir::{self, function, program, Variable, VariableKey};
use crate::vm::{self, InstrAddr, StackOffset};

struct CodeGenContext {
    // Map ir functions to their address in vm code
    function_address_map: HashMap<ir::FunctionKey, vm::InstrAddr>,
    // Keep track of function addresses that need to be patched when all function addresses are known
    function_address_patch_map: HashMap<vm::InstrAddr, ir::FunctionKey>,
}

impl CodeGenContext {
    pub fn new() -> Self {
        Self {
            function_address_map: HashMap::new(),
            function_address_patch_map: HashMap::new(),
        }
    }

    pub fn record_function_address(
        &mut self,
        irfunckey: ir::FunctionKey,
        vmaddress: vm::InstrAddr,
    ) {
        self.function_address_map.insert(irfunckey, vmaddress);
    }

    pub fn get_function_address(&self, irfunc: &ir::FunctionKey) -> Option<vm::InstrAddr> {
        self.function_address_map.get(&irfunc).copied()
    }

    pub fn patch_function_addresses(&self, programbuilder: &mut vm::ProgramBuilder) {
        for kvp in &self.function_address_patch_map {
            let function_address = self.get_function_address(&kvp.1).unwrap();
            let instr_address_to_patch = *kvp.0;

            programbuilder.patch_address(instr_address_to_patch, function_address);
        }
    }
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

    fn is_used(&self, reg: vm::Register) -> bool {
        self.registers_used[reg as usize]
    }

    fn acquire_param(&mut self, index: usize) -> vm::Register {
        // Params are passed in registers 0+
        assert!(
            !self.registers_used[index],
            "Register {} already in use!",
            index
        );
        self.registers_used[index] = true;
        index as u8
    }

    pub fn acquire(&mut self) -> vm::Register {
        // Allocate temp registers from the bottom to avoid
        //  collisions with call arguments/return values
        // TODO: Bleh, but I don't know how to deal with reverse iterators
        let index = vm::RETURN_REGISTER as usize
            - self.registers_used[0..vm::RETURN_REGISTER as usize]
                .iter()
                .rev()
                .position(|&x| !x)
                .expect("Out of registers!")
            - 1;

        self.registers_used[index] = true;
        index as u8
    }

    pub fn acquire_return_reg(&mut self) -> vm::Register {
        assert!(!self.registers_used[vm::RETURN_REGISTER as usize]);
        self.registers_used[vm::RETURN_REGISTER as usize] = true;
        vm::RETURN_REGISTER
    }

    pub fn release(&mut self, reg: vm::Register) {
        assert!(self.is_used(reg));
        self.registers_used[reg as usize] = false;
    }
}

struct StorageManager {
    current_variable_storage: HashMap<ir::VariableKey, Storage>,
    register_allocator: RegisterAllocator,
    current_stack_offset: u64, // TODO: Handle re-using stack "holes"
}

impl StorageManager {
    pub fn new() -> Self {
        Self {
            current_variable_storage: HashMap::new(),
            register_allocator: RegisterAllocator::new(),
            current_stack_offset: 0,
        }
    }

    pub fn get_current_variable_storage(&self, variable: &ir::VariableKey) -> Storage {
        self.current_variable_storage[variable].clone()
    }

    pub fn allocate_stack(&mut self, size: u64) -> StackOffset {
        let offset = self.current_stack_offset;
        self.current_stack_offset += size;
        offset
    }

    pub fn acquire_register(
        &mut self,
        _programbuilder: &mut vm::ProgramBuilder,
        _size: u64,
    ) -> vm::Register {
        // TODO: Handle out-of-registers
        self.register_allocator.acquire()
    }

    pub fn release_register(&mut self, register: vm::Register) {
        self.register_allocator.release(register);
    }

    // TODO: We never release storage atm
    pub fn acquire_variable_storage(
        &mut self,
        _programbuilder: &mut vm::ProgramBuilder,
        irfunction: &ir::Function,
        variablekey: &ir::VariableKey,
    ) -> Storage {
        assert!(!self.current_variable_storage.contains_key(variablekey));
        let variable = irfunction.variablestore.get(&variablekey);
        let size = variable.get_type().size();

        let storage = if (size <= 8) {
            // TODO: Handle out-of-registers
            let register = self.register_allocator.acquire();
            Storage::Register { register, size }
        } else {
            let stack_offset = self.allocate_stack(size);
            Storage::Stack {
                offset: stack_offset,
                size,
            }
        };

        self.current_variable_storage
            .insert(*variablekey, storage.clone());
        storage
    }

    pub fn move_register_if_needed(
        &mut self,
        programbuilder: &mut vm::ProgramBuilder,
        target_register: vm::Register,
        source_register: vm::Register,
    ) -> vm::Register {
        if target_register == source_register {
            return target_register;
        }

        if self.register_allocator.is_used(target_register) {
            // Target in use, need to move to other register
            // TODO: Deal with out-of-registers and spills
            let new_register = self.register_allocator.acquire();
            programbuilder.move_reg(new_register, target_register);

            // Update variable storage
            for kvp in &mut self.current_variable_storage {
                match kvp.1 {
                    Storage::Register { register, size: _ } => {
                        if *register == target_register {
                            *register = new_register;
                            break;
                        }
                    }
                    _ => {}
                }
            }
        }

        programbuilder.move_reg(target_register, source_register);
        self.register_allocator.release(source_register);

        target_register
    }

    pub fn set_up_variable_as_call_param(
        &mut self,
        programbuilder: &mut vm::ProgramBuilder,
        variable: &VariableKey,
        paramindex: usize,
    ) {
        let storage = &self.current_variable_storage[variable];
        match storage {
            Storage::Register { register, size: _ } => {
                self.move_register_if_needed(programbuilder, paramindex as vm::Register, *register);
            }
            Storage::Stack { offset, size } => {
                let temp = self.register_allocator.acquire();
                programbuilder.load_stack_address(temp, *offset);
                // TODO: This is ABI stuff, how to pass parameters bigger than a register
                //  This should be handled more formally.
                if *size <= 8 {
                    // If value is a register or less, send actual value instead of address
                    programbuilder.load_reg_sized(vm::size_to_opsize(*size), temp, temp);
                }

                self.move_register_if_needed(programbuilder, paramindex as vm::Register, temp);
            }
        }
    }
}

#[derive(Clone)]
enum Storage {
    Register { register: vm::Register, size: u64 },
    Stack { offset: vm::StackOffset, size: u64 },
}

fn generate_function(
    context: &mut CodeGenContext,
    programbuilder: &mut vm::ProgramBuilder,
    irprogram: &ir::Program,
    irfunction: &ir::Function,
) -> InstrAddr {
    let addr = programbuilder.get_current_instruction_address();

    let mut storagemanager = StorageManager::new();

    for blockkey in irfunction.basicblockstore.keys() {
        let block = irfunction.basicblockstore.get(&blockkey);

        for instr in &block.instructions {
            match instr {
                ir::Instruction::Assign(n) => {
                    let targetstorage = storagemanager.acquire_variable_storage(
                        programbuilder,
                        irfunction,
                        &n.variable,
                    );

                    // TODO: Exhaust expression to non-compound

                    match targetstorage {
                        Storage::Register {
                            register: target,
                            size: _, // TODO: Only 64-bit variants of instructions used atm
                        } => match &n.expression {
                            ir::Expression::Variable(n) => {
                                let sourcestorage = storagemanager.get_current_variable_storage(&n);
                                match sourcestorage {
                                    Storage::Register {
                                        register: source,
                                        size,
                                    } => {
                                        assert!(size <= 8);
                                        programbuilder.move_reg(target, source)
                                    }
                                    Storage::Stack { offset, size } => {
                                        programbuilder.load_stack_address(target, offset);
                                        programbuilder.load_reg_sized(
                                            vm::size_to_opsize(size),
                                            target,
                                            target,
                                        );
                                    }
                                }
                            }
                            ir::Expression::Constant(n) => {
                                assert!(n.get_size() <= 8);

                                match n {
                                    ir::Value::Primitive { ptype, data } => match ptype {
                                        crate::typesystem::PrimitiveType::StaticStringUtf8 => {
                                            // Copy const data to vm code
                                            let ir_const_data = irprogram
                                                .constantdatastore
                                                .get(&(*data as ir::ConstantDataKey));

                                            let vm_const_data = programbuilder
                                                .alloc_constdata(ir_const_data.data.len());
                                            programbuilder
                                                .edit_constdata(&vm_const_data)
                                                .copy_from_slice(&ir_const_data.data);

                                            // Strings are weird primitives, they refer to const data
                                            programbuilder
                                                .load_const_address(target, vm_const_data.0)
                                        }
                                        n => programbuilder.load_sized(
                                            vm::size_to_opsize(n.size()),
                                            target,
                                            *data,
                                        ),
                                    },
                                    ir::Value::BuiltInFunction { builtin: _ } => todo!(),
                                    ir::Value::TypedValue { typeid, variable } => {
                                        panic!("Cannot be stored in register!")
                                    }
                                };
                            }
                        },
                        Storage::Stack { offset, size } => match &n.expression {
                            ir::Expression::Variable(n) => {
                                let sourcestorage = storagemanager.get_current_variable_storage(&n);
                                match sourcestorage {
                                    Storage::Register {
                                        register: source,
                                        size,
                                    } => {
                                        assert!(size <= 8);
                                        let tempreg =
                                            storagemanager.acquire_register(programbuilder, size);

                                        programbuilder.load_stack_address(tempreg, offset);
                                        programbuilder.store_reg_sized(
                                            vm::size_to_opsize(size),
                                            tempreg,
                                            source,
                                        );

                                        storagemanager.release_register(tempreg);
                                    }
                                    Storage::Stack {
                                        offset: source_offset,
                                        size,
                                    } => {
                                        assert!(size <= 8);
                                        let sourcereg =
                                            storagemanager.acquire_register(programbuilder, size);

                                        let targetreg =
                                            storagemanager.acquire_register(programbuilder, size);

                                        programbuilder.load_stack_address(sourcereg, source_offset);
                                        programbuilder.load_stack_address(targetreg, source_offset);
                                        programbuilder.store_reg_sized(
                                            vm::size_to_opsize(size),
                                            targetreg,
                                            sourcereg,
                                        );
                                    }
                                }
                            }
                            ir::Expression::Constant(n) => {
                                match n {
                                    ir::Value::Primitive { ptype: n, data } => {
                                        let reg =
                                            storagemanager.acquire_register(programbuilder, 8);
                                        let reg2 =
                                            storagemanager.acquire_register(programbuilder, 8);

                                        let opsize = vm::size_to_opsize(n.size());

                                        programbuilder.load_sized(opsize, reg, *data);
                                        programbuilder.load_stack_address(reg2, offset);
                                        programbuilder.store_reg_sized(opsize, reg2, reg);
                                    }
                                    ir::Value::BuiltInFunction { builtin: _ } => todo!(),
                                    ir::Value::TypedValue { typeid, variable } => {
                                        let reg =
                                            storagemanager.acquire_register(programbuilder, 8);
                                        programbuilder.load_stack_address(reg, offset + 8);

                                        let source_storage =
                                            storagemanager.get_current_variable_storage(variable);

                                        match source_storage {
                                            Storage::Register { register, size } => {
                                                programbuilder.store_reg64(reg, register);
                                            }
                                            Storage::Stack { offset, size } => {
                                                let reg2 = storagemanager
                                                    .acquire_register(programbuilder, 8);
                                                programbuilder.load_stack_address(reg2, offset);
                                                programbuilder.store_reg64(reg, reg2);
                                                storagemanager.release_register(reg2);
                                            }
                                        }

                                        programbuilder.load_stack_address(reg, offset);
                                        programbuilder.store_u64(reg, typeid.type_id());
                                    }
                                };
                            }
                        },
                    }
                }
                ir::Instruction::CallBuiltIn(n) => {
                    let mut paramindex = 0;
                    assert!(n.args.len() < 255);
                    // Make sure args occupy call registers
                    for var in &n.args {
                        storagemanager.set_up_variable_as_call_param(
                            programbuilder,
                            var,
                            paramindex,
                        );
                        paramindex += 1;
                    }

                    // Note: we probably will not have to spill registers here, since it's a built-in
                    // TODO: What should be done about return values

                    // Will parse call param registers internally
                    programbuilder.call_builtin(n.builtin);

                    // TODO: Deal with return values
                }
                ir::Instruction::CallStatic(n) => {
                    let mut paramindex = 0;
                    assert!(n.args.len() < 255);
                    // Make sure args occupy call registers
                    for var in &n.args {
                        storagemanager.set_up_variable_as_call_param(
                            programbuilder,
                            var,
                            paramindex,
                        );
                        paramindex += 1;
                    }

                    // TODO: Spill all registers except call params, since we cannot trust registers
                    //  after the call

                    // TODO: Prevent this causing spilling of param registers
                    let callreg = storagemanager.acquire_register(programbuilder, 8);
                    let patch_address = programbuilder.load_patchable_instruction_address(callreg);

                    // Store patch address for later
                    context
                        .function_address_patch_map
                        .insert(patch_address, n.function);

                    // Will parse call param registers internally
                    programbuilder.call(callreg);

                    // TODO: Deal with return values
                }
                ir::Instruction::Return(n) => {
                    // TODO
                    assert!(n.values.len() == 0);

                    programbuilder.do_return();
                }
                ir::Instruction::Halt(_) => {
                    programbuilder.halt();
                }
            }
        }
    }

    addr
}

pub fn generate_program(irprogram: &ir::Program) -> vm::Program {
    let mut programbuilder = vm::ProgramBuilder::new();
    let mut context = CodeGenContext::new();

    // Generate all functions
    for functionkey in irprogram.functionstore.keys() {
        let function = irprogram.functionstore.get(&functionkey);
        let addr = generate_function(&mut context, &mut programbuilder, irprogram, function);

        context.record_function_address(functionkey, addr);
    }

    // Patch function addresses
    context.patch_function_addresses(&mut programbuilder);

    programbuilder.finish(context.get_function_address(&irprogram.init).unwrap())
}
