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
