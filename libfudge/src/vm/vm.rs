use super::*;

pub type Register = u8;
pub type Val16 = u16;
pub type ConstDataAddr = u64;
pub type ConstDataHandle = (ConstDataAddr, u64);
pub type StackOffset = u64;
pub type InstrAddr = u64;
pub type ConstMemPtr = *const u8;
pub type MutMemPtr = *mut u8;

pub const RETURN_REGISTER: u8 = 255;

pub fn size_to_opsize(size: u64) -> OpSize {
    match size {
        1 => OpSize::Size8,
        2 => OpSize::Size16,
        4 => OpSize::Size32,
        8 => OpSize::Size64,
        _ => {
            panic!("Size {} not supported by ops", size)
        }
    }
}

pub trait VmConfig {
    type RegisterType: std::fmt::Display;
    type ValueType: std::fmt::Display;
    type StackOffsetType: std::fmt::Display;
}

pub mod bytecodevm {
    use super::*;

    #[derive(Debug)]
    pub struct Config;
    impl VmConfig for Config {
        type RegisterType = Register;
        type ValueType = u64;
        type StackOffsetType = u64;
    }

    pub struct Vm {
        pub registers: Vec<u64>,
        pub stack: Vec<u8>,
        pub pc: usize,
    }

    impl Vm {
        pub fn new(pc: usize) -> Self {
            Self {
                registers: vec![0; 256],
                stack: vec![0; 10000], // TODO: make this more sensible
                pc,
            }
        }
    }
}

pub mod abstractvm {
    use super::*;

    #[derive(Debug)]
    pub enum Value {
        ConstantAddress(crate::vm::program::abstractvm::ConstantKey),
        FunctionAddress(crate::vm::program::abstractvm::FunctionKey),
        Static(u64),
    }

    impl std::fmt::Display for Value {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    #[derive(Debug)]
    pub struct Config;
    impl VmConfig for Config {
        type RegisterType = crate::vmcodegen::AbstractRegister;
        type ValueType = Value;
        type StackOffsetType = crate::vmcodegen::AbstractStackOffset;
    }
}

pub fn run(program: &program::bytecodevm::Program) {
    let mut interpreter = Interpreter::new(program);
    interpreter.run();
}
