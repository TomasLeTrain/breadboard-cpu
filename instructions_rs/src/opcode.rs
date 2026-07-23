fn bitTransform(x: u32, x_bit: u32, y_bit: u32) -> u32 {
    if x & (1 << x_bit) == 0 { 0 } else { 1 << y_bit }
}

struct Opcode {
    step: u8,
    ir: u8,
    ir2: u8,
    not_vram_active: u8,
}

fn addrToOpcode(addr: u32) -> Opcode {
    let not_vram_active = bitTransform(addr, 0, 0) as u8;

    let step = (bitTransform(addr, 13, 0)
        | bitTransform(addr, 14, 1)
        | bitTransform(addr, 15, 2)
        | bitTransform(addr, 16, 3)) as u8;

    // lower half
    let ir = (bitTransform(addr, 5, 3)
        | bitTransform(addr, 6, 2)
        | bitTransform(addr, 7, 1)
        | bitTransform(addr, 12, 0)
        | bitTransform(addr, 8, 4)
        | bitTransform(addr, 9, 5)
        | bitTransform(addr, 11, 6)
        | bitTransform(addr, 10, 7)) as u8;

    let ir2 = (bitTransform(addr, 1, 3)
        | bitTransform(addr, 2, 2)
        | bitTransform(addr, 3, 1)
        | bitTransform(addr, 4, 0)) as u8;

    Opcode {
        step,
        ir,
        ir2,
        not_vram_active,
    }
}
