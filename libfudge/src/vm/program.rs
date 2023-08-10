use super::*;

use instructions::instructions;

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

    pub fn finish(self, entrypoint: u64) -> Program {
        Program {
            constdata: self.constdata,
            bytecode: self.bytecode,
            entrypoint,
        }
    }

    fn write_instruction<T: Instruction>(&mut self, instr: T) {
        instr.encode(&mut self.bytecode);
    }

    pub fn get_current_instruction_address(&self) -> InstrAddr {
        self.bytecode.len() as u64
    }

    pub fn patch_address(&mut self, location: InstrAddr, address: InstrAddr) {
        let start = location as usize;
        let stop = location as usize + 8;
        self.bytecode
            .slice_mut(start, stop)
            .copy_from_slice(&address.to_be_bytes())
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
        self.load_u64(target, value as u64)
    }

    pub fn load_u64(&mut self, target: Register, value: u64) {
        self.write_instruction(instructions::LoadImmediate64 { target, value });
    }

    pub fn load_patchable_instruction_address(&mut self, target: Register) -> InstrAddr {
        let current_pc = self.get_current_instruction_address();
        let instr = instructions::LoadImmediate64 {
            target,
            value: u64::MAX,
        };
        let offset = instr.get_value_instruction_offset();
        self.write_instruction(instr);
        current_pc + offset
    }

    pub fn load_reg64(&mut self, target: Register, address_source: Register) {
        self.write_instruction(instructions::LoadReg64 {
            target,
            address_source,
        });
    }

    pub fn store_u64(&mut self, address_source: Register, value: u64) {
        self.write_instruction(instructions::StoreImmediate64 {
            address_source,
            value,
        });
    }

    pub fn store_reg64(&mut self, address_source: Register, value_source: Register) {
        self.write_instruction(instructions::StoreReg64 {
            address_source,
            value_source,
        });
    }

    pub fn move_reg64(&mut self, target: Register, source: Register) {
        self.write_instruction(instructions::MoveReg64 { target, source });
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

    pub fn call(&mut self, instruction_address_target: Register) {
        self.write_instruction(instructions::Call {
            instruction_address_target,
        });
    }

    pub fn do_return(&mut self) {
        self.write_instruction(instructions::Return {});
    }

    pub fn halt(&mut self) {
        self.write_instruction(instructions::Halt {});
    }
}

pub struct Program {
    pub constdata: Vec<u8>,
    pub bytecode: ByteCodeChunk,
    pub entrypoint: u64,
}

pub fn print_program(program: &Program) {
    const ROWLEN: usize = 20;

    println!("  Data:");
    {
        let cdata = &program.constdata;

        let mut index = 0;
        loop {
            let rowcap = std::cmp::min(index + ROWLEN, cdata.len());
            if index == rowcap {
                break;
            }
            print!("    {:#010X}:", index);

            print!("    ");
            // Row of hex pairs
            for i in index..rowcap {
                print!("{:02X} ", cdata[i]);
            }
            // Padding
            for _i in rowcap..index + ROWLEN {
                print!("   ");
            }
            print!("    // ");
            // As ascii
            for i in index..rowcap {
                print!(
                    "{}",
                    if (cdata[i] as char).is_ascii_graphic() {
                        cdata[i] as char
                    } else if (cdata[i] as char) == ' ' {
                        ' '
                    } else {
                        '·'
                    }
                );
            }
            print!("\n");

            index = rowcap;
        }
    }

    println!("  Instructions:");
    {
        let bc = &program.bytecode;
        let mut index = 0;
        while index < bc.len() {
            let index_before_decode = index;

            let op = bc.peek_op(&index);
            let str = match op {
                Op::LoadImmediate64 => {
                    format!(
                        "{:?}",
                        instructions::LoadImmediate64::decode(&bc, &mut index)
                    )
                }
                Op::LoadConstAddress => {
                    format!(
                        "{:?}",
                        instructions::LoadConstAddress::decode(&bc, &mut index)
                    )
                }
                Op::LoadStackAddress => {
                    format!(
                        "{:?}",
                        instructions::LoadStackAddress::decode(&bc, &mut index)
                    )
                }
                Op::StoreImmediate64 => {
                    format!(
                        "{:?}",
                        instructions::StoreImmediate64::decode(&bc, &mut index)
                    )
                }
                Op::CallBuiltIn => {
                    format!("{:?}", instructions::CallBuiltIn::decode(&bc, &mut index))
                }
                Op::Call => {
                    format!("{:?}", instructions::Call::decode(&bc, &mut index))
                }
                Op::Return => {
                    format!("{:?}", instructions::Return::decode(&bc, &mut index))
                }
                Op::Halt => {
                    format!("{:?}", instructions::Halt::decode(&bc, &mut index))
                }
                Op::StoreReg64 => {
                    format!("{:?}", instructions::StoreReg64::decode(&bc, &mut index))
                }
                Op::MoveReg64 => {
                    format!("{:?}", instructions::MoveReg64::decode(&bc, &mut index))
                }
                Op::LoadReg64 => {
                    format!("{:?}", instructions::LoadReg64::decode(&bc, &mut index))
                }
            };

            print!("    {:#010X}:", index_before_decode);

            print!("    ");
            // Row of hex pairs
            for i in bc.slice(index_before_decode, index) {
                print!("{:02X} ", i);
            }
            // Padding
            for _i in 0..std::cmp::min(ROWLEN - (index - index_before_decode), ROWLEN) {
                print!("   ");
            }
            print!("    // ");
            print!("{}", str);

            print!("\n");
        }
    }
}
