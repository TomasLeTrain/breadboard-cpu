#[derive(Debug, Clone)]
pub struct Opcode {
    pub step: u8,
    pub ir: u8,
    pub ir2: u8,
    pub not_vram_active: bool,
}

fn bit_transform(x: u32, x_bit: u32, y_bit: u32) -> u32 {
    if x & (1 << x_bit) == 0 { 0 } else { 1 << y_bit }
}

pub fn addr_to_opcode(addr: u32) -> Opcode {
    let not_vram_active = bit_transform(addr, 0, 0) != 0;

    let step = (bit_transform(addr, 13, 0)
        | bit_transform(addr, 14, 1)
        | bit_transform(addr, 15, 2)
        | bit_transform(addr, 16, 3)) as u8;

    // lower half
    let ir = (bit_transform(addr, 5, 3)
        | bit_transform(addr, 6, 2)
        | bit_transform(addr, 7, 1)
        | bit_transform(addr, 12, 0)
        | bit_transform(addr, 8, 4)
        | bit_transform(addr, 9, 5)
        | bit_transform(addr, 11, 6)
        | bit_transform(addr, 10, 7)) as u8;

    let ir2 = (bit_transform(addr, 1, 3)
        | bit_transform(addr, 2, 2)
        | bit_transform(addr, 3, 1)
        | bit_transform(addr, 4, 0)) as u8;

    Opcode {
        step,
        ir,
        ir2,
        not_vram_active,
    }
}

// one instruction opcode per instruction
#[derive(Debug)]
pub struct InstructionOpcode {
    pub ir: u8,
    pub ir2: Option<u8>,
}

impl InstructionOpcode {
    /// number of bytes needed to represent opcode in memory
    pub fn byte_size(&self) -> usize {
        if self.ir2.is_some() { 2 } else { 1 }
    }

    pub fn get_opcode_bytes(&self) -> Vec<u8> {
        if self.ir2.is_some() {
            vec![self.ir, self.ir2.unwrap()]
        } else {
            vec![self.ir]
        }
    }
}
