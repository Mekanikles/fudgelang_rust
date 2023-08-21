use super::*;

#[repr(u8)]
pub enum Op {
    Halt = 0, // End program. Practial to have as the zero value, in case of executing garbage memory.
    LoadImmediate, // Load value into register
    LoadReg,  // Load value at address in register into register
    LoadAddress, // Load stack address
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
    pub fn size_bits(&self) -> usize {
        match self {
            OpSize::Size8 => 8,
            OpSize::Size16 => 16,
            OpSize::Size32 => 32,
            OpSize::Size64 => 64,
        }
    }
    pub fn size_bytes(&self) -> usize {
        self.size_bits() / 8
    }
}

const __OPSIZE_INVARIANT: () = assert!((OpSize::Size64 as u8) < 4);
pub const OPSIZE_MASK: u8 = 0b11000000;

pub trait Instruction {
    const OP: Op;
    fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self;
}

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum LoadAddressMode {
    StackOffset,
    ConstantAddress,
    InstructionAddress,
}

pub mod abstractvm {
    use super::*;
    use vm::abstractvm::Config;

    pub trait InstructionTrait {
        const OP: Op;
        fn bytecode_size(&self) -> usize;
        fn encode<'a>(&self, data: &mut ByteCodeWriter<'a>, context: &ByteCodeGenContext);
    }

    #[derive(Debug)]
    pub enum Instruction {
        Halt(instructions::Halt),
        LoadImmediate(instructions::LoadImmediate<Config>),
        LoadReg(instructions::LoadReg<Config>),
        LoadAddress(instructions::LoadAddress<Config>),
        StoreImmediate(instructions::StoreImmediate<Config>),
        StoreReg(instructions::StoreReg<Config>),
        MoveReg(instructions::MoveReg<Config>),
        CallBuiltIn(instructions::CallBuiltIn),
        Call(instructions::Call<Config>),
        Return(instructions::Return),
    }

    impl Instruction {
        pub fn bytecode_size(&self) -> usize {
            match self {
                Instruction::Halt(n) => n.bytecode_size(),
                Instruction::LoadImmediate(n) => n.bytecode_size(),
                Instruction::LoadReg(n) => n.bytecode_size(),
                Instruction::LoadAddress(n) => n.bytecode_size(),
                Instruction::StoreImmediate(n) => n.bytecode_size(),
                Instruction::StoreReg(n) => n.bytecode_size(),
                Instruction::MoveReg(n) => n.bytecode_size(),
                Instruction::CallBuiltIn(n) => n.bytecode_size(),
                Instruction::Call(n) => n.bytecode_size(),
                Instruction::Return(n) => n.bytecode_size(),
            }
        }

        pub fn encode<'a>(&self, data: &mut ByteCodeWriter<'a>, context: &ByteCodeGenContext) {
            match self {
                Instruction::Halt(n) => n.encode(data, context),
                Instruction::LoadImmediate(n) => n.encode(data, context),
                Instruction::LoadReg(n) => n.encode(data, context),
                Instruction::LoadAddress(n) => n.encode(data, context),
                Instruction::StoreImmediate(n) => n.encode(data, context),
                Instruction::StoreReg(n) => n.encode(data, context),
                Instruction::MoveReg(n) => n.encode(data, context),
                Instruction::CallBuiltIn(n) => n.encode(data, context),
                Instruction::Call(n) => n.encode(data, context),
                Instruction::Return(n) => n.encode(data, context),
            }
        }

        pub fn to_string(&self) -> String {
            match self {
                Instruction::Halt(n) => n.to_string(),
                Instruction::LoadImmediate(n) => n.to_string(),
                Instruction::LoadReg(n) => n.to_string(),
                Instruction::LoadAddress(n) => n.to_string(),
                Instruction::StoreImmediate(n) => n.to_string(),
                Instruction::StoreReg(n) => n.to_string(),
                Instruction::MoveReg(n) => n.to_string(),
                Instruction::CallBuiltIn(n) => n.to_string(),
                Instruction::Call(n) => n.to_string(),
                Instruction::Return(n) => n.to_string(),
            }
        }
    }
}

pub mod instructions {
    use super::*;

    use vm::abstractvm::Config as AbstractVm;
    use vm::bytecodevm::Config as ByteCodeVm;

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
    pub struct LoadImmediate<Config: VmConfig> {
        pub opsize: OpSize,
        pub target: Config::RegisterType,
        pub value: Config::ValueType,
    }
    impl<Config: VmConfig> LoadImmediate<Config> {
        pub fn to_string(&self) -> String {
            columnize_output3(
                &format!("LoadIm{}", self.opsize.size_bits()),
                &format!("r{}", self.target),
                &format!("{}", self.value),
            )
        }
    }
    impl Instruction for LoadImmediate<ByteCodeVm> {
        const OP: Op = Op::LoadImmediate;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            let opsize = data.read_opsize(pc);
            Self {
                opsize,
                target: data.read_register(pc),
                value: data.read_sized_u64(pc, opsize),
            }
        }
    }
    impl abstractvm::InstructionTrait for LoadImmediate<AbstractVm> {
        const OP: Op = Op::LoadImmediate;

        fn bytecode_size(&self) -> usize {
            1 + 1 + self.opsize.size_bytes()
        }
        fn encode<'a>(&self, data: &mut ByteCodeWriter<'a>, context: &ByteCodeGenContext) {
            data.write_sized_op(Self::OP, self.opsize);
            data.write_register(context.resolve_register(&self.target));
            data.write_sized_u64(self.opsize, context.resolve_value(&self.value));
        }
    }

    #[derive(Debug)]
    pub struct LoadReg<Config: VmConfig> {
        pub opsize: OpSize,
        pub target: Config::RegisterType,
        pub address_source: Config::RegisterType,
    }
    impl<Config: VmConfig> LoadReg<Config> {
        pub fn to_string(&self) -> String {
            columnize_output3(
                &format!("LoadReg{}", self.opsize.size_bits()),
                &format!("r{}", self.target),
                &format!("*r{}", self.address_source),
            )
        }
    }
    impl Instruction for LoadReg<ByteCodeVm> {
        const OP: Op = Op::LoadReg;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            Self {
                opsize: data.read_opsize(pc),
                target: data.read_register(pc),
                address_source: data.read_register(pc),
            }
        }
    }
    impl abstractvm::InstructionTrait for LoadReg<AbstractVm> {
        const OP: Op = Op::LoadReg;

        fn bytecode_size(&self) -> usize {
            1 + 1 + 1
        }
        fn encode<'a>(&self, data: &mut ByteCodeWriter<'a>, context: &ByteCodeGenContext) {
            data.write_sized_op(Self::OP, self.opsize);
            data.write_register(context.resolve_register(&self.target));
            data.write_register(context.resolve_register(&self.address_source));
        }
    }
    /*
        #[derive(Debug)]
        pub struct LoadConstAddress<Config: VmConfig> {
            pub target: Config::RegisterType,
            pub address: Config::ConstDataReferenceType,
        }
        impl<Config: VmConfig> LoadConstAddress<Config> {
            pub fn to_string(&self) -> String {
                columnize_output3(
                    &format!("LoadConstA"),
                    &format!("r{}", self.target),
                    &format!("&cp[{}]", self.address),
                )
            }
        }
        impl Instruction for LoadConstAddress<ByteCodeVm> {
            const OP: Op = Op::LoadConstAddress;

            fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
                data.skip_op(pc);
                Self {
                    target: data.read_register(pc),
                    address: data.read_u64(pc),
                }
            }
        }

        impl abstractvm::InstructionTrait for LoadConstAddress<AbstractVm> {
            const OP: Op = Op::LoadConstAddress;

            fn bytecode_size(&self) -> usize {
                1 + 1 + 8
            }
            fn encode<'a>(&self, data: &mut ByteCodeWriter<'a>, context: &ByteCodeGenContext) {
                let address = context.get_constdata_address(self.address);

                data.write_op(Self::OP);
                data.write_register(self.target);
                data.write_u64(self.address);
            }
        }
    */
    #[derive(Debug)]
    pub struct LoadAddress<Config: VmConfig> {
        pub mode: LoadAddressMode,
        pub target: Config::RegisterType,
        pub value: Config::ValueType,
    }
    impl<Config: VmConfig> LoadAddress<Config> {
        pub fn to_string(&self) -> String {
            let loadstr = match self.mode {
                LoadAddressMode::StackOffset => format!("&sp[{}]", self.value),
                LoadAddressMode::ConstantAddress => format!("&cp[{}]", self.value),
                LoadAddressMode::InstructionAddress => format!("&ip[{}]", self.value),
            };

            columnize_output3(
                &format!("LoadAddress"),
                &format!("r{}", self.target),
                &loadstr,
            )
        }
    }
    impl Instruction for LoadAddress<ByteCodeVm> {
        const OP: Op = Op::LoadAddress;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {
                mode: unsafe { std::mem::transmute(data.read_u8(pc)) },
                target: data.read_register(pc),
                value: data.read_u64(pc),
            }
        }
    }
    impl abstractvm::InstructionTrait for LoadAddress<AbstractVm> {
        const OP: Op = Op::LoadAddress;

        fn bytecode_size(&self) -> usize {
            1 + 1 + 1 + 8
        }
        fn encode<'a>(&self, data: &mut ByteCodeWriter<'a>, context: &ByteCodeGenContext) {
            data.write_op(Self::OP);
            // TODO: We can pack mode into op instead
            data.write_u8(self.mode as u8);
            data.write_register(context.resolve_register(&self.target));
            data.write_u64(context.resolve_value(&self.value));
        }
    }

    #[derive(Debug)]
    pub struct StoreImmediate<Config: VmConfig> {
        pub opsize: OpSize,
        pub address_source: Config::RegisterType,
        pub value: Config::ValueType,
    }
    impl<Config: VmConfig> StoreImmediate<Config> {
        pub fn to_string(&self) -> String {
            columnize_output3(
                &format!("StoreIm{}", self.opsize.size_bits()),
                &format!("*r{}", self.address_source),
                &format!("{}", self.value),
            )
        }
    }
    impl Instruction for StoreImmediate<ByteCodeVm> {
        const OP: Op = Op::StoreImmediate;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            let opsize = data.read_opsize(pc);
            Self {
                opsize,
                address_source: data.read_register(pc),
                value: data.read_sized_u64(pc, opsize),
            }
        }
    }
    impl abstractvm::InstructionTrait for StoreImmediate<AbstractVm> {
        const OP: Op = Op::StoreImmediate;

        fn bytecode_size(&self) -> usize {
            1 + 1 + self.opsize.size_bytes()
        }
        fn encode<'a>(&self, data: &mut ByteCodeWriter<'a>, context: &ByteCodeGenContext) {
            data.write_sized_op(Self::OP, self.opsize);
            data.write_register(context.resolve_register(&self.address_source));
            data.write_sized_u64(self.opsize, context.resolve_value(&self.value));
        }
    }

    #[derive(Debug)]
    pub struct StoreReg<Config: VmConfig> {
        pub opsize: OpSize,
        pub address_source: Config::RegisterType,
        pub value_source: Config::RegisterType,
    }
    impl<Config: VmConfig> StoreReg<Config> {
        pub fn to_string(&self) -> String {
            columnize_output3(
                &format!("StoreReg{}", self.opsize.size_bits()),
                &format!("*r{}", self.address_source),
                &format!("r{}", self.value_source),
            )
        }
    }
    impl Instruction for StoreReg<ByteCodeVm> {
        const OP: Op = Op::StoreReg;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            Self {
                opsize: data.read_opsize(pc),
                value_source: data.read_register(pc),
                address_source: data.read_register(pc),
            }
        }
    }
    impl abstractvm::InstructionTrait for StoreReg<AbstractVm> {
        const OP: Op = Op::StoreReg;

        fn bytecode_size(&self) -> usize {
            1 + 1 + 1
        }
        fn encode<'a>(&self, data: &mut ByteCodeWriter<'a>, context: &ByteCodeGenContext) {
            data.write_sized_op(Self::OP, self.opsize);
            data.write_register(context.resolve_register(&self.value_source));
            data.write_register(context.resolve_register(&self.address_source));
        }
    }

    #[derive(Debug)]
    pub struct MoveReg<Config: VmConfig> {
        pub target: Config::RegisterType,
        pub source: Config::RegisterType,
    }
    impl<Config: VmConfig> MoveReg<Config> {
        pub fn to_string(&self) -> String {
            columnize_output3(
                &format!("MoveReg"),
                &format!("r{}", self.target),
                &format!("r{}", self.source),
            )
        }
    }
    impl Instruction for MoveReg<ByteCodeVm> {
        const OP: Op = Op::MoveReg;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {
                target: data.read_register(pc),
                source: data.read_register(pc),
            }
        }
    }
    impl abstractvm::InstructionTrait for MoveReg<AbstractVm> {
        const OP: Op = Op::MoveReg;

        fn bytecode_size(&self) -> usize {
            1 + 1 + 1
        }
        fn encode<'a>(&self, data: &mut ByteCodeWriter<'a>, context: &ByteCodeGenContext) {
            data.write_op(Self::OP);
            data.write_register(context.resolve_register(&self.target));
            data.write_register(context.resolve_register(&self.source));
        }
    }

    #[derive(Debug)]
    pub struct CallBuiltIn {
        pub builtin: crate::typesystem::BuiltInFunction,
    }
    impl CallBuiltIn {
        const __OP: Op = Op::CallBuiltIn;
        pub fn to_string(&self) -> String {
            columnize_output2(&format!("CallBI"), &format!("{:?}", self.builtin))
        }
    }
    impl Instruction for CallBuiltIn {
        const OP: Op = Op::CallBuiltIn;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {
                builtin: unsafe { std::mem::transmute(data.read_u8(pc)) },
            }
        }
    }
    impl abstractvm::InstructionTrait for CallBuiltIn {
        const OP: Op = Op::CallBuiltIn;

        fn bytecode_size(&self) -> usize {
            1 + 1
        }
        fn encode<'a>(&self, data: &mut ByteCodeWriter<'a>, context: &ByteCodeGenContext) {
            data.write_op(Self::__OP);
            data.write_u8(self.builtin as u8);
        }
    }

    #[derive(Debug)]
    pub struct Call<Config: VmConfig> {
        pub instruction_address_target: Config::RegisterType,
    }
    impl<Config: VmConfig> Call<Config> {
        pub fn to_string(&self) -> String {
            columnize_output2(
                &format!("Call"),
                &format!("*r{}", self.instruction_address_target),
            )
        }
    }
    impl Instruction for Call<ByteCodeVm> {
        const OP: Op = Op::Call;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {
                instruction_address_target: data.read_u8(pc),
            }
        }
    }
    impl abstractvm::InstructionTrait for Call<AbstractVm> {
        const OP: Op = Op::Call;

        fn bytecode_size(&self) -> usize {
            1 + 1
        }
        fn encode<'a>(&self, data: &mut ByteCodeWriter<'a>, context: &ByteCodeGenContext) {
            data.write_op(Self::OP);
            data.write_register(context.resolve_register(&self.instruction_address_target));
        }
    }

    #[derive(Debug)]
    pub struct Return {}
    impl Return {
        const __OP: Op = Op::Return;
        pub fn to_string(&self) -> String {
            columnize_output1(&format!("Return"))
        }
    }
    impl Instruction for Return {
        const OP: Op = Op::Return;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {}
        }
    }
    impl abstractvm::InstructionTrait for Return {
        const OP: Op = Op::Return;

        fn bytecode_size(&self) -> usize {
            1
        }
        fn encode<'a>(&self, data: &mut ByteCodeWriter<'a>, context: &ByteCodeGenContext) {
            data.write_op(Self::__OP);
        }
    }

    #[derive(Debug)]
    pub struct Halt {}
    impl Halt {
        const __OP: Op = Op::Halt;
        pub fn to_string(&self) -> String {
            columnize_output1(&format!("Halt"))
        }
    }
    impl Instruction for Halt {
        const OP: Op = Op::Halt;

        fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self {
            data.skip_op(pc);
            Self {}
        }
    }
    impl abstractvm::InstructionTrait for Halt {
        const OP: Op = Op::Halt;

        fn bytecode_size(&self) -> usize {
            1
        }
        fn encode<'a>(&self, data: &mut ByteCodeWriter<'a>, context: &ByteCodeGenContext) {
            data.write_op(Self::__OP);
        }
    }
}
