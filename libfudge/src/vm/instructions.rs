use super::*;

#[repr(u8)]
pub enum Op {
    LoadImmediate32,
    LoadConstAddress,
    LoadStackAddress,
    StoreImmediate64,
    CallBuiltIn,
    Halt, // Need to be last enum variant
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

    pub struct LoadImmediate32 {
        pub target: Register,
        pub value: u32,
    }
    impl Instruction for LoadImmediate32 {
        const OP: Op = Op::LoadImmediate32;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {
                target: data.read_register(pc),
                value: data.read_u32(pc),
            }
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_op(Self::OP);
            data.write_register(self.target);
            data.write_u32(self.value);
        }
    }

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
}
