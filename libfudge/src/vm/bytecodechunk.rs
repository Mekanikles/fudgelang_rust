use super::*;

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

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn slice(&self, start: usize, stop: usize) -> &[u8] {
        &self.data[start..stop]
    }

    pub fn slice_mut(&mut self, start: usize, stop: usize) -> &mut [u8] {
        &mut self.data[start..stop]
    }

    pub fn peek_op(&self, pc: &usize) -> Op {
        unsafe { std::mem::transmute::<u8, Op>(self.data[*pc] & OP_MASK) }
    }

    pub fn skip_op(&self, pc: &mut usize) {
        *pc += 1
    }

    pub fn read_opsize(&self, pc: &mut usize) -> OpSize {
        let ret = self.read_u8(pc) & OPSIZE_MASK;
        unsafe { std::mem::transmute(ret >> 6) }
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

    pub fn read_u64(&self, pc: &mut usize) -> u64 {
        u64::from_be_bytes(*self.read_bytes(pc))
    }

    pub fn read_sized_u64(&self, pc: &mut usize, opsize: OpSize) -> u64 {
        match opsize {
            OpSize::Size8 => u8::from_be_bytes(*self.read_bytes(pc)) as u64,
            OpSize::Size16 => u16::from_be_bytes(*self.read_bytes(pc)) as u64,
            OpSize::Size32 => u32::from_be_bytes(*self.read_bytes(pc)) as u64,
            OpSize::Size64 => u64::from_be_bytes(*self.read_bytes(pc)),
        }
    }

    pub fn write_op(&mut self, op: Op) {
        self.write_u8(op as u8 & OP_MASK)
    }

    pub fn write_sized_op(&mut self, op: Op, size: OpSize) {
        let opbase = op as u8 & OP_MASK;
        let masked_size = ((size as u8) << 6) & OPSIZE_MASK;

        self.write_u8(opbase + masked_size)
    }

    pub fn write_register(&mut self, reg: Register) {
        self.write_u8(reg as u8)
    }

    pub fn write_u8(&mut self, d: u8) {
        self.data.push(d)
    }

    pub fn write_u64(&mut self, d: u64) {
        self.data.extend_from_slice(&d.to_be_bytes())
    }

    pub fn write_sized_u64(&mut self, opsize: OpSize, d: u64) {
        match opsize {
            OpSize::Size8 => self.data.extend_from_slice(&(d as u8).to_be_bytes()),
            OpSize::Size16 => self.data.extend_from_slice(&(d as u16).to_be_bytes()),
            OpSize::Size32 => self.data.extend_from_slice(&(d as u32).to_be_bytes()),
            OpSize::Size64 => self.data.extend_from_slice(&(d as u64).to_be_bytes()),
        }
    }
}
