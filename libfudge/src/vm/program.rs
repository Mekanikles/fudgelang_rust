use std::{collections::HashMap, hash::Hash};

use crate::{asg::objectstore::ObjectStore, vm::program::abstractvm::ChunkKey};

use self::abstractvm::ConstantKey;

use super::*;

use instructions::instructions;

fn bytedata_to_ascii_string(data: &[u8], caplength: usize) -> String {
    let mut ret = String::new();

    let length = std::cmp::min(data.len(), caplength);
    for b in &data[0..length] {
        ret.push(if (*b as char).is_ascii_graphic() {
            *b as char
        } else if (*b as char) == ' ' {
            ' '
        } else {
            '·'
        });
    }
    ret
}

pub mod abstractvm {
    use super::*;
    use crate::asg::objectstore::{IndexedObjectStore, ObjectStore};
    use crate::vm::instructions::abstractvm::Instruction;
    use crate::vmcodegen::AbstractRegister as Register;
    use crate::vmcodegen::AbstractStackOffset as StackOffset;

    pub struct ChunkEditor<'a> {
        pub chunk: &'a mut Chunk,
    }

    impl<'a> ChunkEditor<'a> {
        pub fn new(chunk: &'a mut Chunk) -> Self {
            Self { chunk }
        }

        fn push_instr(&mut self, instr: Instruction) {
            self.chunk.instructions.push(instr)
        }

        pub fn store_sized(&mut self, opsize: OpSize, address_source: Register, value: u64) {
            self.push_instr(Instruction::StoreImmediate(instructions::StoreImmediate {
                opsize,
                address_source,
                value: vm::abstractvm::Value::Static(value),
            }));
        }

        pub fn load_sized(&mut self, opsize: OpSize, target: Register, value: u64) {
            self.push_instr(Instruction::LoadImmediate(instructions::LoadImmediate {
                opsize,
                target,
                value: vm::abstractvm::Value::Static(value),
            }));
        }

        pub fn load_reg_sized(
            &mut self,
            opsize: OpSize,
            target: Register,
            address_source: Register,
        ) {
            self.push_instr(Instruction::LoadReg(instructions::LoadReg {
                opsize,
                target,
                address_source,
            }));
        }

        pub fn store_reg_sized(
            &mut self,
            opsize: OpSize,
            address_source: Register,
            value_source: Register,
        ) {
            self.push_instr(Instruction::StoreReg(instructions::StoreReg {
                opsize,
                address_source,
                value_source,
            }));
        }

        pub fn move_reg_sized(&mut self, opsize: OpSize, target: Register, source: Register) {
            self.push_instr(Instruction::MoveReg(instructions::MoveReg {
                target,
                source,
            }));
        }

        /*pub fn get_current_instruction_address(&self) -> InstrAddr {
            self.bytecode.len() as u64
        }

        pub fn patch_address(&mut self, location: InstrAddr, address: InstrAddr) {
            let start = location as usize;
            let stop = location as usize + 8;
            self.bytecode
                .slice_mut(start, stop)
                .copy_from_slice(&address.to_be_bytes())
        }*/

        /*
        pub fn alloc_constdata(&mut self, size: usize) -> ConstDataHandle {
            let old_len = self.constdata.len();
            self.constdata.resize(old_len + size, 0);
            (old_len as u64, size as u64)
        }

        pub fn edit_constdata(&mut self, handle: &ConstDataHandle) -> &mut [u8] {
            self.constdata[(handle.0 as usize)..(handle.0 + handle.1) as usize].as_mut()
        }
        */

        /*pub fn load_patchable_instruction_address(&mut self, target: Register) -> InstrAddr {
            let current_pc = self.get_current_instruction_address();
            let instr = instructions::LoadImmediate {
                opsize: OpSize::Size64,
                target,
                value: u64::MAX,
            };
            let offset = instr.get_value_instruction_offset();
            self.push_instr(instr);
            current_pc + offset
        }*/

        // Load immediate
        pub fn load_u8(&mut self, target: Register, value: u8) {
            self.load_sized(OpSize::Size8, target, value as u64)
        }
        pub fn load_u16(&mut self, target: Register, value: u16) {
            self.load_sized(OpSize::Size16, target, value as u64)
        }
        pub fn load_u32(&mut self, target: Register, value: u32) {
            self.load_sized(OpSize::Size32, target, value as u64)
        }
        pub fn load_u64(&mut self, target: Register, value: u64) {
            self.load_sized(OpSize::Size64, target, value)
        }

        // Load reg
        pub fn load_reg8(&mut self, target: Register, address_source: Register) {
            self.load_reg_sized(OpSize::Size8, target, address_source)
        }
        pub fn load_reg16(&mut self, target: Register, address_source: Register) {
            self.load_reg_sized(OpSize::Size16, target, address_source)
        }
        pub fn load_reg32(&mut self, target: Register, address_source: Register) {
            self.load_reg_sized(OpSize::Size32, target, address_source)
        }
        pub fn load_reg64(&mut self, target: Register, address_source: Register) {
            self.load_reg_sized(OpSize::Size64, target, address_source)
        }

        // Store immediate
        pub fn store_u8(&mut self, address_source: Register, value: u8) {
            self.store_sized(OpSize::Size8, address_source, value as u64)
        }
        pub fn store_u16(&mut self, address_source: Register, value: u16) {
            self.store_sized(OpSize::Size16, address_source, value as u64)
        }
        pub fn store_u32(&mut self, address_source: Register, value: u32) {
            self.store_sized(OpSize::Size32, address_source, value as u64)
        }
        pub fn store_u64(&mut self, address_source: Register, value: u64) {
            self.store_sized(OpSize::Size64, address_source, value)
        }

        // Store reg
        pub fn store_reg8(&mut self, address_source: Register, value_source: Register) {
            self.store_reg_sized(OpSize::Size8, address_source, value_source)
        }
        pub fn store_reg16(&mut self, address_source: Register, value_source: Register) {
            self.store_reg_sized(OpSize::Size16, address_source, value_source)
        }
        pub fn store_reg32(&mut self, address_source: Register, value_source: Register) {
            self.store_reg_sized(OpSize::Size32, address_source, value_source)
        }
        pub fn store_reg64(&mut self, address_source: Register, value_source: Register) {
            self.store_reg_sized(OpSize::Size64, address_source, value_source)
        }

        // Move reg
        pub fn move_reg(&mut self, target: Register, source: Register) {
            self.push_instr(Instruction::MoveReg(instructions::MoveReg {
                target,
                source,
            }));
        }

        pub fn load_const_address(&mut self, target: Register, constant: ConstantKey) {
            self.push_instr(Instruction::LoadAddress(instructions::LoadAddress {
                mode: LoadAddressMode::ConstantAddress,
                target,
                value: vm::abstractvm::Value::ConstantAddress(constant),
            }));
        }

        pub fn load_function_address(&mut self, target: Register, function: FunctionKey) {
            // Note: Since "Call" knows we are dealing with instruction addresses,
            //  we can just load a static instruction offset here
            // TODO: This is pretty confusing, but it's nice for the Call instruction
            //  to not be able to jump to random data
            self.push_instr(Instruction::LoadImmediate(instructions::LoadImmediate {
                opsize: OpSize::Size64,
                target,
                value: vm::abstractvm::Value::FunctionAddress(function),
            }));
        }

        pub fn load_stack_address(&mut self, target: Register, offset: StackOffset) {
            self.push_instr(Instruction::LoadAddress(instructions::LoadAddress {
                mode: LoadAddressMode::StackOffset,
                target,
                value: vm::abstractvm::Value::Static(offset as u64),
            }));
        }

        pub fn call_builtin(&mut self, builtin: crate::typesystem::BuiltInFunction) {
            self.push_instr(Instruction::CallBuiltIn(instructions::CallBuiltIn {
                builtin,
            }));
        }

        pub fn call(&mut self, instruction_address_target: Register) {
            self.push_instr(Instruction::Call(instructions::Call {
                instruction_address_target,
            }));
        }

        pub fn do_return(&mut self) {
            self.push_instr(Instruction::Return(instructions::Return {}));
        }

        pub fn halt(&mut self) {
            self.push_instr(Instruction::Halt(instructions::Halt {}));
        }
    }

    pub struct FunctionEditor<'a> {
        function: &'a mut Function,
    }

    impl<'a> FunctionEditor<'a> {
        pub fn new(function: &'a mut Function) -> Self {
            Self { function }
        }

        pub fn create_chunk(&mut self) -> ChunkKey {
            self.function.chunkstore.add(Chunk {
                instructions: Vec::new(),
            })
        }

        pub fn edit_chunk(&mut self, key: ChunkKey) -> ChunkEditor {
            ChunkEditor::new(self.function.chunkstore.get_mut(&key))
        }
    }

    pub struct ProgramBuilder {
        functionstore: FunctionStore,
        constantstore: ConstantStore,
    }

    impl ProgramBuilder {
        pub fn new() -> Self {
            Self {
                functionstore: FunctionStore::new(),
                constantstore: ConstantStore::new(),
            }
        }

        pub fn create_function(&mut self, function: Function) -> FunctionKey {
            self.functionstore.add(function)
        }

        pub fn edit_function(&mut self, functionkey: FunctionKey) -> FunctionEditor {
            FunctionEditor::new(self.functionstore.get_mut(&functionkey))
        }

        pub fn add_constant(&mut self, constant: Constant) -> ConstantKey {
            self.constantstore.add(constant)
        }

        pub fn finish(self, init: FunctionKey) -> Program {
            Program {
                functionstore: self.functionstore,
                constantstore: self.constantstore,
                init,
            }
        }
    }

    pub type ChunkStore = IndexedObjectStore<Chunk>;
    pub type ChunkKey = usize;

    pub type FunctionStore = IndexedObjectStore<Function>;
    pub type FunctionKey = usize;

    pub type ConstantStore = IndexedObjectStore<Constant>;
    pub type ConstantKey = usize;

    pub struct Function {
        pub name: String,
        pub chunkstore: ChunkStore,
    }

    impl Function {
        pub fn new(name: String) -> Self {
            Self {
                name,
                chunkstore: ChunkStore::new(),
            }
        }
    }

    pub struct Chunk {
        pub instructions: Vec<abstractvm::Instruction>,
    }

    impl Chunk {
        pub fn calculate_bytecode_size(&self) -> usize {
            let mut size = 0;
            for instr in &self.instructions {
                size += instr.bytecode_size();
            }
            size
        }
    }

    pub struct Constant {
        pub data: Vec<u8>,
    }

    impl Constant {
        pub fn new(data: Vec<u8>) -> Self {
            Self { data }
        }
    }

    pub struct Program {
        pub functionstore: FunctionStore,
        pub constantstore: ConstantStore,
        pub init: FunctionKey,
    }

    pub fn print_program(program: &Program) {
        const ROWLEN: usize = 20;

        println!("  Constants:");
        {
            let cdatastore = &program.constantstore;

            for cdatakey in cdatastore.keys() {
                let cdata = cdatastore.get(&cdatakey);

                let headeranddata = format!(
                    "c{} - {}",
                    cdatakey,
                    bytedata_to_ascii_string(&cdata.data, 69)
                );

                // Data
                let length = std::cmp::min(69, cdata.data.len());
                println!("    {: <79}// Size:{}", headeranddata, cdata.data.len());
            }
        }

        println!("  Functions:");
        {
            for fkey in program.functionstore.keys() {
                let function = program.functionstore.get(&fkey);

                println!("    f{} - {}", fkey, function.name);

                for chunkkey in function.chunkstore.keys() {
                    let chunk = function.chunkstore.get(&chunkkey);

                    println!("      c{}:", chunkkey);

                    for instr in &chunk.instructions {
                        println!("        {}", instr.to_string())
                    }
                }
            }
        }
    }
}

pub mod bytecodevm {
    use super::*;

    pub struct Program {
        pub constdata: Vec<u8>,
        pub bytecode: ByteCodeChunk,
        pub entrypoint: u64,
    }

    pub fn print_program(program: &Program) {
        const ROWLEN: usize = 20;

        println!("  Data:");
        {
            let cdata = &program.constdata;

            let mut index = 0;
            loop {
                let rowcap = std::cmp::min(index + ROWLEN, cdata.len());
                if index == rowcap {
                    break;
                }
                print!("    {:#010X}:", index);

                print!("    ");
                // Row of hex pairs
                for i in index..rowcap {
                    print!("{:02X} ", cdata[i]);
                }
                // Padding
                for _i in rowcap..index + ROWLEN {
                    print!("   ");
                }
                print!("    // ");
                // As ascii
                print!(
                    "{}",
                    bytedata_to_ascii_string(&cdata[index..rowcap], ROWLEN).as_str()
                );

                print!("\n");

                index = rowcap;
            }
        }

        println!("  Instructions:");
        {
            let bc = &program.bytecode;
            let mut index = 0;
            while index < bc.len() {
                let index_before_decode = index;

                let op = bc.peek_op(&index);
                let str = match op {
                    Op::LoadImmediate => {
                        format!(
                            "{}",
                            instructions::LoadImmediate::decode(&bc, &mut index).to_string()
                        )
                    }
                    Op::LoadAddress => {
                        format!(
                            "{}",
                            instructions::LoadAddress::decode(&bc, &mut index).to_string()
                        )
                    }
                    Op::StoreImmediate => {
                        format!(
                            "{}",
                            instructions::StoreImmediate::decode(&bc, &mut index).to_string()
                        )
                    }
                    Op::CallBuiltIn => {
                        format!(
                            "{}",
                            instructions::CallBuiltIn::decode(&bc, &mut index).to_string()
                        )
                    }
                    Op::Call => format!(
                        "{}",
                        instructions::Call::decode(&bc, &mut index).to_string()
                    ),
                    Op::Return => {
                        format!(
                            "{}",
                            instructions::Return::decode(&bc, &mut index).to_string()
                        )
                    }
                    Op::Halt => format!(
                        "{}",
                        instructions::Halt::decode(&bc, &mut index).to_string()
                    ),
                    Op::StoreReg => {
                        format!(
                            "{}",
                            instructions::StoreReg::decode(&bc, &mut index).to_string()
                        )
                    }
                    Op::MoveReg => {
                        format!(
                            "{}",
                            instructions::MoveReg::decode(&bc, &mut index).to_string()
                        )
                    }
                    Op::LoadReg => {
                        format!(
                            "{}",
                            instructions::LoadReg::decode(&bc, &mut index).to_string()
                        )
                    }
                };

                print!("    {:#010X}:", index_before_decode);

                print!("    ");
                // Row of hex pairs
                for i in bc.slice(index_before_decode, index) {
                    print!("{:02X} ", i);
                }
                // Padding
                for _i in 0..std::cmp::min(ROWLEN - (index - index_before_decode), ROWLEN) {
                    print!("   ");
                }
                print!("    // ");
                print!("{}", str);

                print!("\n");
            }
        }
    }
}

pub struct ByteCodeGenFunctionInfo {
    pub address: InstrAddr,
    pub chunk_offset_map: HashMap<ChunkKey, (usize, usize)>,
}

pub struct ByteCodeGenContext {
    constaddress_lookup: HashMap<crate::vm::program::abstractvm::ConstantKey, ConstDataAddr>,
    functioninfo_lookup:
        HashMap<crate::vm::program::abstractvm::FunctionKey, ByteCodeGenFunctionInfo>,
}

impl ByteCodeGenContext {
    pub fn new() -> Self {
        Self {
            constaddress_lookup: HashMap::new(),
            functioninfo_lookup: HashMap::new(),
        }
    }

    pub fn get_constdata_address(
        &self,
        constkey: crate::vm::program::abstractvm::ConstantKey,
    ) -> ConstDataAddr {
        self.constaddress_lookup[&constkey]
    }

    pub fn get_function_address(
        &self,
        funckey: crate::vm::program::abstractvm::FunctionKey,
    ) -> InstrAddr {
        self.functioninfo_lookup[&funckey].address
    }

    pub fn resolve_value(&self, value: &vm::abstractvm::Value) -> u64 {
        match value {
            vm::abstractvm::Value::ConstantAddress(n) => self.get_constdata_address(*n),
            vm::abstractvm::Value::FunctionAddress(n) => self.get_function_address(*n),
            vm::abstractvm::Value::Static(n) => *n,
        }
    }

    pub fn resolve_register(&self, register: &crate::vmcodegen::AbstractRegister) -> vm::Register {
        // TODO
        *register as u8
    }
}

pub fn generate_bytecode(avmprogram: &abstractvm::Program) -> bytecodevm::Program {
    let mut context = ByteCodeGenContext::new();
    let mut constdata = Vec::<u8>::new();

    // Copy all const data and record addresses
    for constkey in avmprogram.constantstore.keys() {
        let data = avmprogram.constantstore.get(&constkey);

        pub fn write_constdata(targetdata: &mut Vec<u8>, sourcedata: &Vec<u8>) -> ConstDataAddr {
            let old_len = targetdata.len();
            let data_size = sourcedata.len();
            targetdata.append(&mut sourcedata.clone()); // TODO
            old_len as u64
        }

        let addr = write_constdata(&mut constdata, &data.data);
        context.constaddress_lookup.insert(constkey, addr);
    }

    // Measure and allocate blocks for all functions
    let mut total_instr_size: usize = 0;
    for functionkey in avmprogram.functionstore.keys() {
        let function = avmprogram.functionstore.get(&functionkey);

        let address = total_instr_size;
        let mut chunk_offset_map = HashMap::new();

        let mut chunk_offset = 0;
        for chunkkey in function.chunkstore.keys() {
            let chunk = function.chunkstore.get(&chunkkey);
            let chunk_start = chunk_offset;
            chunk_offset += chunk.calculate_bytecode_size();
            chunk_offset_map.insert(chunkkey, (chunk_start, chunk_offset));
        }

        context.functioninfo_lookup.insert(
            functionkey,
            ByteCodeGenFunctionInfo {
                address: address as InstrAddr,
                chunk_offset_map,
            },
        );
        total_instr_size += chunk_offset;
    }

    let mut bytecode = ByteCodeChunk::new(total_instr_size);

    // Write all instructions to bytecode
    for functionkey in avmprogram.functionstore.keys() {
        let function = avmprogram.functionstore.get(&functionkey);

        let functioninfo = &context.functioninfo_lookup[&functionkey];

        for chunkkey in function.chunkstore.keys() {
            let chunk = function.chunkstore.get(&chunkkey);

            let (chunk_start, chunk_stop) = functioninfo.chunk_offset_map[&chunkkey];

            let mut slice = bytecode.slice_mut(
                functioninfo.address as usize + chunk_start,
                functioninfo.address as usize + chunk_stop,
            );
            let mut bytecode_writer = ByteCodeWriter::new(&mut slice);

            for instr in &chunk.instructions {
                let offset_before = bytecode_writer.offset();
                instr.encode(&mut bytecode_writer, &context);
                let offset_after = bytecode_writer.offset();
                debug_assert!(
                    instr.bytecode_size() == offset_after - offset_before,
                    "Mismatch between written and declared size for instruction: {:?}",
                    instr
                );
            }
        }
    }

    bytecodevm::Program {
        constdata,
        bytecode,
        entrypoint: context.get_function_address(avmprogram.init),
    }
}
