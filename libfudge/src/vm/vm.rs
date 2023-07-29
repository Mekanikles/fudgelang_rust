pub type Register = u8;
pub type Val16 = u16;
pub type ConstDataAddr = u64;
pub type ConstDataHandle = (ConstDataAddr, u64);
pub type ConstMemPtr = *const u8;
pub type MutMemPtr = *mut u8;

use dyn_fmt::AsStrFormatExt;

unsafe fn read_bytes_from_mem<const T: usize>(memptr: *const u8) -> &'static [u8; T] {
    &*(memptr as *const [u8; T])
}

pub struct ByteCodeChunk {
    data: Vec<u8>,
}

impl ByteCodeChunk {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn peek_op(&self, pc: &usize) -> Op {
        unsafe { std::mem::transmute::<u8, Op>(self.data[*pc] & OP_BITMASK) }
    }

    pub fn skip_op(&self, pc: &mut usize) {
        *pc += 1
    }

    pub fn read_register(&self, pc: &mut usize) -> Register {
        self.read_u8(pc)
    }

    fn read_bytes<const T: usize>(&self, pc: &mut usize) -> &[u8; T] {
        let ret = unsafe { read_bytes_from_mem(&self.data[*pc] as *const u8) };
        *pc += T;
        ret
    }

    pub fn read_u8(&self, pc: &mut usize) -> u8 {
        let d = self.data[*pc];
        *pc += 1;
        d
    }

    pub fn read_u32(&self, pc: &mut usize) -> u32 {
        u32::from_be_bytes(*self.read_bytes(pc))
    }

    pub fn read_u64(&self, pc: &mut usize) -> u64 {
        u64::from_be_bytes(*self.read_bytes(pc))
    }

    pub fn write_op(&mut self, op: Op) {
        self.write_u8(op as u8)
    }

    pub fn write_register(&mut self, reg: Register) {
        self.write_u8(reg as u8)
    }

    pub fn write_u8(&mut self, d: u8) {
        self.data.push(d)
    }

    pub fn write_u32(&mut self, d: u32) {
        self.data.extend_from_slice(&d.to_be_bytes())
    }

    pub fn write_u64(&mut self, d: u64) {
        self.data.extend_from_slice(&d.to_be_bytes())
    }
}

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
const OP_BITMASK: u8 = 0b0011111;

trait Instruction {
    const OP: Op;
    fn decode(data: &ByteCodeChunk, pc: &mut usize) -> Self;
    fn encode(&self, data: &mut ByteCodeChunk);
}

mod instructions {
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

pub struct Interpreter<'a> {
    vm: Vm,
    program: &'a Program,
}

impl<'a> Interpreter<'a> {
    pub fn new(program: &'a Program) -> Self {
        Self {
            vm: Vm::new(),
            program: program,
        }
    }

    fn peek_op(&self) -> Op {
        self.program.bytecode.peek_op(&self.vm.pc)
    }

    fn skip_op(&mut self) {
        self.program.bytecode.skip_op(&mut self.vm.pc)
    }

    fn read_instruction<T: Instruction>(&mut self) -> T {
        T::decode(&self.program.bytecode, &mut self.vm.pc)
    }

    fn reg_to_memptr(&self, reg: Register) -> ConstMemPtr {
        self.vm.registers[reg as usize] as usize as *const u8
    }

    fn store_u64(&self, memptr: MutMemPtr, value: u64) {
        let bytes: &mut [u8; 8] = unsafe { &mut *(memptr as *mut [u8; 8]) };
        bytes.copy_from_slice(&value.to_be_bytes());
    }

    fn read_u64(&self, memptr: ConstMemPtr) -> u64 {
        let bytes: [u8; 8] = unsafe { *(memptr as *const [u8; 8]) };
        u64::from_be_bytes(bytes)
    }

    fn call_builtin(&mut self, builtin: &crate::typesystem::BuiltInFunction) {
        match builtin {
            crate::typesystem::BuiltInFunction::PrintFormat => self.builtin_printformat(),
        }
    }

    fn builtin_printformat(&mut self) {
        use crate::typesystem::PrimitiveType;

        fn primitive_to_string(ptype: PrimitiveType, val: u64) -> String {
            match ptype {
                crate::typesystem::PrimitiveType::StaticStringUtf8 => {
                    panic!("String arguments to print_format not yet supported!")
                }
                PrimitiveType::Bool => format!("{}", (val != 0) as bool),
                PrimitiveType::U8 => format!("{}", val as u8),
                PrimitiveType::U16 => format!("{}", val as u16),
                PrimitiveType::U32 => format!("{}", val as u32),
                PrimitiveType::U64 => format!("{}", val as u64),
                PrimitiveType::S8 => format!("{}", val as i8),
                PrimitiveType::S16 => format!("{}", val as i16),
                PrimitiveType::S32 => format!("{}", val as i32),
                PrimitiveType::S64 => format!("{}", val as i64),
                PrimitiveType::F32 => {
                    // TODO: This is not correct, because of endianess, amongt other things
                    format!("{}", unsafe {
                        *((&(val as u32) as *const u32) as *const f32)
                    })
                }
                PrimitiveType::F64 => {
                    // TODO: This is not correct, because of endianess
                    format!("{}", unsafe { *((val as *const u64) as *const f64) })
                }
            }
        }

        let fmtstr: &str = unsafe {
            let fmtstrptr = self.reg_to_memptr(0);
            let fmtstrlen = self.read_u64(fmtstrptr);

            std::str::from_utf8(std::slice::from_raw_parts(
                fmtstrptr.offset(8),
                fmtstrlen as usize,
            ))
            .unwrap()
        };

        let argcount = self.vm.registers[1] as u64;

        let mut argstrings = Vec::new();
        for i in 0..argcount as usize {
            let typedvalueptr = self.reg_to_memptr((2 + i) as u8);
            let ptype: PrimitiveType =
                unsafe { std::mem::transmute(self.read_u64(typedvalueptr) as u8) };
            let val = self.read_u64(unsafe { typedvalueptr.offset(8) });
            argstrings.push(primitive_to_string(ptype, val));
        }

        print!("{}", fmtstr.format(&argstrings));
    }

    pub fn run(&mut self) {
        loop {
            let op = self.peek_op();
            match op {
                Op::LoadImmediate32 => {
                    let instr = self.read_instruction::<instructions::LoadImmediate32>();
                    let target = instr.target as usize;
                    self.vm.registers[target] = instr.value as u64;
                }
                Op::LoadConstAddress => {
                    let const_base: usize =
                        unsafe { std::mem::transmute(&self.program.constdata[0]) };
                    let instr = self.read_instruction::<instructions::LoadConstAddress>();

                    let target = instr.target as usize;
                    self.vm.registers[target] = (const_base + instr.address as usize) as u64;
                }
                Op::LoadStackAddress => {
                    // TODO: This should be a stack window base
                    let stack_base: usize = unsafe { std::mem::transmute(&self.vm.stack[0]) };
                    let instr = self.read_instruction::<instructions::LoadStackAddress>();

                    let target = instr.target as usize;
                    self.vm.registers[target] = (stack_base + instr.offset as usize) as u64;
                }
                Op::StoreImmediate64 => {
                    let instr = self.read_instruction::<instructions::StoreImmediate64>();
                    let address_source = instr.address_source as usize;
                    let value = instr.value;
                    let memptr: MutMemPtr =
                        unsafe { std::mem::transmute(self.vm.registers[address_source]) };

                    self.store_u64(memptr, value);
                }
                Op::CallBuiltIn => {
                    let instr = self.read_instruction::<instructions::CallBuiltIn>();
                    let builtin = instr.builtin;
                    self.call_builtin(&builtin);
                }
                Op::Halt => {
                    self.skip_op();
                    break;
                }
            }
        }
    }
}

pub fn run(program: &Program) {
    let mut interpreter = Interpreter::new(program);
    interpreter.run();
}
