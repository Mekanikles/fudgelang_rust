use std::collections::HashMap;
use std::hash::Hash;

use crate::asg::objectstore::IndexedObjectStore;
use crate::source;
use crate::utils::objectstore::ObjectStore;

use crate::ir;
use crate::vm::{self, InstrAddr, StackOffset};

// TODO: u32 just to make them distinct for now
pub type AbstractRegister = u32;
pub type AbstractStackOffset = u32;

struct CodeGenFunctionInfo {
    vmkey: crate::vm::program::abstractvm::FunctionKey,
    basicblock_lookup: HashMap<ir::BasicBlockKey, crate::vm::program::abstractvm::ChunkKey>,
}

struct CodeGenContext {
    // Map ir constants to their location in the vm program
    constant_lookup: HashMap<ir::ConstantDataKey, crate::vm::program::abstractvm::ConstantKey>,

    // Map ir functions to their location in the vm program
    functioninfo_lookup: HashMap<ir::FunctionKey, CodeGenFunctionInfo>,
}

impl CodeGenContext {
    pub fn new() -> Self {
        Self {
            constant_lookup: HashMap::new(),
            functioninfo_lookup: HashMap::new(),
        }
    }

    pub fn register_constant(
        &mut self,
        irconstkey: ir::ConstantDataKey,
        vmconstkey: crate::vm::program::abstractvm::ConstantKey,
    ) {
        self.constant_lookup.insert(irconstkey, vmconstkey);
    }

    pub fn get_vmconstant(
        &self,
        irconstkey: ir::ConstantDataKey,
    ) -> crate::vm::program::abstractvm::ConstantKey {
        self.constant_lookup[&irconstkey]
    }

    pub fn get_vmfunction(
        &self,
        irfunckey: ir::ConstantDataKey,
    ) -> crate::vm::program::abstractvm::FunctionKey {
        self.functioninfo_lookup[&irfunckey].vmkey
    }

    pub fn register_function(
        &mut self,
        irfunckey: ir::FunctionKey,
        vmfunckey: crate::vm::program::abstractvm::FunctionKey,
    ) {
        self.functioninfo_lookup.insert(
            irfunckey,
            CodeGenFunctionInfo {
                vmkey: vmfunckey,
                basicblock_lookup: HashMap::new(),
            },
        );
    }
}

// TODO: Registers are actually not abstract yet, consider if this is necessary
struct AbstractRegisterAllocator {
    pub registers_used: [bool; 256],
}

impl AbstractRegisterAllocator {
    fn new() -> Self {
        Self {
            registers_used: [false; 256],
        }
    }

    fn is_used(&self, reg: AbstractRegister) -> bool {
        self.registers_used[reg as usize]
    }

    fn acquire_param(&mut self, index: usize) -> AbstractRegister {
        // Params are passed in registers 0+
        assert!(
            !self.registers_used[index],
            "Register {} already in use!",
            index
        );
        self.registers_used[index] = true;
        index as AbstractRegister
    }

    pub fn acquire(&mut self) -> AbstractRegister {
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
        index as AbstractRegister
    }

    pub fn acquire_return_reg(&mut self) -> vm::Register {
        assert!(!self.registers_used[vm::RETURN_REGISTER as usize]);
        self.registers_used[vm::RETURN_REGISTER as usize] = true;
        vm::RETURN_REGISTER
    }

    pub fn release(&mut self, reg: AbstractRegister) {
        assert!(self.is_used(reg));
        self.registers_used[reg as usize] = false;
    }
}

struct AbstractStorageManager {
    current_variable_storage: HashMap<ir::VariableKey, Storage>,
    register_allocator: AbstractRegisterAllocator,
    current_stack_offset: u64, // TODO: Handle re-using stack "holes"
}

impl AbstractStorageManager {
    pub fn new() -> Self {
        Self {
            current_variable_storage: HashMap::new(),
            register_allocator: AbstractRegisterAllocator::new(),
            current_stack_offset: 0,
        }
    }

    pub fn get_current_variable_storage(&self, variable: &ir::VariableKey) -> Storage {
        self.current_variable_storage[variable].clone()
    }

    pub fn allocate_stack(&mut self, size: u64) -> AbstractStackOffset {
        let offset = self.current_stack_offset;
        self.current_stack_offset += size;
        offset as AbstractStackOffset
    }

    pub fn acquire_register(&mut self) -> AbstractRegister {
        // TODO: Handle out-of-registers
        self.register_allocator.acquire()
    }

    pub fn release_register(&mut self, register: AbstractRegister) {
        self.register_allocator.release(register)
    }

    // TODO: We never release storage atm
    pub fn acquire_variable_storage(
        &mut self,
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
            let offset = self.allocate_stack(size);
            Storage::Stack { offset, size }
        };

        self.current_variable_storage
            .insert(*variablekey, storage.clone());
        storage
    }

    pub fn move_param_register_if_needed<'a>(
        &mut self,
        chunkeditor: &mut crate::vm::program::abstractvm::ChunkEditor<'a>,
        paramindex: usize,
        source_register: AbstractRegister,
    ) -> AbstractRegister {
        let target_register = paramindex as AbstractRegister;

        if target_register == source_register {
            return target_register;
        }

        if self.register_allocator.is_used(target_register) {
            // Target in use, need to move to other register
            // TODO: Deal with out-of-registers and spills
            let new_register = self.register_allocator.acquire();
            chunkeditor.move_reg(new_register, target_register);

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

        chunkeditor.move_reg(target_register, source_register);
        self.register_allocator.release(source_register);

        target_register
    }

    pub fn set_up_variable_as_call_param<'a>(
        &mut self,
        chunkeditor: &mut crate::vm::program::abstractvm::ChunkEditor<'a>,
        variable: &ir::VariableKey,
        paramindex: usize,
    ) {
        let storage = &self.current_variable_storage[variable];
        match storage {
            Storage::Register { register, size: _ } => {
                self.move_param_register_if_needed(chunkeditor, paramindex, *register);
            }
            Storage::Stack { offset, size } => {
                let temp = self.register_allocator.acquire();
                chunkeditor.load_stack_address(temp, *offset);
                // TODO: This is ABI stuff, how to pass parameters bigger than a register
                //  This should be handled more formally.
                if *size <= 8 {
                    // If value is a register or less, send actual value instead of address
                    chunkeditor.load_reg_sized(vm::size_to_opsize(*size), temp, temp);
                }

                self.move_param_register_if_needed(chunkeditor, paramindex, temp);
            }
        }
    }
}

#[derive(Clone)]
enum Storage {
    Register {
        register: AbstractRegister,
        size: u64,
    },
    Stack {
        offset: AbstractStackOffset,
        size: u64,
    },
}
fn populate_function(
    context: &mut CodeGenContext,
    functioneditor: &mut crate::vm::program::abstractvm::FunctionEditor,
    irfunction: &ir::Function,
) {
    let mut storagemanager = AbstractStorageManager::new();

    for blockkey in irfunction.basicblockstore.keys() {
        let block = irfunction.basicblockstore.get(&blockkey);

        let chunkkey = functioneditor.create_chunk();
        let mut chunkeditor = functioneditor.edit_chunk(chunkkey);

        for instr in &block.instructions {
            match instr {
                ir::Instruction::Assign(n) => {
                    let targetstorage =
                        storagemanager.acquire_variable_storage(irfunction, &n.variable);

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
                                        chunkeditor.move_reg(target, source)
                                    }
                                    Storage::Stack { offset, size } => {
                                        chunkeditor.load_stack_address(target, offset);
                                        chunkeditor.load_reg_sized(
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
                                            let irconstkey = *data as ir::ConstantDataKey;
                                            let vm_const_data = context.get_vmconstant(irconstkey);

                                            // Strings are weird primitives, they refer to const data
                                            chunkeditor.load_const_address(target, vm_const_data)
                                        }
                                        n => chunkeditor.load_sized(
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
                                        let tempreg = storagemanager.acquire_register();

                                        chunkeditor.load_stack_address(tempreg, offset);
                                        chunkeditor.store_reg_sized(
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
                                        let sourcereg = storagemanager.acquire_register();

                                        let targetreg = storagemanager.acquire_register();

                                        chunkeditor.load_stack_address(sourcereg, source_offset);
                                        chunkeditor.load_stack_address(targetreg, source_offset);
                                        chunkeditor.store_reg_sized(
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
                                        let reg = storagemanager.acquire_register();
                                        let reg2 = storagemanager.acquire_register();

                                        let opsize = vm::size_to_opsize(n.size());

                                        chunkeditor.load_sized(opsize, reg, *data);
                                        chunkeditor.load_stack_address(reg2, offset);
                                        chunkeditor.store_reg_sized(opsize, reg2, reg);
                                    }
                                    ir::Value::BuiltInFunction { builtin: _ } => todo!(),
                                    ir::Value::TypedValue { typeid, variable } => {
                                        let reg = storagemanager.acquire_register();
                                        chunkeditor.load_stack_address(reg, offset + 8);

                                        let source_storage =
                                            storagemanager.get_current_variable_storage(variable);

                                        match source_storage {
                                            Storage::Register { register, size } => {
                                                chunkeditor.store_reg64(reg, register);
                                            }
                                            Storage::Stack { offset, size } => {
                                                let reg2 = storagemanager.acquire_register();
                                                chunkeditor.load_stack_address(reg2, offset);
                                                chunkeditor.store_reg64(reg, reg2);
                                                storagemanager.release_register(reg2);
                                            }
                                        }

                                        chunkeditor.load_stack_address(reg, offset);
                                        chunkeditor.store_u64(reg, typeid.type_id());
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
                            &mut chunkeditor,
                            var,
                            paramindex,
                        );
                        paramindex += 1;
                    }

                    // Note: we probably will not have to spill registers here, since it's a built-in
                    // TODO: What should be done about return values

                    // Will parse call param registers internally
                    chunkeditor.call_builtin(n.builtin);

                    // TODO: Deal with return values
                }
                ir::Instruction::CallStatic(n) => {
                    let mut paramindex = 0;
                    assert!(n.args.len() < 255);
                    // Make sure args occupy call registers
                    for var in &n.args {
                        storagemanager.set_up_variable_as_call_param(
                            &mut chunkeditor,
                            var,
                            paramindex,
                        );
                        paramindex += 1;
                    }

                    // TODO: Spill all registers except call params, since we cannot trust registers
                    //  after the call

                    let vmfunctionkey = context.get_vmfunction(n.function);

                    // TODO: Prevent this causing spilling of param registers
                    let callreg = storagemanager.acquire_register();
                    chunkeditor.load_function_address(callreg, vmfunctionkey);

                    // Will parse call param registers internally
                    chunkeditor.call(callreg);

                    // TODO: Deal with return values
                }
                ir::Instruction::Return(n) => {
                    // TODO
                    assert!(n.values.len() == 0);
                    chunkeditor.do_return();
                }
                ir::Instruction::Halt(_) => {
                    chunkeditor.halt();
                }
            }
        }
    }
}

pub fn generate_program(irprogram: &ir::Program) -> crate::vm::program::abstractvm::Program {
    let mut programbuilder = crate::vm::program::abstractvm::ProgramBuilder::new();
    let mut context = CodeGenContext::new();

    // Generate constant data
    for constantdatakey in irprogram.constantdatastore.keys() {
        // TODO: Should remove from source here instead of copy
        let constantdata = irprogram.constantdatastore.get(&constantdatakey);

        let constant = crate::vm::program::abstractvm::Constant::new(constantdata.data.clone());
        let vmconstantkey = programbuilder.add_constant(constant);

        context.register_constant(constantdatakey, vmconstantkey);
    }

    // Register all functions
    for irfunctionkey in irprogram.functionstore.keys() {
        let irfunction = irprogram.functionstore.get(&irfunctionkey);
        let vmfunction = crate::vm::program::abstractvm::Function::new(irfunction.name.clone());

        let vmfunctionkey = programbuilder.create_function(vmfunction);

        context.register_function(irfunctionkey, vmfunctionkey)
    }

    // Populate all functions
    for irfunctionkey in irprogram.functionstore.keys() {
        let irfunction = irprogram.functionstore.get(&irfunctionkey);
        let vmfunctionkey = context.get_vmfunction(irfunctionkey);

        let mut functioneeditor = programbuilder.edit_function(vmfunctionkey);

        populate_function(&mut context, &mut functioneeditor, irfunction);
    }

    programbuilder.finish(context.get_vmfunction(irprogram.init))
}
