use super::*;
use instructions::instructions;

use dyn_fmt::AsStrFormatExt;

pub struct Interpreter<'a> {
    vm: Vm,
    program: &'a Program,
}

impl<'a> Interpreter<'a> {
    pub fn new(program: &'a Program) -> Self {
        Self {
            vm: Vm::new(program.entrypoint as usize),
            program: program,
        }
    }

    fn peek_op(&self) -> Op {
        self.program.bytecode.peek_op(&self.vm.pc)
    }

    fn read_instruction<T: Instruction>(&mut self) -> T {
        T::decode(&self.program.bytecode, &mut self.vm.pc)
    }

    fn reg_to_memptr(&self, reg: Register) -> ConstMemPtr {
        self.vm.registers[reg as usize] as usize as *const u8
    }

    // Store
    fn store_u8(&self, memptr: MutMemPtr, value: u8) {
        let bytes: &mut [u8; 1] = unsafe { &mut *(memptr as *mut [u8; 1]) };
        bytes.copy_from_slice(&value.to_be_bytes());
    }
    fn store_u16(&self, memptr: MutMemPtr, value: u16) {
        let bytes: &mut [u8; 2] = unsafe { &mut *(memptr as *mut [u8; 2]) };
        bytes.copy_from_slice(&value.to_be_bytes());
    }
    fn store_u32(&self, memptr: MutMemPtr, value: u32) {
        let bytes: &mut [u8; 4] = unsafe { &mut *(memptr as *mut [u8; 4]) };
        bytes.copy_from_slice(&value.to_be_bytes());
    }
    fn store_u64(&self, memptr: MutMemPtr, value: u64) {
        let bytes: &mut [u8; 8] = unsafe { &mut *(memptr as *mut [u8; 8]) };
        bytes.copy_from_slice(&value.to_be_bytes());
    }

    // Read
    fn read_u8(&self, memptr: ConstMemPtr) -> u8 {
        let bytes: [u8; 1] = unsafe { *(memptr as *const [u8; 1]) };
        u8::from_be_bytes(bytes)
    }
    fn read_u16(&self, memptr: ConstMemPtr) -> u16 {
        let bytes: [u8; 2] = unsafe { *(memptr as *const [u8; 2]) };
        u16::from_be_bytes(bytes)
    }
    fn read_u32(&self, memptr: ConstMemPtr) -> u32 {
        let bytes: [u8; 4] = unsafe { *(memptr as *const [u8; 4]) };
        u32::from_be_bytes(bytes)
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
                    panic!("String typed-value arguments to print_format not yet supported!")
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
                    // TODO: This is not correct, because of endianess, among other things
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

        const FMTSTR_REG: Register = 0;
        const DYNARG_COUNT_REG: Register = 1;
        const DYNARG_START_REG: Register = 2;

        let fmtstr: &str = unsafe {
            let fmtstrptr = self.reg_to_memptr(FMTSTR_REG);
            let fmtstrlen = self.read_u64(fmtstrptr);

            std::str::from_utf8(std::slice::from_raw_parts(
                fmtstrptr.offset(8),
                fmtstrlen as usize,
            ))
            .unwrap()
        };

        let argcount = self.vm.registers[DYNARG_COUNT_REG as usize] as u64;

        let mut argstrings = Vec::new();
        for i in 0..argcount as usize {
            let typedvalueptr = self.reg_to_memptr((DYNARG_START_REG + i as u8) as u8);
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
                Op::LoadImmediate => {
                    let instr = self.read_instruction::<instructions::LoadImmediate>();
                    let target = instr.target as usize;
                    self.vm.registers[target] = instr.value as u64;
                }
                Op::LoadReg => {
                    let instr = self.read_instruction::<instructions::LoadReg>();
                    let target = instr.target as usize;
                    let source = instr.address_source as usize;
                    let memptr: MutMemPtr =
                        unsafe { std::mem::transmute(self.vm.registers[source]) };

                    self.vm.registers[target] = match instr.opsize {
                        OpSize::Size8 => self.read_u8(memptr) as u64,
                        OpSize::Size16 => self.read_u16(memptr) as u64,
                        OpSize::Size32 => self.read_u32(memptr) as u64,
                        OpSize::Size64 => self.read_u64(memptr),
                    };
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
                Op::StoreImmediate => {
                    let instr = self.read_instruction::<instructions::StoreImmediate>();
                    let address_source = instr.address_source as usize;
                    let value = instr.value;
                    let memptr: MutMemPtr =
                        unsafe { std::mem::transmute(self.vm.registers[address_source]) };

                    match instr.opsize {
                        OpSize::Size8 => self.store_u8(memptr, value as u8),
                        OpSize::Size16 => self.store_u16(memptr, value as u16),
                        OpSize::Size32 => self.store_u32(memptr, value as u32),
                        OpSize::Size64 => self.store_u64(memptr, value),
                    };
                }
                Op::StoreReg => {
                    let instr = self.read_instruction::<instructions::StoreReg>();
                    let addressreg = instr.address_source as usize;
                    let valuereg = instr.value_source as usize;

                    let memptr: MutMemPtr =
                        unsafe { std::mem::transmute(self.vm.registers[addressreg]) };

                    let value = self.vm.registers[valuereg];

                    match instr.opsize {
                        OpSize::Size8 => self.store_u8(memptr, value as u8),
                        OpSize::Size16 => self.store_u16(memptr, value as u16),
                        OpSize::Size32 => self.store_u32(memptr, value as u32),
                        OpSize::Size64 => self.store_u64(memptr, value),
                    };
                }
                Op::MoveReg => {
                    let instr = self.read_instruction::<instructions::MoveReg>();
                    self.vm.registers[instr.target as usize] =
                        self.vm.registers[instr.source as usize];
                }
                Op::CallBuiltIn => {
                    let instr = self.read_instruction::<instructions::CallBuiltIn>();
                    let builtin = instr.builtin;
                    self.call_builtin(&builtin);
                }
                Op::Call => {
                    let instr = self.read_instruction::<instructions::Call>();
                    self.vm.registers[RETURN_REGISTER as usize] = self.vm.pc as u64;
                    let new_pc = self.vm.registers[instr.instruction_address_target as usize];
                    self.vm.pc = new_pc as usize;
                }
                Op::Return => {
                    self.read_instruction::<instructions::Return>();
                    self.vm.pc = self.vm.registers[RETURN_REGISTER as usize] as usize;
                }
                Op::Halt => {
                    self.read_instruction::<instructions::Halt>();
                    break;
                }
            }
        }
    }
}
