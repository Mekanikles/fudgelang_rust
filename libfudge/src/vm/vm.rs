pub type Register = u8;
pub type Val16 = u16;
pub type ConstDataHandle = (usize, usize);
pub type ConstMemPtr = *const u8;

use dyn_fmt::AsStrFormatExt;

trait Instruction {
    const OP: Op;
}

mod Instructions {
    use super::*;

    #[repr(packed)]
    pub struct LoadImmediate32 {
        pub target: Register,
        pub value: u32,
    }
    impl Instruction for LoadImmediate32 {
        const OP: Op = Op::LoadImmediate32;
    }

    #[repr(packed)]
    pub struct LoadConstAddress {
        pub target: Register,
        pub handle: ConstDataHandle,
    }
    impl Instruction for LoadConstAddress {
        const OP: Op = Op::LoadConstAddress;
    }

    #[repr(packed)]
    pub struct LoadStackAddress {
        pub target: Register,
        pub offset: usize,
    }
    impl Instruction for LoadStackAddress {
        const OP: Op = Op::LoadStackAddress;
    }

    #[repr(packed)]
    pub struct StoreImmediate64 {
        pub address_source: Register,
        pub value: u64,
    }
    impl Instruction for StoreImmediate64 {
        const OP: Op = Op::StoreImmediate64;
    }

    #[repr(packed)]
    pub struct CallBuiltIn {
        pub builtin: crate::typesystem::BuiltInFunction,
    }

    impl Instruction for CallBuiltIn {
        const OP: Op = Op::CallBuiltIn;
    }
}

#[repr(u8)]
pub enum Op {
    LoadImmediate32,
    LoadConstAddress,
    LoadStackAddress,
    StoreImmediate64,
    CallBuiltIn,
    Halt,
}

unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::core::slice::from_raw_parts((p as *const T) as *const u8, ::core::mem::size_of::<T>())
}

pub struct ProgramBuilder {
    pub constdata: Vec<u8>,
    pub bytecode: Vec<u8>,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        Self {
            constdata: Vec::<u8>::new(),
            bytecode: Vec::<u8>::new(),
        }
    }

    pub fn finish(mut self) -> Program {
        self.bytecode.push(Op::Halt as u8);

        Program {
            constdata: self.constdata,
            bytecode: self.bytecode,
        }
    }

    fn write_instruction<T: Instruction>(&mut self, instr: T) {
        self.bytecode.push(T::OP as u8);
        self.bytecode
            .extend_from_slice(unsafe { any_as_u8_slice(&instr) });
    }

    pub fn alloc_constdata(&mut self, size: usize) -> ConstDataHandle {
        let old_len = self.constdata.len();
        self.constdata.resize(old_len + size, 0);
        (old_len, old_len + size)
    }

    pub fn edit_constdata(&mut self, handle: &ConstDataHandle) -> &mut [u8] {
        self.constdata[handle.0..handle.1].as_mut()
    }

    pub fn load_u8(&mut self, target: Register, value: u8) {
        self.load_u32(target, value as u32)
    }

    pub fn load_u32(&mut self, target: Register, value: u32) {
        self.write_instruction(Instructions::LoadImmediate32 { target, value });
    }

    pub fn store_u64(&mut self, address_source: Register, value: u64) {
        self.write_instruction(Instructions::StoreImmediate64 {
            address_source,
            value,
        });
    }

    pub fn load_const_address(&mut self, target: Register, handle: ConstDataHandle) {
        self.write_instruction(Instructions::LoadConstAddress { target, handle });
    }

    pub fn load_stack_address(&mut self, target: Register, offset: usize) {
        self.write_instruction(Instructions::LoadStackAddress { target, offset });
    }

    pub fn call_builtin(&mut self, builtin: crate::typesystem::BuiltInFunction) {
        self.write_instruction(Instructions::CallBuiltIn { builtin });
    }
}

pub struct Program {
    pub constdata: Vec<u8>,
    pub bytecode: Vec<u8>,
}

pub struct Vm {
    pub registers: Vec<u64>,
    pub stack: Vec<u8>,
    pub pc: usize,
}

impl Vm {
    pub fn new() -> Self {
        assert!(std::mem::size_of::<Op>() == 1);
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

    fn read_op(&mut self) -> Op {
        let op = self.program.bytecode[self.vm.pc];
        self.vm.pc += 1;
        unsafe { std::mem::transmute::<u8, Op>(op) }
    }

    fn read_instruction<T: Instruction>(&mut self) -> &T {
        let ret: &T = unsafe { std::mem::transmute(&self.program.bytecode[self.vm.pc]) };
        self.vm.pc += std::mem::size_of::<T>();
        ret
    }

    fn reg_to_memptr(&self, reg: Register) -> ConstMemPtr {
        self.vm.registers[reg as usize] as usize as *const u8
    }

    unsafe fn deref_u64(&self, memptr: ConstMemPtr) -> u64 {
        let bytes: [u8; 8] = *(memptr as *const [u8; 8]);
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
            let fmtstrlen = self.deref_u64(fmtstrptr);

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
            let ptype: PrimitiveType = unsafe { std::mem::transmute(*typedvalueptr as u8) };
            let val = unsafe { self.deref_u64(typedvalueptr.offset(8)) };
            argstrings.push(primitive_to_string(ptype, val));
        }

        print!("{}", fmtstr.format(&argstrings));
    }

    pub fn run(&mut self) {
        loop {
            let op = self.read_op();
            match op {
                Op::LoadImmediate32 => {
                    let instr = self.read_instruction::<Instructions::LoadImmediate32>();
                    let target = instr.target as usize;
                    self.vm.registers[target] = instr.value as u64;
                }
                Op::LoadConstAddress => {
                    let const_base: usize =
                        unsafe { std::mem::transmute(&self.program.constdata[0]) };
                    let instr = self.read_instruction::<Instructions::LoadConstAddress>();

                    let target = instr.target as usize;
                    self.vm.registers[target] = (const_base + instr.handle.0) as u64;
                }
                Op::LoadStackAddress => {
                    // TODO: This should be a stack window base
                    let stack_base: usize = unsafe { std::mem::transmute(&self.vm.stack[0]) };
                    let instr = self.read_instruction::<Instructions::LoadStackAddress>();

                    let target = instr.target as usize;
                    self.vm.registers[target] = (stack_base + instr.offset) as u64;
                }
                Op::StoreImmediate64 => {
                    let instr = self.read_instruction::<Instructions::StoreImmediate64>();
                    let address_source = instr.address_source as usize;
                    let value = instr.value;
                    let ptr: *mut u64 =
                        unsafe { std::mem::transmute(self.vm.registers[address_source]) };

                    unsafe { *ptr = value };
                }
                Op::CallBuiltIn => {
                    let instr = self.read_instruction::<Instructions::CallBuiltIn>();
                    let builtin = instr.builtin;
                    self.call_builtin(&builtin);
                }
                Op::Halt => break,
            }
        }
    }
}

pub fn run(program: &Program) {
    let mut interpreter = Interpreter::new(program);
    interpreter.run();
}
