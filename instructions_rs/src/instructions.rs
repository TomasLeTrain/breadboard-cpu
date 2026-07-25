use crate::{
    action::Action::{self, *},
    step_template::StepTemplate,
    step_template_to_output,
};

use std::{
    ops::{Deref, DerefMut},
    str::Matches,
    sync::LazyLock,
};

mod register_defs;
use register_defs::*;

mod defs;
use defs::*;

pub use defs::{InstructionType, NamedInstruction};

// move register to register (reg0 = reg1)
fn move_word_reg_instructions(destination: &mut Vec<NamedInstruction>) {
    let base_template = vec![
        *UNIVERSAL_STEP_0,
        *UNIVERSAL_STEP_1,
        *LOAD_IR2,                                   // load ir 2
        [Reg1Bout, Reg0Write, PC.cnt, Reset].into(), // read from reg1 to reg0, pc cnt
    ];

    // removes the pc cnt in case pc is getting written to
    let base_template_pc = vec![
        *UNIVERSAL_STEP_0,
        *UNIVERSAL_STEP_1,
        *LOAD_IR2,                           // load ir 2
        [Reg1Bout, Reg0Write, Reset].into(), // read from reg1 to reg0
    ];

    // excludes PC
    for reg0 in WRITE_REGISTERS.iter() {
        // excludes PC, avoids duplicates
        let rhs = READ_REGISTERS
            .iter()
            .filter(|e| !matches!(e.name, "PC.lo" | "PC.hi"))
            .filter(|e| e.name != reg0.name);
        for reg1 in rhs {
            // avoid duplicates
            let mut current = {
                if reg0.name == "PC.lo" || reg0.name == "PC.hi" {
                    &base_template_pc
                } else {
                    &base_template
                }
            }
            .clone();

            fill_reg0(&mut current, reg0);
            fill_reg1(&mut current, reg1);

            assert!(all_regs_filled(&current));

            destination.push(NamedInstruction {
                istr: current,
                name: format!("mv {}, {}", reg0.name, reg1.name),
                istr_type: InstructionType::Extended,
            });
        }
    }
}

// move imm8 to register (reg0 = imm8)
fn move_word_imm_instructions(destination: &mut Vec<NamedInstruction>) {
    let base_template = vec![
        *UNIVERSAL_STEP_0,
        *UNIVERSAL_STEP_1,
        [MEM.bout, PC.addr, Reg0Write].into(), // write immediate to reg0
        [PC.cnt, Reset].into(),                // pc cnt
    ];

    // writes to A first to be able to then write to reg0
    let base_template_pc = vec![
        *UNIVERSAL_STEP_0,
        *UNIVERSAL_STEP_1,
        [MEM.bout, PC.addr, A.write].into(), // write immediate to A
        [A.bout, Reg0Write, Reset].into(),   // write A contents to PC reg
    ];

    for reg in WRITE_REGISTERS.iter() {
        let mut current = {
            if reg.name == "PC.lo" || reg.name == "PC.hi" {
                &base_template_pc
            } else {
                &base_template
            }
        }
        .clone();

        fill_reg0(&mut current, reg);
        assert!(all_regs_filled(&current));

        destination.push(NamedInstruction {
            istr: current,
            name: format!("mv {}, imm8", reg.name),
            istr_type: InstructionType::Single,
        });
    }
}

// reg = [mar]
fn lw_template_mar_instructions(destination: &mut Vec<NamedInstruction>) {
    let base_template = vec![
        *UNIVERSAL_STEP_0,
        [MEM.bout, Addr0Out, Reg0Write, PC.cnt, Reset].into(), // read from addr mar into register, pc cnt
    ];

    // does not include the pc.cnt since PC is getting written to
    let base_template_pc = vec![
        *UNIVERSAL_STEP_0,
        [MEM.bout, Addr0Out, Reg0Write, Reset].into(), // read from addr mar into register, pc cnt
    ];

    // write to A to be able to write to addr reg
    let base_template_conflict = vec![
        *UNIVERSAL_STEP_0,
        [MEM.bout, Addr0Out, A.write, PC.cnt].into(), // read from addr mar into register, pc cnt
        [A.bout, Reg0Write, Reset].into(),            // read from addr mar into register, pc cnt
    ];

    for addr_reg in ADDR_REGISTERS.iter() {
        for reg in WRITE_REGISTERS.iter() {
            let conflict = match addr_reg.reg {
                AddressRegisters::Mar(_) => matches!(reg.name, "MAR.lo" | "MAR.hi"),
                AddressRegisters::Sp(_) => matches!(reg.name, "SP.lo" | "SP.hi"),
            };

            let mut current = {
                if reg.name == "PC.lo" || reg.name == "PC.hi" {
                    &base_template_pc
                } else if conflict {
                    &base_template_conflict
                } else {
                    &base_template
                }
            }
            .clone();

            fill_addr_reg0(&mut current, addr_reg);
            fill_reg0(&mut current, reg);

            assert!(all_regs_filled(&current));
            assert!(addr_reg_filled(&current));

            destination.push(NamedInstruction {
                istr: current,
                name: format!("lw {}, mem[{}]", reg.name, addr_reg.name),
                istr_type: InstructionType::Single,
            });
        }
    }
}

// reg = [imm16]
fn lw_template_imm16_instructions(destination: &mut Vec<NamedInstruction>) {
    let base_template = vec![
        IMM_TO_ADDR_REG[0],
        IMM_TO_ADDR_REG[1],
        IMM_TO_ADDR_REG[2],
        IMM_TO_ADDR_REG[3],
        IMM_TO_ADDR_REG[4],
        [MEM.bout, Addr0Out, Reg0Write, PC.cnt, Reset].into(), // read from addr mar into register, pc cnt
    ];

    // does not include the pc.cnt since PC is getting written to
    let base_template_pc = vec![
        IMM_TO_ADDR_REG[0],
        IMM_TO_ADDR_REG[1],
        IMM_TO_ADDR_REG[2],
        IMM_TO_ADDR_REG[3],
        IMM_TO_ADDR_REG[4],
        [MEM.bout, Addr0Out, Reg0Write, Reset].into(), // read from addr mar into register, pc cnt
    ];

    // write to A to be able to write to addr reg
    let base_template_conflict = vec![
        IMM_TO_ADDR_REG[0],
        IMM_TO_ADDR_REG[1],
        IMM_TO_ADDR_REG[2],
        IMM_TO_ADDR_REG[3],
        IMM_TO_ADDR_REG[4],
        [MEM.bout, Addr0Out, A.write, PC.cnt].into(), // read from addr mar into register, pc cnt
        [A.bout, Reg0Write, Reset].into(),            // read from addr mar into register, pc cnt
    ];

    for addr_reg in ADDR_REGISTERS.iter() {
        for reg in WRITE_REGISTERS.iter() {
            let conflict = match addr_reg.reg {
                AddressRegisters::Mar(_) => matches!(reg.name, "MAR.lo" | "MAR.hi"),
                AddressRegisters::Sp(_) => matches!(reg.name, "SP.lo" | "SP.hi"),
            };

            let mut current = {
                if reg.name == "PC.lo" || reg.name == "PC.hi" {
                    &base_template_pc
                } else if conflict {
                    &base_template_conflict
                } else {
                    &base_template
                }
            }
            .clone();

            fill_addr_reg0(&mut current, addr_reg);
            fill_reg0(&mut current, reg);

            assert!(all_regs_filled(&current));
            assert!(addr_reg_filled(&current));

            destination.push(NamedInstruction {
                istr: current,
                name: format!("lw {}, mem[imm16], {}", reg.name, addr_reg.name),
                istr_type: InstructionType::Single,
            });
        }
    }
}

// mem[mar] = reg
fn sw_template_addr_reg_instructions(destination: &mut Vec<NamedInstruction>) {
    let base_template = vec![
        *UNIVERSAL_STEP_0,
        [MEM.write, Addr0Out, Reg0Bout, PC.cnt, Reset].into(),
    ];

    // writes to a first to avoid addr bus conflicts
    let base_template_addr_reg = vec![
        *UNIVERSAL_STEP_0,
        [A.write, Reg0Bout].into(),
        [MEM.write, Addr0Out, A.bout, PC.cnt, Reset].into(),
    ];

    for addr_reg in ADDR_REGISTERS.iter() {
        for reg in READ_REGISTERS
            .iter()
            // excluded PC (useless op, can be replaced with imm8)
            .filter(|e| !matches!(e.name, "PC.lo" | "PC.hi"))
        {
            let addr_bus_conflict = matches!(reg.name, "MAR.lo" | "MAR.hi" | "SP.lo" | "SP.hi");

            let mut current = {
                if addr_bus_conflict {
                    &base_template_addr_reg
                } else {
                    &base_template
                }
            }
            .clone();

            fill_addr_reg0(&mut current, addr_reg);
            fill_reg0(&mut current, reg);

            assert!(all_regs_filled(&current));
            assert!(addr_reg_filled(&current));

            destination.push(NamedInstruction {
                istr: current,
                name: format!("sw {}, mem[{}]", reg.name, addr_reg.name),
                istr_type: InstructionType::Single,
            });
        }
    }
}

// mem[imm16] = reg
fn sw_template_imm16_instructions(destination: &mut Vec<NamedInstruction>) {
    let base_template = vec![
        IMM_TO_ADDR_REG[0],
        IMM_TO_ADDR_REG[1],
        IMM_TO_ADDR_REG[2],
        IMM_TO_ADDR_REG[3],
        IMM_TO_ADDR_REG[4],
        [MEM.write, Addr0Out, Reg0Bout, PC.cnt, Reset].into(), // read from addr mar into register, pc cnt
    ];

    // writes to a first to avoid bus conflicts
    let base_template_addr_reg = vec![
        IMM_TO_ADDR_REG[0],
        IMM_TO_ADDR_REG[1],
        IMM_TO_ADDR_REG[2],
        IMM_TO_ADDR_REG[3],
        IMM_TO_ADDR_REG[4],
        [A.write, Reg0Bout].into(),
        [MEM.write, Addr0Out, A.bout, PC.cnt, Reset].into(),
    ];

    for addr_reg in ADDR_REGISTERS.iter() {
        for reg in READ_REGISTERS
            .iter()
            // excluded PC (useless op, can be replaced with imm8)
            .filter(|e| !matches!(e.name, "PC.lo" | "PC.hi"))
        {
            let addr_bus_conflict = matches!(reg.name, "MAR.lo" | "MAR.hi" | "SP.lo" | "SP.hi");

            let mut current = {
                if addr_bus_conflict {
                    &base_template_addr_reg
                } else {
                    &base_template
                }
            }
            .clone();

            fill_addr_reg0(&mut current, addr_reg);
            fill_reg0(&mut current, reg);

            assert!(all_regs_filled(&current));
            assert!(addr_reg_filled(&current));

            destination.push(NamedInstruction {
                istr: current,
                name: format!("sw {}, mem[imm16], {}", reg.name, addr_reg.name),
                istr_type: InstructionType::Single,
            });
        }
    }
}

// [sp--] = reg
fn push_reg_instructions(destination: &mut Vec<NamedInstruction>) {
    let base_template = vec![
        *UNIVERSAL_STEP_0,
        [PC.cnt, SP.dec].into(), // decrement before pushing value
        [MEM.write, SP.addr, Reg0Bout, Reset].into(), // read from reg into mem at sp addr, pc cnt
    ];

    // have to write to A first to avoid addr bus contention
    let base_template_addr_reg = vec![
        *UNIVERSAL_STEP_0,
        [A.write, Reg0Bout, PC.cnt, SP.dec].into(), // decrement before pushing value
        [MEM.write, SP.addr, A.bout, Reset].into(), // read from reg into mem at sp addr, pc cnt
    ];

    for reg in READ_REGISTERS
        .iter()
        // excluded PC (useless op, can be replaced with imm8)
        .filter(|e| !matches!(e.name, "PC.lo" | "PC.hi"))
    {
        let addr_bus_conflict = matches!(reg.name, "MAR.lo" | "MAR.hi" | "SP.lo" | "SP.hi");

        let mut current = {
            if addr_bus_conflict {
                &base_template_addr_reg
            } else {
                &base_template
            }
        }
        .clone();

        fill_reg0(&mut current, reg);

        assert!(all_regs_filled(&current));

        destination.push(NamedInstruction {
            istr: current,
            name: format!("push {}", reg.name),
            istr_type: InstructionType::Single,
        });
    }
}

// [sp--] = imm8, overrides a reg
fn push_imm8_instructions(destination: &mut Vec<NamedInstruction>) {
    let current = vec![
        *UNIVERSAL_STEP_0,
        [PC.cnt, SP.dec].into(),             // decrement before pushing value
        [MEM.bout, PC.addr, A.write].into(), // write into ir2
        [MEM.write, SP.addr, A.bout, PC.cnt, Reset].into(), // read from ir2 into [sp], pc cnt
    ];

    destination.push(NamedInstruction {
        istr: current,
        name: "push imm8".to_string(),
        istr_type: InstructionType::Single,
    });
}

// Reg0 = [sp++]
fn pop_reg_instructions(destination: &mut Vec<NamedInstruction>) {
    let base_template = vec![
        *UNIVERSAL_STEP_0,
        [MEM.bout, SP.addr, Reg0Write, PC.cnt].into(), // write from [sp] into Reg0, pc cnt
        [SP.inc, Reset].into(),                        // cnt after popping value
    ];

    // have to write to A first to avoid addr bus contention
    let base_template_addr_reg = vec![
        *UNIVERSAL_STEP_0,
        [MEM.bout, SP.addr, A.write, PC.cnt].into(), // write from [sp] into A, pc cnt
        [A.bout, Reg0Write, SP.inc, Reset].into(),   // cnt after popping value
    ];

    for reg in WRITE_REGISTERS
        .iter()
        // excluded PC and SP (pop addr makes more sense in that context)
        .filter(|e| !matches!(e.name, "PC.lo" | "PC.hi" | "SP.lo" | "SP.hi"))
    {
        let addr_bus_conflict = matches!(reg.name, "MAR.lo" | "MAR.hi");

        let mut current = {
            if addr_bus_conflict {
                &base_template_addr_reg
            } else {
                &base_template
            }
        }
        .clone();

        fill_reg0(&mut current, reg);

        assert!(all_regs_filled(&current));

        destination.push(NamedInstruction {
            istr: current,
            name: format!("pop {}", reg.name),
            istr_type: InstructionType::Single,
        });
    }
}

// AddrReg = mem[SP], SP += 2
fn pop_addr_reg_instructions(destination: &mut Vec<NamedInstruction>) {
    let sp_to_mar = vec![
        *UNIVERSAL_STEP_0,
        [MEM.bout, SP.addr, MAR.hi.write, PC.cnt].into(),
        [SP.inc].into(),
        [MEM.bout, SP.addr, MAR.lo.write].into(),
        [SP.inc, Reset].into(),
    ];

    destination.push(NamedInstruction {
        istr: sp_to_mar,
        name: "pop MAR".to_string(),
        istr_type: InstructionType::Single,
    });

    let sp_to_sp_through_mar = vec![
        *UNIVERSAL_STEP_0,
        [MEM.bout, SP.addr, MAR.hi.write, PC.cnt].into(),
        [SP.inc].into(),
        [MEM.bout, SP.addr, MAR.lo.write].into(),
        [MAR.hi.bout, SP.hi.write].into(),
        [MAR.lo.bout, SP.lo.write].into(),
        [SP.inc, Reset].into(),
    ];

    destination.push(NamedInstruction {
        istr: sp_to_sp_through_mar,
        name: "pop SP, MAR".to_string(),
        istr_type: InstructionType::Single,
    });

    let sp_to_sp_through_ab = vec![
        *UNIVERSAL_STEP_0,
        [MEM.bout, SP.addr, B.write, PC.cnt].into(),
        [SP.inc].into(),
        [MEM.bout, SP.addr, A.write].into(),
        [B.bout, SP.hi.write].into(),
        [A.bout, SP.lo.write].into(),
        [SP.inc, Reset].into(),
    ];

    destination.push(NamedInstruction {
        istr: sp_to_sp_through_ab,
        name: "pop SP, AB".to_string(),
        istr_type: InstructionType::Single,
    });
}

// mar/sp = imm16
fn lda_imm16_instructions(destination: &mut Vec<NamedInstruction>) {
    let base_template = vec![
        IMM_TO_ADDR_REG[0],
        IMM_TO_ADDR_REG[1],
        IMM_TO_ADDR_REG[2],
        IMM_TO_ADDR_REG[3],
        IMM_TO_ADDR_REG[4],
        [PC.cnt, Reset].into(), // pc cnt
    ];

    for addr_reg in ADDR_REGISTERS.iter() {
        let mut current = base_template.clone();

        fill_addr_reg0(&mut current, addr_reg);

        assert!(addr_reg_filled(&current));

        destination.push(NamedInstruction {
            istr: current,
            name: format!("lda {}, imm16", addr_reg.name),
            istr_type: InstructionType::Single,
        });
    }
}

// mar/sp = imm16
fn mv_addr_reg_instructions(destination: &mut Vec<NamedInstruction>) {
    let base_template = vec![
        *UNIVERSAL_STEP_0,
        [MEM.bout, Addr1Out, Addr0HiWrite, PC.cnt].into(), // first byte has msb
        [MEM.bout, Addr1Out, Addr0LoWrite].into(),         // second byte has lsb
        [PC.cnt, Reset].into(),                            // pc cnt
    ];

    for addr_reg0 in ADDR_REGISTERS.iter() {
        for addr_reg1 in ADDR_REGISTERS.iter().filter(|e| e.name != addr_reg0.name) {
            let mut current = base_template.clone();

            fill_addr_reg0(&mut current, addr_reg0);
            fill_addr_reg1(&mut current, addr_reg1);

            assert!(addr_reg_filled(&current));

            destination.push(NamedInstruction {
                istr: current,
                name: format!("mva {}, {}", addr_reg0.name, addr_reg1.name),
                istr_type: InstructionType::Single,
            });
        }
    }
}

fn addr_cnt_instructions(destination: &mut Vec<NamedInstruction>) {
    // sp dec
    let sp_dec = vec![
        *UNIVERSAL_STEP_0,
        [PC.cnt, SP.dec, Reset].into(), // decrement sp
    ];

    destination.push(NamedInstruction {
        istr: sp_dec,
        name: "dec SP".to_string(),
        istr_type: InstructionType::Single,
    });

    // sp cnt
    let sp_inc = vec![*UNIVERSAL_STEP_0, [PC.cnt, SP.inc, Reset].into()];

    destination.push(NamedInstruction {
        istr: sp_inc,
        name: "inc SP".to_string(),
        istr_type: InstructionType::Single,
    });

    // mar cnt
    let mar_inc = vec![
        *UNIVERSAL_STEP_0,
        [PC.cnt, MAR.cnt, Reset].into(), // cntrement mar
    ];

    destination.push(NamedInstruction {
        istr: mar_inc,
        name: "inc MAR".to_string(),
        istr_type: InstructionType::Single,
    });
}

//
// // jnz reg -> pc = mar if reg != 0 else nop
// IstrTemplateType jnzTemplateReg = [
//     universalStep0,
//     [A.write, Reg0Bout,
//      PC.cnt].into(),          // note: pc cnt happens in case jump doesn't happens
//     [FLAGS.aluWrite].into(), // write zero result to flag register
//     [MAR.HI.bout, PC.HI.write, FLAGS.SELECT.zero].into(),
//     [MAR.LO.bout, PC.LO.write, FLAGS.SELECT.zero, Reset].into(),
// ].into();
//
// // can save instruction if a is already loaded
// IstrTemplateType jnzTemplateRegA = [
//     universalStep0,
//     [FLAGS.aluWrite, PC.cnt].into(), // update flag register
//     [MAR.HI.bout, PC.HI.write, FLAGS.SELECT.zero].into(),
//     [MAR.LO.bout, PC.LO.write, FLAGS.SELECT.zero, Reset].into(),
// ].into();
//
// // jump if equal flag is carry flag is true
// IstrTemplateType jmpImm16Template = [
//     loadAddressProcedure[0],
//     loadAddressProcedure[1],
//     loadAddressProcedure[2],
//     loadAddressProcedure[3],
//     loadAddressProcedure[4],
//     [PC.cnt].into(), // note: pc cnt happens in case jump doesn't happens
//     [MAR.HI.bout, PC.HI.write,
//      outputFlagsSelector].into(), // load from mar into pc if flag
//     [MAR.LO.bout, PC.LO.write, outputFlagsSelector,
//      Reset].into(), // load from mar into pc if flag
// ].into();
//
// // jump if equal flag is true
// IstrTemplateType jmpMarTemplate = [
//     universalStep0,
//     [PC.cnt].into(), // note: pc cnt in case jump doesn't happen
//     [MAR.HI.bout, PC.HI.write, outputFlagsSelector].into(),
//     [MAR.LO.bout, PC.LO.write, outputFlagsSelector, Reset].into(),
// ].into();
//
// // todo: all math variants could have faster variants if Reg0/Reg1 are equal to
// // a/b todo: special case if Reg0 = b, Reg1 = a (impossible to swap registers
// // without intermediate)
//
// // Reg0 = Reg0 op Reg1
// IstrTemplateType mathCarryTemplateReg = [
//     universalStep0,
//     universalStep1,
//     [MEM.bout, PC.addr, ir2.write].into(), // need to load ir2 to figure out Reg1
//     [Reg0Bout, A.write, PC.cnt].into(),    // load Reg0 into a
//     [Reg1Bout, B.write].into(),             // load Reg1 into b
//     [fAluBout, FLAGS.aluWrite, Reg0Write,
//      FLAGS.SELECT.carry].into(), // do math op, save to Reg0, writes to flag reg
//     [Reset].into(),
// ].into();
//
// IstrTemplateType mathNoCarryTemplateReg = [
//     universalStep0,
//     universalStep1,
//     [MEM.bout, PC.addr, ir2.write].into(), // need to load ir2 to figure out Reg1
//     [Reg0Bout, A.write, PC.cnt].into(),    // load Reg0 into a
//     [Reg1Bout, B.write].into(),             // load Reg1 into b
//     [fAluBout, FLAGS.aluWrite, Reg0Write,
//      FLAGS.SELECT.direct].into(), // do math op, save to Reg0, writes to flag reg
//     [Reset].into(),
// ].into();
//
// // Reg0 = Reg0 op Reg1
// IstrTemplateType mathCarryTemplateImm = [
//     universalStep0,
//     [Reg0Bout, A.write,
//      PC.cnt].into(), // load Reg0 into a first (in case Reg0 = b), pc cnt
//     [MEM.bout, PC.addr, B.write].into(), // load imm into b
//     [fAluBout, flagWriteAlu, Reg0Write, FLAGS.SELECT.carry,
//      PC.cnt].into(), // save f to Reg0, writes to flag reg
//     [Reset].into(),
// ].into();
//
// IstrTemplateType mathNoCarryTemplateImm = [
//     universalStep0,
//     [Reg0Bout, A.write,
//      PC.cnt].into(), // load Reg0 into a first (in case Reg0 = b), pc cnt
//     [MEM.bout, PC.addr, B.write].into(), // load imm into b
//     [fAluBout, flagWriteAlu, Reg0Write, FLAGS.SELECT.direct,
//      PC.cnt].into(), // save f to Reg0, writes to flag reg
//     [Reset].into(),
// ].into();
//
// // Reg0 = ~Reg0
// IstrTemplateType notTemplateNone = [
//     universalStep0,
//     [Reg0Bout, A.write, PC.cnt].into(), // load Reg0 into a, pc cnt
//     [fAluBout, flagWriteAlu,
//      Reg0Write].into(), // do math op, save to Reg0, writes to flag reg
//     [Reset].into(),
// ].into();
//
// // Reg0 = ~Reg1
// IstrTemplateType notTemplateReg = [
//     universalStep0,
//     universalStep1,
//     [MEM.bout, PC.addr, ir2.write].into(), // need to load ir2 to figure out Reg1
//     [Reg1Bout, A.write, PC.cnt].into(),    // load Reg0 into a
//     [fAluBout, flagWriteAlu,
//      Reg0Write].into(), // do math op, save to Reg1, writes to flag reg
//     [Reset].into(),
// ].into();
//
// // sp <- imm16
// IstrTemplateType spTemplateImm16 = [
//     universalStep0,
//     [PC.cnt].into(),
//     [MEM.bout, PC.addr,
//      SP.HI.write].into(), // write first part of address to sp lo
//     [PC.cnt].into(),       // pc cnt
//     [MEM.bout, PC.addr,
//      SP.LO.write].into(), // write second part of address to sp hi
//     [Reset, PC.cnt].into(),
// ].into();
//
// // Reg0 = Reg0 op Reg1
// IstrTemplateType cmpTemplateReg = [
//     universalStep0,
//     universalStep1,
//     [MEM.bout, PC.addr, ir2.write].into(), // need to load ir2 to figure out Reg1
//     [Reg0Bout, A.write, PC.cnt].into(),    // load Reg0 into a
//     [Reg1Bout, B.write].into(),             // load Reg1 into b
//     [FLAGS.aluWrite].into(),                // writes to flag reg
//     [Reset].into(),
// ].into();
//
// // Reg0 = Reg0 op Reg1
// IstrTemplateType cmpTemplateImm = [
//     universalStep0,
//     [Reg0Bout, A.write,
//      PC.cnt].into(), // load Reg0 into a first (in case Reg0 = b), pc cnt
//     [MEM.bout, PC.addr, B.write].into(), // load imm into b
//     [FLAGS.aluWrite, PC.cnt].into(),
//     [Reset].into(),
// ].into();
//
// // Reg0 = keyboard input
// IstrTemplateType keyboardTemplate = [
//     universalStep0,
//     [keybBout, Reg0Write, PC.cnt, Reset].into(),
// ].into();
//
// // Reg0 = keyboard input
// IstrTemplateType updateFlagRegisterTemplate = [
//     universalStep0,
//     [flagWriteAlu, PC.cnt].into(),
//     [Reset].into(),
// ].into();
//
// IstrTemplateType haltTemplate = [
//     universalStep0,
//     [halt].into(),
// ].into();
//
// // 2 instruction nop
// IstrTemplateType nopTemplate = [
//     universalStep0,
//     [PC.cnt, Reset].into(),
// ].into();
//
// IstrTemplateType vramReadTemplateNoDelay = [
//     universalStep0,
//     [VRAM.bout, MAR.addr].into(), // note: must add register write manually
//     [nop].into(),
//     [PC.cnt, Reset].into(),
// ].into();
//
// IstrTemplateType vramReadTemplateDelay = [
//     universalStep0,
//     [nop].into(),
//     [VRAM.bout, MAR.addr].into(), // note: must add register write manually
//     [PC.cnt, Reset].into(),
// ].into();
//
// IstrTemplateType vramWriteTemplate = [
//     universalStep0,
//     [VRAM.write, MAR.addr].into(),
//     [VRAM.write, MAR.addr].into(), // note: must add register bout manually
//     [PC.cnt, Reset].into(),         /// todo: can add MAR.cnt
// ].into();

pub fn build_all_instructions() -> Vec<NamedInstruction> {
    let mut all_istrs: Vec<NamedInstruction> = Vec::new();
    move_word_reg_instructions(&mut all_istrs);
    move_word_imm_instructions(&mut all_istrs);

    lw_template_mar_instructions(&mut all_istrs);
    lw_template_imm16_instructions(&mut all_istrs);

    sw_template_addr_reg_instructions(&mut all_istrs);
    sw_template_imm16_instructions(&mut all_istrs);
    push_reg_instructions(&mut all_istrs);
    push_imm8_instructions(&mut all_istrs);
    pop_addr_reg_instructions(&mut all_istrs);
    lda_imm16_instructions(&mut all_istrs);
    mv_addr_reg_instructions(&mut all_istrs);
    addr_cnt_instructions(&mut all_istrs);

    all_istrs
}
