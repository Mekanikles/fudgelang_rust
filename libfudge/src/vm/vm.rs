use super::*;

pub type Register = u8;
pub type Val16 = u16;
pub type ConstDataAddr = u64;
pub type ConstDataHandle = (ConstDataAddr, u64);
pub type InstrAddr = u64;
pub type ConstMemPtr = *const u8;
pub type MutMemPtr = *mut u8;

pub struct Vm {
    pub registers: Vec<u64>,
    pub stack: Vec<u8>,
    pub pc: usize,
}

impl Vm {
    pub fn new() -> Self {
        Self {
            registers: vec![0; 256],
            stack: vec![0; 10000], // TODO: make this more sensible
            pc: 0,
        }
    }
}

pub fn run(program: &Program) {
    let mut interpreter = Interpreter::new(program);
    interpreter.run();
}
