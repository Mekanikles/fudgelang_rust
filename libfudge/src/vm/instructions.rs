use super::*;

#[repr(u8)]
pub enum Op {
    Halt = 0, // End program. Practial to have as the zero value, in case of executing garbage memory.
    LoadImmediate, // Load value into register
    LoadReg,  // Load value at address in register into register
    LoadConstAddress, // Load const data address from offset
    LoadStackAddress, // Load stack address from offset
    StoreImmediate, // Store value at address in register
    StoreReg, // Store value in register at address in register
    MoveReg,  // Move value from register to register
    CallBuiltIn, // Call specified built-in function
    Call,     // Call function at address in register

    // Keep return as last instruction
    Return, // Set pc to instruction address in return register.
}

// Make sure op fits into 6 bits
const __OP_INVARIANT: () = assert!((Op::Return as u8) < 64);
pub const OP_MASK: u8 = 0b00111111;

// 2-bit size enum
#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum OpSize {
    Size8 = 0,
    Size16 = 1,
    Size32 = 2,
    Size64 = 3,
}
impl OpSize {
    pub fn size(&self) -> u64 {
        match self {
            OpSize::Size8 => 8,
            OpSize::Size16 => 16,
            OpSize::Size32 => 32,
            OpSize::Size64 => 64,
        }
    }
}

const __OPSIZE_INVARIANT: () = assert!((OpSize::Size64 as u8) < 4);
pub const OPSIZE_MASK: u8 = 0b11000000;

pub trait Instruction {
    const OP: Op;
    fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self;
    fn encode(&self, data: &mut ByteCodeChunk);
    fn to_string(&self) -> String;
}

pub mod instructions {
    use super::*;

    // I'm sure there's a better way of doing this, but meh
    fn columnize_output1(name: &str) -> String {
        columnize_output2(name, "".into())
    }
    fn columnize_output2(name: &str, arg1: &str) -> String {
        columnize_output3(name, arg1, "".into())
    }
    fn columnize_output3(name: &str, arg1: &str, arg2: &str) -> String {
        format!("{: <11} {: >5} {}", name, arg1, arg2)
    }

    #[derive(Debug)]
    pub struct LoadImmediate {
        pub opsize: OpSize,
        pub target: Register,
        pub value: u64,
    }
    impl Instruction for LoadImmediate {
        const OP: Op = Op::LoadImmediate;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            let opsize = data.read_opsize(pc);
            Self {
                opsize,
                target: data.read_register(pc),
                value: data.read_sized_u64(pc, opsize),
            }
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_sized_op(Self::OP, self.opsize);
            data.write_register(self.target);
            data.write_sized_u64(self.opsize, self.value);
        }
        fn to_string(&self) -> String {
            columnize_output3(
                &format!("LoadIm{}", self.opsize.size()),
                &format!("r{}", self.target),
                &format!("{}", self.value),
            )
        }
    }
    impl LoadImmediate {
        // This is used for address patching
        pub fn get_value_instruction_offset(&self) -> u64 {
            debug_assert!(self.opsize == OpSize::Size64);
            return 2; // Address is located two bytes into instruction
        }
    }

    #[derive(Debug)]
    pub struct LoadReg {
        pub opsize: OpSize,
        pub target: Register,
        pub address_source: Register,
    }
    impl Instruction for LoadReg {
        const OP: Op = Op::LoadReg;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            Self {
                opsize: data.read_opsize(pc),
                target: data.read_register(pc),
                address_source: data.read_register(pc),
            }
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_sized_op(Self::OP, self.opsize);
            data.write_register(self.target);
            data.write_register(self.address_source);
        }
        fn to_string(&self) -> String {
            columnize_output3(
                &format!("LoadReg{}", self.opsize.size()),
                &format!("r{}", self.target),
                &format!("*r{}", self.address_source),
            )
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
        fn to_string(&self) -> String {
            columnize_output3(
                &format!("LoadConst"),
                &format!("r{}", self.target),
                &format!("&cp[{}]", self.address),
            )
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
        fn to_string(&self) -> String {
            columnize_output3(
                &format!("StoreReg"),
                &format!("r{}", self.target),
                &format!("&sp[{}]", self.offset),
            )
        }
    }

    #[derive(Debug)]
    pub struct StoreImmediate {
        pub opsize: OpSize,
        pub address_source: Register,
        pub value: u64,
    }
    impl Instruction for StoreImmediate {
        const OP: Op = Op::StoreImmediate;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            let opsize = data.read_opsize(pc);
            Self {
                opsize,
                address_source: data.read_register(pc),
                value: data.read_sized_u64(pc, opsize),
            }
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_sized_op(Self::OP, self.opsize);
            data.write_register(self.address_source);
            data.write_sized_u64(self.opsize, self.value);
        }
        fn to_string(&self) -> String {
            columnize_output3(
                &format!("StoreIm{}", self.opsize.size()),
                &format!("*r{}", self.address_source),
                &format!("{}", self.value),
            )
        }
    }

    #[derive(Debug)]
    pub struct StoreReg {
        pub opsize: OpSize,
        pub address_source: Register,
        pub value_source: Register,
    }
    impl Instruction for StoreReg {
        const OP: Op = Op::StoreReg;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            Self {
                opsize: data.read_opsize(pc),
                value_source: data.read_register(pc),
                address_source: data.read_register(pc),
            }
        }
        fn encode(&self, data: &mut ByteCodeChunk) {
            data.write_sized_op(Self::OP, self.opsize);
            data.write_register(self.value_source);
            data.write_register(self.address_source);
        }
        fn to_string(&self) -> String {
            columnize_output3(
                &format!("StoreReg{}", self.opsize.size()),
                &format!("*r{}", self.address_source),
                &format!("r{}", self.value_source),
            )
        }
    }

    #[derive(Debug)]
    pub struct MoveReg {
        pub target: Register,
        pub source: Register,
    }
    impl Instruction for MoveReg {
        const OP: Op = Op::MoveReg;

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
        fn to_string(&self) -> String {
            columnize_output3(
                &format!("MoveReg"),
                &format!("r{}", self.target),
                &format!("r{}", self.source),
            )
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
        fn to_string(&self) -> String {
            columnize_output2(&format!("CallBI"), &format!("{:?}", self.builtin))
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
        fn to_string(&self) -> String {
            columnize_output2(
                &format!("Call"),
                &format!("{:#010X}", self.instruction_address_target),
            )
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
        fn to_string(&self) -> String {
            columnize_output1(&format!("Return"))
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
        fn to_string(&self) -> String {
            columnize_output1(&format!("Halt"))
        }
    }
}
