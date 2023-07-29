use super::*;

use instructions::instructions;

pub struct ProgramBuilder {
    pub constdata: Vec<u8>,
    pub bytecode: ByteCodeChunk,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self {
            constdata: Vec::<u8>::new(),
            bytecode: ByteCodeChunk::new(),
        }
    }

    pub fn finish(mut self) -> Program {
        self.bytecode.write_u8(Op::Halt as u8);

        Program {
            constdata: self.constdata,
            bytecode: self.bytecode,
        }
    }

    fn write_instruction<T: Instruction>(&mut self, instr: T) {
        instr.encode(&mut self.bytecode);
    }

    pub fn alloc_constdata(&mut self, size: usize) -> ConstDataHandle {
        let old_len = self.constdata.len();
        self.constdata.resize(old_len + size, 0);
        (old_len as u64, size as u64)
    }

    pub fn edit_constdata(&mut self, handle: &ConstDataHandle) -> &mut [u8] {
        self.constdata[(handle.0 as usize)..(handle.0 + handle.1) as usize].as_mut()
    }

    pub fn load_u8(&mut self, target: Register, value: u8) {
        self.load_u32(target, value as u32)
    }

    pub fn load_u32(&mut self, target: Register, value: u32) {
        self.write_instruction(instructions::LoadImmediate32 { target, value });
    }

    pub fn store_u64(&mut self, address_source: Register, value: u64) {
        self.write_instruction(instructions::StoreImmediate64 {
            address_source,
            value,
        });
    }

    pub fn load_const_address(&mut self, target: Register, address: ConstDataAddr) {
        self.write_instruction(instructions::LoadConstAddress { target, address });
    }

    pub fn load_stack_address(&mut self, target: Register, offset: u64) {
        self.write_instruction(instructions::LoadStackAddress { target, offset });
    }

    pub fn call_builtin(&mut self, builtin: crate::typesystem::BuiltInFunction) {
        self.write_instruction(instructions::CallBuiltIn { builtin });
    }
}

pub struct Program {
    pub constdata: Vec<u8>,
    pub bytecode: ByteCodeChunk,
}
