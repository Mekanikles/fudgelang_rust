use super::*;

const OP_BITMASK: u8 = 0b0011111;

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

    pub fn write_u64(&mut self, d: u64) {
        self.data.extend_from_slice(&d.to_be_bytes())
    }
}
