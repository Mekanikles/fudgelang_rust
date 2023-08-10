use super::*;

#[repr(u8)]
pub enum Op {
    LoadImmediate64,  // Load value into register
    LoadReg64,        // Load value at address in register into register
    LoadConstAddress, // Load const data address from offset
    LoadStackAddress, // Load stack address from offset
    StoreImmediate64, // Store value at address in register
    StoreReg64,       // Store value in register at address in register
    MoveReg64,        // Move value from register to register
    CallBuiltIn,      // Call specified built-in function
    Call,             // Call function at address in register
    Return,           // Set pc to instruction address in return register
    Halt,             // End program. Need to be last enum variant
}

// Make sure op fits into 6 bits
const __OP_INVARIANT: () = assert!((Op::Halt as u8) < 64);

pub trait Instruction {
    const OP: Op;
    fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self;
    fn encode(&self, data: &mut ByteCodeChunk);
}

pub mod instructions {
    use super::*;

    #[derive(Debug)]
    pub struct LoadImmediate64 {
        pub target: Register,
        pub value: u64,
    }
    impl Instruction for LoadImmediate64 {
        const OP: Op = Op::LoadImmediate64;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {
                target: data.read_register(pc),
                value: data.read_u64(pc),
            }
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_op(Self::OP);
            data.write_register(self.target);
            data.write_u64(self.value);
        }
    }
    impl LoadImmediate64 {
        // This is used for address patching
        pub fn get_value_instruction_offset(&self) -> u64 {
            return 2; // Address is located two bytes into instruction
        }
    }

    #[derive(Debug)]
    pub struct LoadReg64 {
        pub target: Register,
        pub address_source: Register,
    }
    impl Instruction for LoadReg64 {
        const OP: Op = Op::LoadReg64;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {
                target: data.read_register(pc),
                address_source: data.read_register(pc),
            }
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_op(Self::OP);
            data.write_register(self.target);
            data.write_register(self.address_source);
        }
    }

    #[derive(Debug)]
    pub struct LoadConstAddress {
        pub target: Register,
        pub address: ConstDataAddr,
    }
    impl Instruction for LoadConstAddress {
        const OP: Op = Op::LoadConstAddress;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {
                target: data.read_register(pc),
                address: data.read_u64(pc),
            }
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_op(Self::OP);
            data.write_register(self.target);
            data.write_u64(self.address);
        }
    }

    #[derive(Debug)]
    pub struct LoadStackAddress {
        pub target: Register,
        pub offset: u64,
    }
    impl Instruction for LoadStackAddress {
        const OP: Op = Op::LoadStackAddress;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {
                target: data.read_register(pc),
                offset: data.read_u64(pc),
            }
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_op(Self::OP);
            data.write_register(self.target);
            data.write_u64(self.offset);
        }
    }

    #[derive(Debug)]
    pub struct StoreImmediate64 {
        pub address_source: Register,
        pub value: u64,
    }
    impl Instruction for StoreImmediate64 {
        const OP: Op = Op::StoreImmediate64;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {
                address_source: data.read_register(pc),
                value: data.read_u64(pc),
            }
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_op(Self::OP);
            data.write_register(self.address_source);
            data.write_u64(self.value);
        }
    }

    #[derive(Debug)]
    pub struct StoreReg64 {
        pub address_source: Register,
        pub value_source: Register,
    }
    impl Instruction for StoreReg64 {
        const OP: Op = Op::StoreReg64;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {
                value_source: data.read_register(pc),
                address_source: data.read_register(pc),
            }
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_op(Self::OP);
            data.write_register(self.value_source);
            data.write_register(self.address_source);
        }
    }

    #[derive(Debug)]
    pub struct MoveReg64 {
        pub target: Register,
        pub source: Register,
    }
    impl Instruction for MoveReg64 {
        const OP: Op = Op::MoveReg64;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {
                target: data.read_register(pc),
                source: data.read_register(pc),
            }
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_op(Self::OP);
            data.write_register(self.target);
            data.write_register(self.source);
        }
    }

    #[derive(Debug)]
    pub struct CallBuiltIn {
        pub builtin: crate::typesystem::BuiltInFunction,
    }
    impl Instruction for CallBuiltIn {
        const OP: Op = Op::CallBuiltIn;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {
                builtin: unsafe { std::mem::transmute(data.read_u8(pc)) },
            }
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_op(Self::OP);
            data.write_u8(self.builtin as u8);
        }
    }

    #[derive(Debug)]
    pub struct Call {
        pub instruction_address_target: Register,
    }
    impl Instruction for Call {
        const OP: Op = Op::Call;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {
                instruction_address_target: data.read_u8(pc),
            }
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_op(Self::OP);
            data.write_u8(self.instruction_address_target as u8);
        }
    }

    #[derive(Debug)]
    pub struct Return {}
    impl Instruction for Return {
        const OP: Op = Op::Return;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {}
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_op(Self::OP);
        }
    }

    #[derive(Debug)]
    pub struct Halt {}
    impl Instruction for Halt {
        const OP: Op = Op::Halt;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {}
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_op(Self::OP);
        }
    }
}
