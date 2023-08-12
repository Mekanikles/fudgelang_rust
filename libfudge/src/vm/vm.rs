use super::*;

pub type Register = u8;
pub type Val16 = u16;
pub type ConstDataAddr = u64;
pub type ConstDataHandle = (ConstDataAddr, u64);
pub type StackOffset = u64;
pub type InstrAddr = u64;
pub type ConstMemPtr = *const u8;
pub type MutMemPtr = *mut u8;

pub struct Vm {
    pub registers: Vec<u64>,
    pub stack: Vec<u8>,
    pub pc: usize,
}

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

impl Vm {
    pub fn new(pc: usize) -> Self {
        Self {
            registers: vec![0; 256],
            stack: vec![0; 10000], // TODO: make this more sensible
            pc,
        }
    }
}

pub fn run(program: &Program) {
    let mut interpreter = Interpreter::new(program);
    interpreter.run();
}
