use crate::Action::*;
use crate::instructions::defs::*;
use crate::instructions::istr_utils::{InstructionImpl, SimpleInstruction, VramInstruction};
use crate::instructions::register_defs::*;
use crate::instructions::*;

// move register to register (reg0 = reg1)
pub fn move_word_reg_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    let base_template = vec![
        [Reg1Bout, Reg0Write, PC.cnt, Reset].into(), // read from reg1 to reg0, pc cnt
    ];

    for reg0 in Register::write_iterator() {
        // excludes PC, avoids duplicates
        let rhs = Register::read_iterator()
            .filter(|e| !matches!(e, Register::PcLo | Register::PcHi))
            .filter(|&e| e != reg0);
        for reg1 in rhs {
            let mut current = base_template.clone();

            // if writing to PC then don't increment
            if matches!(reg0, Register::PcLo | Register::PcHi) {
                replace_action(&mut current, PC.cnt, Nop);
            }

            reg0.fill_reg0(&mut current);
            reg1.fill_reg1(&mut current);

            assert!(all_regs_filled(&current));

            result.push(InstructionImpl::Simple(SimpleInstruction {
                istr: current,
                name: format!("mv {}, {}", reg0.name(), reg1.name()),
            }));
        }
    }
    result
}

// move imm8 to register (reg0 = imm8)
pub fn move_word_imm_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    let base_template = vec![
        *UNIVERSAL_STEP_1,
        [MEM.bout, PC.addr, Reg0Write].into(), // write immediate to reg0
        [PC.cnt, Reset].into(),                // pc cnt
    ];

    // writes to A first to be able to then write to reg0
    let base_template_pc = vec![
        *UNIVERSAL_STEP_1,
        [MEM.bout, PC.addr, A.write].into(), // write immediate to A
        [A.bout, Reg0Write, Reset].into(),   // write A contents to PC reg
    ];

    for reg in Register::write_iterator() {
        let mut current = {
            if matches!(reg, Register::PcLo | Register::PcHi) {
                &base_template_pc
            } else {
                &base_template
            }
        }
        .clone();

        reg.fill_reg0(&mut current);
        assert!(all_regs_filled(&current));

        result.push(InstructionImpl::Simple(SimpleInstruction {
            istr: current,
            name: format!("mv {}, imm8", reg.name()),
        }));
    }

    result
}

// reg = [mar]
pub fn lw_template_addr_reg_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    // NOTE: must work if pc cnt is removed (fine here since no pc mem access)
    let base_template = vec![
        [MEM.bout, Addr0Out, Reg0Write, PC.cnt, Reset].into(), // read from addr mar into register, pc cnt
    ];

    // write to A to be able to write to addr reg
    let base_template_conflict = vec![
        [MEM.bout, Addr0Out, A.write, PC.cnt].into(), // read from addr mar into register, pc cnt
        [A.bout, Reg0Write, Reset].into(),            // read from addr mar into register, pc cnt
    ];

    for addr_reg in AddressRegisterEnum::iterator() {
        for reg in Register::write_iterator() {
            let conflict = match addr_reg.to_reg_impl() {
                AddressRegisterImpl::Mar(_) => matches!(reg, Register::MarLo | Register::MarHi),
                AddressRegisterImpl::Sp(_) => matches!(reg, Register::SpLo | Register::SpHi),
            };

            let mut current = if conflict {
                &base_template_conflict
            } else {
                &base_template
            }
            .clone();

            // if writing to PC then don't increment
            if matches!(reg, Register::PcLo | Register::PcHi) {
                replace_action(&mut current, PC.cnt, Nop);
            }

            addr_reg.fill_addr_reg0(&mut current);
            reg.fill_reg0(&mut current);

            assert!(all_regs_filled(&current));
            assert!(addr_reg_filled(&current));

            result.push(InstructionImpl::Simple(SimpleInstruction {
                istr: current,
                name: format!("lw {}, mem[{}]", reg.name(), addr_reg.name()),
            }));
        }
    }

    result
}

// reg = [imm16]
pub fn lw_template_imm16_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    let base_template = vec![
        IMM_TO_ADDR_REG[1],
        IMM_TO_ADDR_REG[2],
        IMM_TO_ADDR_REG[3],
        IMM_TO_ADDR_REG[4],
        [MEM.bout, Addr0Out, Reg0Write, PC.cnt, Reset].into(), // read from addr mar into register, pc cnt
    ];

    // does not include the pc.cnt since PC is getting written to
    let base_template_pc = vec![
        IMM_TO_ADDR_REG[1],
        IMM_TO_ADDR_REG[2],
        IMM_TO_ADDR_REG[3],
        IMM_TO_ADDR_REG[4],
        [MEM.bout, Addr0Out, Reg0Write, Reset].into(), // read from addr mar into register, pc cnt
    ];

    // write to A to be able to write to addr reg
    let base_template_conflict = vec![
        IMM_TO_ADDR_REG[1],
        IMM_TO_ADDR_REG[2],
        IMM_TO_ADDR_REG[3],
        IMM_TO_ADDR_REG[4],
        [MEM.bout, Addr0Out, A.write, PC.cnt].into(), // read from addr mar into register, pc cnt
        [A.bout, Reg0Write, Reset].into(),            // read from addr mar into register, pc cnt
    ];

    for addr_reg in AddressRegisterEnum::iterator() {
        for reg in Register::write_iterator() {
            let conflict = match addr_reg.to_reg_impl() {
                AddressRegisterImpl::Mar(_) => matches!(reg, Register::MarLo | Register::MarHi),
                AddressRegisterImpl::Sp(_) => matches!(reg, Register::SpLo | Register::SpHi),
            };

            let mut current = {
                if matches!(reg, Register::PcLo | Register::PcHi) {
                    &base_template_pc
                } else if conflict {
                    &base_template_conflict
                } else {
                    &base_template
                }
            }
            .clone();

            addr_reg.fill_addr_reg0(&mut current);
            reg.fill_reg0(&mut current);

            assert!(all_regs_filled(&current));
            assert!(addr_reg_filled(&current));

            result.push(InstructionImpl::Simple(SimpleInstruction {
                istr: current,
                name: format!("lw {}, mem[imm16], {}", reg.name(), addr_reg.name()),
            }));
        }
    }

    result
}

pub fn sw_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    // mem[mar] = reg
    let sw = vec![[MEM.write, Addr0Out, Reg0Bout, PC.cnt, Reset].into()];

    // writes to a first to avoid addr bus conflicts
    let sw_addr_reg = vec![
        [A.write, Reg0Bout].into(),
        [MEM.write, Addr0Out, A.bout, PC.cnt, Reset].into(),
    ];

    // mem[imm16] = reg
    let sw_imm = vec![
        IMM_TO_ADDR_REG[1],
        IMM_TO_ADDR_REG[2],
        IMM_TO_ADDR_REG[3],
        IMM_TO_ADDR_REG[4],
        [MEM.write, Addr0Out, Reg0Bout, PC.cnt, Reset].into(), // read from addr mar into register, pc cnt
    ];

    // writes to a first to avoid bus conflicts
    let sw_imm_addr_reg = vec![
        IMM_TO_ADDR_REG[1],
        IMM_TO_ADDR_REG[2],
        IMM_TO_ADDR_REG[3],
        IMM_TO_ADDR_REG[4],
        [A.write, Reg0Bout].into(),
        [MEM.write, Addr0Out, A.bout, PC.cnt, Reset].into(),
    ];

    for imm in [false, true] {
        for addr_reg in AddressRegisterEnum::iterator() {
            for reg in Register::read_iterator()
                // excluded PC (useless op, can be replaced with imm8)
                .filter(|e| !matches!(e, Register::PcLo | Register::PcHi))
            {
                let addr_bus_conflict = matches!(
                    reg,
                    Register::MarLo | Register::MarHi | Register::SpLo | Register::SpHi
                );

                let conflict_template = if imm { &sw_imm_addr_reg } else { &sw_addr_reg };
                let base_template = if imm { &sw_imm } else { &sw };

                let mut current = {
                    if addr_bus_conflict {
                        conflict_template
                    } else {
                        base_template
                    }
                }
                .clone();

                addr_reg.fill_addr_reg0(&mut current);
                reg.fill_reg0(&mut current);

                assert!(all_regs_filled(&current));
                assert!(addr_reg_filled(&current));

                if imm {
                    result.push(InstructionImpl::Simple(SimpleInstruction {
                        istr: current,
                        name: format!("sw {}, mem[imm16], {}", reg.name(), addr_reg.name()),
                    }));
                } else {
                    result.push(InstructionImpl::Simple(SimpleInstruction {
                        istr: current,
                        name: format!("sw {}, mem[{}]", reg.name(), addr_reg.name()),
                    }));
                }
            }
        }
    }
    result
}

// [sp--] = reg
pub fn push_reg_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    let base_template = vec![
        [PC.cnt, SP.dec].into(), // decrement before pushing value
        [MEM.write, SP.addr, Reg0Bout, Reset].into(), // read from reg into mem at sp addr, pc cnt
    ];

    // have to write to A first to avoid addr bus contention
    let base_template_addr_reg = vec![
        [A.write, Reg0Bout, PC.cnt, SP.dec].into(), // decrement before pushing value
        [MEM.write, SP.addr, A.bout, Reset].into(), // read from reg into mem at sp addr, pc cnt
    ];

    for reg in Register::read_iterator().filter(|e| !matches!(e, Register::PcLo | Register::PcHi)) {
        let addr_bus_conflict = matches!(
            reg,
            Register::MarLo | Register::MarHi | Register::SpLo | Register::SpHi
        );

        let mut current = {
            if addr_bus_conflict {
                &base_template_addr_reg
            } else {
                &base_template
            }
        }
        .clone();

        reg.fill_reg0(&mut current);

        assert!(all_regs_filled(&current));

        result.push(InstructionImpl::Simple(SimpleInstruction {
            istr: current,
            name: format!("push {}", reg.name()),
        }));
    }
    result
}

// [sp--] = imm8, overrides a reg
pub fn push_imm8_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    let current = vec![
        [PC.cnt, SP.dec].into(),             // decrement before pushing value
        [MEM.bout, PC.addr, A.write].into(), // write into ir2
        [MEM.write, SP.addr, A.bout, PC.cnt, Reset].into(), // read from ir2 into [sp], pc cnt
    ];

    result.push(InstructionImpl::Simple(SimpleInstruction {
        istr: current,
        name: "push imm8".to_string(),
    }));
    result
}

// Reg0 = [sp++]
pub fn pop_reg_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    let base_template = vec![
        [MEM.bout, SP.addr, Reg0Write, PC.cnt].into(), // write from [sp] into Reg0, pc cnt
        [SP.inc, Reset].into(),                        // cnt after popping value
    ];

    // have to write to A first to avoid addr bus contention
    let base_template_addr_reg = vec![
        [MEM.bout, SP.addr, A.write, PC.cnt].into(), // write from [sp] into A, pc cnt
        [A.bout, Reg0Write, SP.inc, Reset].into(),   // cnt after popping value
    ];

    // excluded PC and SP (pop addr makes more sense in that context)
    for reg in Register::write_iterator().filter(|e| {
        !matches!(
            e,
            Register::PcLo | Register::PcHi | Register::SpLo | Register::SpHi
        )
    }) {
        let addr_bus_conflict = matches!(reg, Register::MarLo | Register::MarHi);

        let mut current = {
            if addr_bus_conflict {
                &base_template_addr_reg
            } else {
                &base_template
            }
        }
        .clone();

        reg.fill_reg0(&mut current);

        assert!(all_regs_filled(&current));

        result.push(InstructionImpl::Simple(SimpleInstruction {
            istr: current,
            name: format!("pop {}", reg.name()),
        }));
    }
    result
}

// AddrReg = mem[SP], SP += 2
pub fn pop_addr_reg_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    let sp_to_mar = vec![
        [MEM.bout, SP.addr, MAR.hi.write, PC.cnt].into(),
        [SP.inc].into(),
        [MEM.bout, SP.addr, MAR.lo.write].into(),
        [SP.inc, Reset].into(),
    ];

    result.push(InstructionImpl::Simple(SimpleInstruction {
        istr: sp_to_mar,
        name: "pop MAR".to_string(),
    }));

    let sp_to_sp_through_mar = vec![
        [MEM.bout, SP.addr, MAR.hi.write, PC.cnt].into(),
        [SP.inc].into(),
        [MEM.bout, SP.addr, MAR.lo.write].into(),
        [MAR.hi.bout, SP.hi.write].into(),
        [MAR.lo.bout, SP.lo.write].into(),
        [SP.inc, Reset].into(),
    ];

    result.push(InstructionImpl::Simple(SimpleInstruction {
        istr: sp_to_sp_through_mar,
        name: "pop SP, MAR".to_string(),
    }));

    let sp_to_sp_through_ab = vec![
        [MEM.bout, SP.addr, B.write, PC.cnt].into(),
        [SP.inc].into(),
        [MEM.bout, SP.addr, A.write].into(),
        [B.bout, SP.hi.write].into(),
        [A.bout, SP.lo.write].into(),
        [SP.inc, Reset].into(),
    ];

    result.push(InstructionImpl::Simple(SimpleInstruction {
        istr: sp_to_sp_through_ab,
        name: "pop SP, AB".to_string(),
    }));

    result
}

// mar/sp = imm16
pub fn lda_imm16_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    let base_template = vec![
        IMM_TO_ADDR_REG[1],
        IMM_TO_ADDR_REG[2],
        IMM_TO_ADDR_REG[3],
        IMM_TO_ADDR_REG[4],
        [PC.cnt, Reset].into(), // pc cnt
    ];

    for addr_reg in AddressRegisterEnum::iterator() {
        let mut current = base_template.clone();

        addr_reg.fill_addr_reg0(&mut current);

        assert!(addr_reg_filled(&current));

        result.push(InstructionImpl::Simple(SimpleInstruction {
            istr: current,
            name: format!("lda {}, imm16", addr_reg.name()),
        }));
    }
    result
}

// mar/sp = imm16
pub fn mv_addr_reg_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    let base_template = vec![
        [MEM.bout, Addr1Out, Addr0HiWrite, PC.cnt].into(), // first byte has msb
        [MEM.bout, Addr1Out, Addr0LoWrite].into(),         // second byte has lsb
        [PC.cnt, Reset].into(),                            // pc cnt
    ];

    for addr_reg0 in AddressRegisterEnum::iterator() {
        for addr_reg1 in AddressRegisterEnum::iterator().filter(|&e| e != addr_reg0) {
            let mut current = base_template.clone();

            addr_reg0.fill_addr_reg0(&mut current);
            addr_reg1.fill_addr_reg1(&mut current);

            assert!(addr_reg_filled(&current));

            result.push(InstructionImpl::Simple(SimpleInstruction {
                istr: current,
                name: format!("mva {}, {}", addr_reg0.name(), addr_reg1.name()),
            }));
        }
    }
    result
}

pub fn misc_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    // sp dec
    let sp_dec = vec![
        [PC.cnt, SP.dec, Reset].into(), // decrement sp
    ];

    result.push(InstructionImpl::Simple(SimpleInstruction {
        istr: sp_dec,
        name: "dec SP".to_string(),
    }));

    // sp cnt
    let sp_inc = vec![*UNIVERSAL_STEP_0, [PC.cnt, SP.inc, Reset].into()];

    result.push(InstructionImpl::Simple(SimpleInstruction {
        istr: sp_inc,
        name: "inc SP".to_string(),
    }));

    // mar cnt
    let mar_inc = vec![
        [PC.cnt, MAR.cnt, Reset].into(), // cntrement mar
    ];

    result.push(InstructionImpl::Simple(SimpleInstruction {
        istr: mar_inc,
        name: "inc MAR".to_string(),
    }));

    let halt = vec![[Halt].into()];

    result.push(InstructionImpl::Simple(SimpleInstruction {
        istr: halt,
        name: "halt".to_string(),
    }));

    // 2 instruction nop
    let nop = vec![[PC.cnt, Reset].into()];

    result.push(InstructionImpl::Simple(SimpleInstruction {
        istr: nop,
        name: "nop".to_string(),
    }));

    result
}

pub fn vram_read_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    let vram_read_template_odd = vec![
        [VramRead, Addr0Out, Reg0Write].into(),
        [Nop].into(),
        [PC.cnt, Reset].into(),
    ];

    let vram_read_template_even = vec![
        [Nop].into(),
        [VramRead, Addr0Out, Reg0Write].into(),
        [PC.cnt, Reset].into(),
    ];

    let vram_read_template_odd_conflict = vec![
        [VramRead, Addr0Out, A.write].into(),
        [A.bout, Reg0Write].into(),
        [PC.cnt, Reset].into(),
    ];

    let vram_read_template_even_conflict = vec![
        [Nop].into(),
        [VramRead, Addr0Out, A.write].into(),
        [A.bout, Reg0Write].into(),
        [PC.cnt, Reset].into(),
    ];

    for addr_reg in AddressRegisterEnum::iterator() {
        for reg in Register::write_iterator()
            // exclude PC
            .filter(|e| !matches!(e, Register::PcLo | Register::PcHi))
        {
            let conflict = match addr_reg.to_reg_impl() {
                AddressRegisterImpl::Mar(_) => matches!(reg, Register::MarLo | Register::MarHi),
                AddressRegisterImpl::Sp(_) => matches!(reg, Register::SpLo | Register::SpHi),
            };

            let mut odd_current = {
                if conflict {
                    &vram_read_template_odd_conflict
                } else {
                    &vram_read_template_odd
                }
            }
            .clone();

            let mut even_current = {
                if conflict {
                    &vram_read_template_even_conflict
                } else {
                    &vram_read_template_even
                }
            }
            .clone();

            reg.fill_reg0(&mut odd_current);
            reg.fill_reg0(&mut even_current);

            addr_reg.fill_addr_reg0(&mut odd_current);
            addr_reg.fill_addr_reg0(&mut even_current);

            assert!(all_regs_filled(&odd_current));
            assert!(all_regs_filled(&even_current));

            let name = format!("lw {}, vram[{}]", reg.name(), addr_reg.name());

            result.push(InstructionImpl::Vram(VramInstruction {
                active_odd: SimpleInstruction {
                    istr: odd_current,
                    name: format!("{}; odd", name.clone()),
                },
                active_even: SimpleInstruction {
                    istr: even_current,
                    name: format!("{}; even", name.clone()),
                },
                name,
            }));
        }
    }
    result
}

pub fn vram_write_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    let vram_write_template = vec![
        [VramWrite, Addr0Out, Reg0Bout].into(),
        [VramWrite, Addr0Out, Reg0Bout].into(),
        [PC.cnt, Reset].into(),
    ];

    let vram_write_template_conflict = vec![
        [Reg0Bout, A.write].into(),
        [VramWrite, Addr0Out, A.bout].into(),
        [VramWrite, Addr0Out, A.bout].into(),
        [PC.cnt, Reset].into(),
    ];

    for addr_reg in AddressRegisterEnum::iterator() {
        for reg in Register::read_iterator()
            // exclude PC
            .filter(|e| !matches!(e, Register::PcLo | Register::PcHi))
        {
            let addr_bus_conflict = matches!(
                reg,
                Register::MarLo | Register::MarHi | Register::SpLo | Register::SpHi
            );

            let mut current = {
                if addr_bus_conflict {
                    &vram_write_template_conflict
                } else {
                    &vram_write_template
                }
            }
            .clone();

            reg.fill_reg0(&mut current);
            addr_reg.fill_addr_reg0(&mut current);

            assert!(all_regs_filled(&current));

            result.push(InstructionImpl::Simple(SimpleInstruction {
                istr: current,
                name: format!("sw {}, vram[{}]", reg.name(), addr_reg.name()),
            }));
        }
    }
    result
}

pub fn not_instructions() -> Vec<(InstructionImpl, MathIstrTypes)> {
    let mut result = Vec::new();

    // Reg0 = ~Reg0
    let not = vec![
        [Reg0Bout, A.write, PC.cnt].into(), // load Reg0 into a, pc cnt
        [FAluBout, FlagWriteAlu, Reg0Write].into(), // do math op, save to Reg0, writes to flag reg
        [Reset].into(), // must reset on separate instruction since it shares bits with flag write
    ];

    // no need to write to a since we are opping directly on A
    let not_a = vec![
        [FAluBout, FlagWriteAlu, Reg0Write, PC.cnt].into(), // do math op, save to Reg0, writes to flag reg
        [Reset].into(), // must reset on separate instruction since it shares bits with flag write
    ];

    for reg in Register::write_iterator().filter(|e| !matches!(e, Register::PcLo | Register::PcHi))
    {
        let mut current = if matches!(reg, Register::A) {
            &not_a
        } else {
            &not
        }
        .clone();

        reg.fill_reg0(&mut current);

        assert!(all_regs_filled(&current));

        result.push((
            InstructionImpl::Simple(SimpleInstruction {
                istr: current,
                name: format!("not {}", reg.name()),
            }),
            MathIstrTypes::Not,
        ));
    }
    result
}

pub fn not_reg_instructions() -> Vec<(InstructionImpl, MathIstrTypes)> {
    let mut result = Vec::new();

    // Reg0 = ~Reg1
    let base_template = vec![
        [Reg1Bout, A.write, PC.cnt].into(),         // load Reg1 into a
        [FAluBout, FlagWriteAlu, Reg0Write].into(), // do math op, save to Reg1, writes to flag reg
        [Reset].into(), // must reset on separate instruction since it shares bits with flag write
    ];

    // a = reg1, so no need to load reg1 into a
    let base_template_reg1_a = vec![
        [FAluBout, FlagWriteAlu, Reg0Write, PC.cnt].into(), // do math op, save to Reg1, writes to flag reg
        [Reset].into(), // must reset on separate instruction since it shares bits with flag write
    ];

    // PC excluded
    for reg0 in Register::write_iterator().filter(|e| !matches!(e, Register::PcLo | Register::PcHi))
    {
        // excludes PC, avoids duplicates
        let rhs = Register::read_iterator()
            .filter(|e| !matches!(e, Register::PcLo | Register::PcHi))
            .filter(|&e| e != reg0);
        for reg1 in rhs {
            // avoid duplicates
            let mut current = {
                if matches!(reg1, Register::A) {
                    &base_template_reg1_a
                } else {
                    &base_template
                }
            }
            .clone();

            reg0.fill_reg0(&mut current);
            reg1.fill_reg1(&mut current);

            assert!(all_regs_filled(&current));

            result.push((
                InstructionImpl::Simple(SimpleInstruction {
                    istr: current,
                    name: format!("not {}, {}", reg0.name(), reg1.name()),
                }),
                MathIstrTypes::Not,
            ));
        }
    }
    result
}

pub fn math_imm_instructions() -> Vec<(InstructionImpl, MathIstrTypes)> {
    let mut result = Vec::new();

    // Reg0 = Reg0 op imm
    let math_imm = vec![
        [Reg0Bout, A.write, PC.cnt].into(),  // load Reg0 into A, pc cnt
        [MEM.bout, PC.addr, B.write].into(), // load imm into B
        [
            FAluBout,
            FlagWriteAlu,
            Reg0Write,
            OutputFlagsSelector,
            PC.cnt,
        ]
        .into(), // save f to Reg0, writes to flag reg
        [Reset].into(), // must reset on separate instruction since it shares bits with flag write
    ];

    // Reg0 = imm op Reg0
    let math_imm_reverse = vec![
        [Reg0Bout, B.write, PC.cnt].into(),  // load Reg0 into B, pc cnt
        [MEM.bout, PC.addr, A.write].into(), // load imm into A
        [
            FAluBout,
            FlagWriteAlu,
            Reg0Write,
            OutputFlagsSelector,
            PC.cnt,
        ]
        .into(), // save f to Reg0, writes to flag reg
        [Reset].into(), // must reset on separate instruction since it shares bits with flag write
    ];

    // edge cases where register gets loaded to itself
    // all other cases have good register overwrite order
    let math_imm_a = vec![
        [PC.cnt].into(),                     // load A into A (nop), pc cnt
        [MEM.bout, PC.addr, B.write].into(), // load imm into B
        [FAluBout, FlagWriteAlu, A.write, OutputFlagsSelector, PC.cnt].into(), // save f to Reg0, writes to flag reg
        [Reset].into(), // must reset on separate instruction since it shares bits with flag write
    ];

    let math_imm_b_reverse = vec![
        [PC.cnt].into(),                     // load B into B (nop), pc cnt
        [MEM.bout, PC.addr, A.write].into(), // load imm into A
        [FAluBout, FlagWriteAlu, B.write, OutputFlagsSelector, PC.cnt].into(), // save f to B, writes to flag reg
        [Reset].into(), // must reset on separate instruction since it shares bits with flag write
    ];

    for math_type in MathIstrTypes::iterator().filter(|e| !matches!(e, MathIstrTypes::Not)) {
        for reversed in [false, true] {
            for reg in
                Register::write_iterator().filter(|e| !matches!(e, Register::PcLo | Register::PcHi))
            {
                let mut current = {
                    if matches!(reg, Register::A) && !reversed {
                        &math_imm_a
                    } else if matches!(reg, Register::B) && reversed {
                        &math_imm_b_reverse
                    } else {
                        if reversed {
                            &math_imm_reverse
                        } else {
                            &math_imm
                        }
                    }
                }
                .clone();

                if matches!(math_type, MathIstrTypes::Cmp) {
                    // on cmp, replace writing actions to nothing
                    replace_action(&mut current, Reg0Write, Nop);
                    replace_action(&mut current, FAluBout, Nop);
                }

                reg.fill_reg0(&mut current);

                fill_flag_select(&mut current, math_type.to_action());

                assert!(all_regs_filled(&current));
                assert!(flag_select_filled(&current));

                let name = if reversed {
                    format!("{} imm8, {}", math_type, reg.name())
                } else {
                    format!("{} {}, imm8", math_type, reg.name())
                };

                result.push((
                    InstructionImpl::Simple(SimpleInstruction {
                        istr: current,
                        name,
                    }),
                    math_type.clone(),
                ));
            }
        }
    }
    result
}

pub fn math_reg_instructions() -> Vec<(InstructionImpl, MathIstrTypes)> {
    let mut result = Vec::new();

    // possible edge cases:
    // reg0 = A, reg1 = ? -> remove a load step
    // reg0 = ?, reg1 = B -> remove b load step
    // reg0 = A, reg1 = B -> remove both load steps
    // reg0 = B, reg1 = A -> impossible

    // NOTE: all istrs here must be able to work if no pc.cnt happens

    // Reg0 = Reg0 op Reg1
    let math_reg = vec![
        [Reg0Bout, A.write, PC.cnt].into(), // load Reg0 into a
        [Reg1Bout, B.write].into(),         // load Reg1 into b
        [FAluBout, FlagWriteAlu, Reg0Write, OutputFlagsSelector].into(), // do math op, save to Reg0, writes to flag reg
        [Reset].into(),
    ];

    // a and b already loaded
    let math_reg_a_b = vec![
        [FAluBout, FlagWriteAlu, A.write, OutputFlagsSelector].into(), // do math op, save to Reg0, writes to flag reg
        [Reset, PC.cnt].into(),
    ];

    let math_reg_a = vec![
        [Reg1Bout, B.write, PC.cnt].into(), // load Reg1 into b
        [FAluBout, FlagWriteAlu, Reg0Write, OutputFlagsSelector].into(), // do math op, save to Reg0, writes to flag reg
        [Reset].into(),
    ];

    let math_reg_b = vec![
        [Reg0Bout, A.write, PC.cnt].into(), // load Reg0 into a
        [FAluBout, FlagWriteAlu, Reg0Write, OutputFlagsSelector].into(), // do math op, save to Reg0, writes to flag reg
        [Reset].into(),
    ];

    for math_type in MathIstrTypes::iterator().filter(|e| !matches!(e, MathIstrTypes::Not)) {
        for reg0 in Register::write_iterator() {
            // excludes PC
            for reg1 in
                Register::read_iterator().filter(|e| !matches!(e, Register::PcLo | Register::PcHi))
            {
                // impossible case
                if matches!(reg1, Register::A) && matches!(reg0, Register::B) {
                    continue;
                }

                let mut current = {
                    let reg0_a = matches!(reg0, Register::A);
                    let reg1_b = matches!(reg1, Register::B);

                    if reg0_a && reg1_b {
                        &math_reg_a_b
                    } else if reg0_a {
                        &math_reg_a
                    } else if reg1_b {
                        &math_reg_b
                    } else {
                        &math_reg
                    }
                }
                .clone();

                if matches!(math_type, MathIstrTypes::Cmp) {
                    // on cmp, replace writing actions to nothing
                    replace_action(&mut current, Reg0Write, Nop);
                    replace_action(&mut current, FAluBout, Nop);
                } else {
                    if matches!(reg0, Register::PcLo | Register::PcHi) {
                        // here we are writing to PC (since not cmp), so avoid pc.cnt
                        replace_action(&mut current, PC.cnt, Nop);
                    }
                }

                reg0.fill_reg0(&mut current);
                reg1.fill_reg1(&mut current);
                fill_flag_select(&mut current, math_type.to_action());

                assert!(all_regs_filled(&current));
                assert!(flag_select_filled(&current));

                result.push((
                    InstructionImpl::Simple(SimpleInstruction {
                        istr: current,
                        name: format!("{} {}, {}", math_type, reg0.name(), reg1.name()),
                    }),
                    math_type.clone(),
                ));
            }
        }
    }
    result
}

pub fn jnz_reg_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    // jnz reg -> pc = mar if reg != 0 else nop
    let jnz_template_reg = vec![
        [A.write, Reg0Bout, PC.cnt].into(), // note: pc cnt happens in case jump doesn't happens
        [FlagWriteAlu].into(),              // write zero result to flag register
        [Addr0HiBout, PC.hi.write, FlagNotZero].into(),
        [Addr0LoBout, PC.lo.write, FlagNotZero, Reset].into(),
    ];

    // can save instruction if a is already loaded
    let jnz_template_reg_a = vec![
        [FlagWriteAlu, PC.cnt].into(), // update flag register
        [Addr0HiBout, PC.hi.write, FlagNotZero].into(),
        [Addr0LoBout, PC.lo.write, FlagNotZero, Reset].into(),
    ];

    for addr_reg in AddressRegisterEnum::iterator() {
        for reg in
            Register::read_iterator().filter(|e| !matches!(e, Register::PcLo | Register::PcHi))
        {
            let mut current = {
                if matches!(reg, Register::A) {
                    &jnz_template_reg_a
                } else {
                    &jnz_template_reg
                }
            }
            .clone();

            addr_reg.fill_addr_reg0(&mut current);
            reg.fill_reg0(&mut current);

            assert!(all_regs_filled(&current));
            assert!(addr_reg_filled(&current));

            result.push(InstructionImpl::Simple(SimpleInstruction {
                istr: current,
                name: format!("jnz {}, {}", reg.name(), addr_reg.name()),
            }));
        }
    }
    result
}

pub fn jmp_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    // jump if equal flag is carry flag is true
    let jmp_imm16_template = vec![
        IMM_TO_ADDR_REG[1],
        IMM_TO_ADDR_REG[2],
        IMM_TO_ADDR_REG[3],
        IMM_TO_ADDR_REG[4],
        [PC.cnt].into(), // note: pc cnt happens in case jump doesn't happens
        [Addr0HiBout, PC.hi.write, OutputFlagsSelector].into(), // load from mar into pc if flag
        [Addr0LoBout, PC.lo.write, OutputFlagsSelector, Reset].into(), // load from mar into pc if flag
    ];

    // jump if equal flag is true
    let jmp_addr_reg_template = vec![
        [PC.cnt].into(), // note: pc cnt in case jump doesn't happen
        [Addr0HiBout, PC.hi.write, OutputFlagsSelector].into(),
        [Addr0LoBout, PC.lo.write, OutputFlagsSelector, Reset].into(),
    ];

    for imm in [false, true] {
        for addr_reg in AddressRegisterEnum::iterator() {
            for flag in OutputFlags::iterator() {
                let mut current = if imm {
                    &jmp_imm16_template
                } else {
                    &jmp_addr_reg_template
                }
                .clone();

                addr_reg.fill_addr_reg0(&mut current);
                fill_flag_select(&mut current, flag.get_action());

                assert!(all_regs_filled(&current));
                assert!(addr_reg_filled(&current));
                assert!(flag_select_filled(&current));

                let name = if imm {
                    format!("{} imm16, {}", flag.get_jump_name(), addr_reg.name())
                } else {
                    format!("{} {}", flag.get_jump_name(), addr_reg.name())
                };

                result.push(InstructionImpl::Simple(SimpleInstruction {
                    istr: current,
                    name,
                }));
            }
        }
    }
    result
}

// TODO: add instruction to update flag reg on op
pub fn shift_instructions() -> Vec<InstructionImpl> {
    let mut result = Vec::new();

    let shift_left = vec![[X.shift_left, PC.cnt, Reset].into()];
    let shift_right = vec![[X.shift_right, PC.cnt, Reset].into()];

    for dir_left in [false, true] {
        for reg in [Register::X, Register::Y] {
            let mut current = if dir_left { &shift_left } else { &shift_right }.clone();

            if matches!(reg, Register::Y) {
                replace_action(&mut current, X.shift_left, Y.shift_left);
                replace_action(&mut current, X.shift_right, Y.shift_right);
            }

            let name = if dir_left {
                format!("shl {}", reg.name())
            } else {
                format!("shr {}", reg.name())
            };

            result.push(InstructionImpl::Simple(SimpleInstruction {
                istr: current,
                name,
            }));
        }
    }

    result
}
