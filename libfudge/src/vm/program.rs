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
                    } else if (cdata[i] as char).is_ascii_whitespace() {
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
                Op::LoadImmediate32 => {
                    format!(
                        "{:?}",
                        instructions::LoadImmediate32::decode(&bc, &mut index)
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
                Op::Halt => {
                    format!("{:?}", instructions::Halt::decode(&bc, &mut index))
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
