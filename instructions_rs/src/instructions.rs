use crate::{
    action::Action::{self, *},
    step_template::StepTemplate,
    step_template_to_output,
};

mod register_defs;
use register_defs::*;

use std::{ops::Deref, str::Matches, sync::LazyLock};

type IstrTemplate = Vec<StepTemplate>;

static UNIVERSAL_STEP_0: LazyLock<StepTemplate> =
    LazyLock::new(|| [MemRead, PcAddr, IrWrite].into());
static UNIVERSAL_STEP_1: LazyLock<StepTemplate> = LazyLock::new(|| [PcCnt].into());
static LOAD_IR2: LazyLock<StepTemplate> = LazyLock::new(|| [MEM.bout, PC.addr, IR2.write].into());

static LOAD_ADDRESS_PROCEDURE: LazyLock<IstrTemplate> = LazyLock::new(|| {
    vec![
        *UNIVERSAL_STEP_0,
        [PC.cnt].into(),
        [PC.cnt, PC.addr, MAR.hi.write].into(), // first byte has msb
        [PC.cnt].into(),                        // pc cnt
        [MEM.bout, PC.addr, MAR.lo.write].into(), // second byte has lsb
    ]
});

trait Bout {
    fn bout(&self) -> Action;
}
trait Write {
    fn write(&self) -> Action;
}

impl Bout for BoutWriteRegister {
    fn bout(&self) -> Action {
        self.bout
    }
}

impl Write for BoutWriteRegister {
    fn write(&self) -> Action {
        self.write
    }
}

impl Bout for BoutRegister {
    fn bout(&self) -> Action {
        self.bout
    }
}

impl Write for WriteRegister {
    fn write(&self) -> Action {
        self.write
    }
}

fn replace_action(istr_temp: &mut [StepTemplate], pattern: Action, replacement: Action) {
    for step in istr_temp {
        for action in step.iter_mut() {
            if *action == pattern {
                *action = replacement;
            }
        }
    }
}

fn set_reg0_write(istr_temp: &mut [StepTemplate], reg0: &impl Write) {
    replace_action(istr_temp, Reg0Write, reg0.write());
}

fn set_reg0_bout(istr_temp: &mut [StepTemplate], reg0: &impl Bout) {
    replace_action(istr_temp, Reg0Bout, reg0.bout());
}

fn set_reg0(istr_temp: &mut [StepTemplate], reg0: &(impl Bout + Write)) {
    set_reg0_write(istr_temp, reg0);
    set_reg0_bout(istr_temp, reg0);
}

fn set_reg1_write(istr_temp: &mut [StepTemplate], reg1: &impl Write) {
    replace_action(istr_temp, Reg1Write, reg1.write());
}

fn set_reg1_bout(istr_temp: &mut [StepTemplate], reg1: &impl Bout) {
    replace_action(istr_temp, Reg1Bout, reg1.bout());
}

fn set_reg1(istr_temp: &mut [StepTemplate], reg1: &(impl Bout + Write)) {
    set_reg1_write(istr_temp, reg1);
    set_reg1_bout(istr_temp, reg1);
}

fn fill_reg0(istr_temp: &mut [StepTemplate], reg: &NamedRegister) {
    match reg.reg {
        Registers::BoutWrite(reg) => set_reg0(istr_temp, reg),
        Registers::Write(write_reg) => set_reg0_write(istr_temp, write_reg),
        Registers::Bout(bout_reg) => set_reg0_bout(istr_temp, bout_reg),
    };
}
fn fill_reg1(istr_temp: &mut [StepTemplate], reg: &NamedRegister) {
    match reg.reg {
        Registers::BoutWrite(reg) => set_reg1(istr_temp, reg),
        Registers::Write(write_reg) => set_reg1_write(istr_temp, write_reg),
        Registers::Bout(bout_reg) => set_reg1_bout(istr_temp, bout_reg),
    };
}

fn all_regs_filled(istr_temp: &[StepTemplate]) -> bool {
    for step in istr_temp {
        for action in step.iter() {
            if matches!(action, Reg0Bout | Reg0Write | Reg1Bout | Reg1Write) {
                return false;
            }
        }
    }
    true
}

#[derive(PartialEq, Debug, Clone)]
enum Registers<'a> {
    BoutWrite(&'a BoutWriteRegister),
    Write(&'a WriteRegister),
    Bout(&'a BoutRegister),
}

#[derive(Clone)]
struct NamedRegister<'a> {
    reg: Registers<'a>,
    name: &'a str,
}

pub struct NamedInstruction {
    pub istr: IstrTemplate,
    pub name: String,
}

static ALL_REGISTERS: LazyLock<Vec<NamedRegister>> = LazyLock::new(|| {
    vec![
        NamedRegister {
            reg: Registers::BoutWrite(&A),
            name: "A",
        },
        NamedRegister {
            reg: Registers::BoutWrite(&B),
            name: "B",
        },
        NamedRegister {
            reg: Registers::BoutWrite(&X.register),
            name: "X",
        },
        NamedRegister {
            reg: Registers::BoutWrite(&Y.register),
            name: "Y",
        },
        NamedRegister {
            reg: Registers::BoutWrite(&Z),
            name: "Z",
        },
        NamedRegister {
            reg: Registers::BoutWrite(&MAR.lo),
            name: "MAR.lo",
        },
        NamedRegister {
            reg: Registers::BoutWrite(&MAR.hi),
            name: "MAR.hi",
        },
        NamedRegister {
            reg: Registers::BoutWrite(&PC.lo),
            name: "PC.lo",
        },
        NamedRegister {
            reg: Registers::BoutWrite(&PC.hi),
            name: "PC.hi",
        },
        NamedRegister {
            reg: Registers::BoutWrite(&SP.lo),
            name: "SP.lo",
        },
        NamedRegister {
            reg: Registers::BoutWrite(&SP.hi),
            name: "SP.hi",
        },
        NamedRegister {
            reg: Registers::Bout(&FLAGS),
            name: "FLAGS",
        },
        NamedRegister {
            reg: Registers::Bout(&KEYB),
            name: "KEYB",
        },
    ]
});

static READ_REGISTERS: LazyLock<Vec<NamedRegister>> = LazyLock::new(|| {
    ALL_REGISTERS
        .clone()
        .into_iter()
        .filter(|e| matches!(e.reg, Registers::BoutWrite(_) | Registers::Bout(_)))
        .collect()
});

static WRITE_REGISTERS: LazyLock<Vec<NamedRegister>> = LazyLock::new(|| {
    ALL_REGISTERS
        .clone()
        .into_iter()
        .filter(|e| matches!(e.reg, Registers::BoutWrite(_) | Registers::Write(_)))
        .collect()
});

// move register to register (reg0 = reg1)
// TODO: express extended/normal instruction!
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

    for reg0 in WRITE_REGISTERS.iter() {
        for reg1 in READ_REGISTERS.iter() {
            // avoid duplicates
            if reg0.name == reg1.name {
                continue;
            }

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
            });
        }
    }

    // TODO: add somewhere a check that all template actions are filled
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
        });
    }
}

// reg = [mar]
fn lw_template_mar_instructions(destination: &mut Vec<NamedInstruction>) {
    let base_template = vec![
        *UNIVERSAL_STEP_0,
        [MEM.bout, MAR.addr, Reg0Write, PC.cnt, Reset].into(), // read from addr mar into register, pc cnt
    ];

    // does not include the pc.cnt since PC is getting written to
    let base_template_pc = vec![
        *UNIVERSAL_STEP_0,
        [MEM.bout, MAR.addr, Reg0Write, Reset].into(), // read from addr mar into register, pc cnt
    ];

    // write to A to be able to write to MAR
    let base_template_mar = vec![
        *UNIVERSAL_STEP_0,
        [MEM.bout, MAR.addr, A.write, PC.cnt].into(), // read from addr mar into register, pc cnt
        [A.bout, Reg0Write, Reset].into(),            // read from addr mar into register, pc cnt
    ];

    for reg in WRITE_REGISTERS.iter() {
        let mut current = {
            if reg.name == "PC.lo" || reg.name == "PC.hi" {
                &base_template_pc
            } else if reg.name == "MAR.lo" || reg.name == "MAR.hi" {
                &base_template_mar
            } else {
                &base_template
            }
        }
        .clone();

        fill_reg0(&mut current, reg);

        assert!(all_regs_filled(&current));

        destination.push(NamedInstruction {
            istr: current,
            name: format!("lw {}, mem[MAR]", reg.name),
        });
    }
}

// reg = [imm16]
fn lw_template_imm16_instructions(destination: &mut Vec<NamedInstruction>) {
    let base_template = vec![
        LOAD_ADDRESS_PROCEDURE[0],
        LOAD_ADDRESS_PROCEDURE[1],
        LOAD_ADDRESS_PROCEDURE[2],
        LOAD_ADDRESS_PROCEDURE[3],
        LOAD_ADDRESS_PROCEDURE[4],
        [MEM.bout, MAR.addr, Reg0Write, PC.cnt, Reset].into(), // read from addr mar into register, pc cnt
    ];

    // does not include the pc.cnt since PC is getting written to
    let base_template_pc = vec![
        LOAD_ADDRESS_PROCEDURE[0],
        LOAD_ADDRESS_PROCEDURE[1],
        LOAD_ADDRESS_PROCEDURE[2],
        LOAD_ADDRESS_PROCEDURE[3],
        LOAD_ADDRESS_PROCEDURE[4],
        [MEM.bout, MAR.addr, Reg0Write, Reset].into(), // read from addr mar into register, pc cnt
    ];

    // write to A to be able to write to MAR
    let base_template_mar = vec![
        LOAD_ADDRESS_PROCEDURE[0],
        LOAD_ADDRESS_PROCEDURE[1],
        LOAD_ADDRESS_PROCEDURE[2],
        LOAD_ADDRESS_PROCEDURE[3],
        LOAD_ADDRESS_PROCEDURE[4],
        [MEM.bout, MAR.addr, A.write, PC.cnt].into(), // read from addr mar into register, pc cnt
        [A.bout, Reg0Write, Reset].into(),            // read from addr mar into register, pc cnt
    ];

    for reg in WRITE_REGISTERS.iter() {
        let mut current = {
            if reg.name == "PC.lo" || reg.name == "PC.hi" {
                &base_template_pc
            } else if reg.name == "MAR.lo" || reg.name == "MAR.hi" {
                &base_template_mar
            } else {
                &base_template
            }
        }
        .clone();

        fill_reg0(&mut current, reg);

        destination.push(NamedInstruction {
            istr: current,
            name: format!("lw {}, mem[imm16]", reg.name),
        });
    }

    // TODO: add somewhere a check that all template actions are filled
}

// fn lw_template_imm16_instructions(destination: &mut Vec<NamedInstruction>) {
//     let base_template = vec![
//         LOAD_ADDRESS_PROCEDURE[0],
//         LOAD_ADDRESS_PROCEDURE[1],
//         LOAD_ADDRESS_PROCEDURE[2],
//         LOAD_ADDRESS_PROCEDURE[3],
//         LOAD_ADDRESS_PROCEDURE[4],
//         [MEM.bout, MAR.addr, Reg0Write, PC.cnt, Reset].into(), // read from addr mar into register, pc cnt
//     ];
//
//     // does not include the pc.cnt since PC is getting written to
//     let base_template_pc = vec![
//         LOAD_ADDRESS_PROCEDURE[0],
//         LOAD_ADDRESS_PROCEDURE[1],
//         LOAD_ADDRESS_PROCEDURE[2],
//         LOAD_ADDRESS_PROCEDURE[3],
//         LOAD_ADDRESS_PROCEDURE[4],
//         [MEM.bout, MAR.addr, Reg0Write, Reset].into(), // read from addr mar into register, pc cnt
//     ];
//
//     // write to A to be able to write to MAR
//     let base_template_mar = vec![
//         LOAD_ADDRESS_PROCEDURE[0],
//         LOAD_ADDRESS_PROCEDURE[1],
//         LOAD_ADDRESS_PROCEDURE[2],
//         LOAD_ADDRESS_PROCEDURE[3],
//         LOAD_ADDRESS_PROCEDURE[4],
//         [MEM.bout, MAR.addr, A.write, PC.cnt].into(), // read from addr mar into register, pc cnt
//         [A.bout, Reg0Write, Reset].into(),            // read from addr mar into register, pc cnt
//     ];
//
//     for reg in WRITE_REGISTERS.iter() {
//         let mut current = {
//             if reg.name == "PC.lo" || reg.name == "PC.hi" {
//                 &base_template_pc
//             } else if reg.name == "MAR.lo" || reg.name == "MAR.hi" {
//                 &base_template_mar
//             } else {
//                 &base_template
//             }
//         }
//         .clone();
//
//         fill_reg0(&mut current, reg);
//
//         destination.push(NamedInstruction {
//             istr: current,
//             name: format!("lw {}, mem[imm16]", reg.name),
//         });
//     }
//
//     // TODO: add somewhere a check that all template actions are filled
// }

//
// // reg = [mar]
// IstrTemplateType swTemplateMar = [
//     universalStep0,
//     [MEM.write, MAR.addr, Reg0Bout, PC.cnt,
//      Reset].into(), // read from addr mar into register, pc cnt
// ].into();
//
// // reg = [imm16]
// IstrTemplateType swTemplateImm = [
//     loadAddressProcedure[0],
//     loadAddressProcedure[1],
//     loadAddressProcedure[2],
//     loadAddressProcedure[3],
//     loadAddressProcedure[4],
//     [MEM.write, MAR.addr, Reg0Bout, PC.cnt,
//      Reset].into(), // read from addr mar into register, pc cnt
// ].into();
//
// // [sp--] = reg
// IstrTemplateType pushTemplateReg = [
//     universalStep0,
//     [PC.cnt, SP.dec].into(), // decrement before pushing value
//     [MEM.write, SP.addr, Reg0Bout,
//      Reset].into(), // read from reg into mem at sp addr, pc cnt
// ].into();
//
// // [sp--] = imm8, overrides a reg
// IstrTemplateType pushTemplateImm8 = [
//     universalStep0,
//     [PC.cnt, SP.dec].into(),              // decrement before pushing value
//     [MEM.bout, PC.addr, A.write].into(), // write into ir2
//     [MEM.write, SP.addr, A.bout, PC.cnt,
//      Reset].into(), // read from ir2 into [sp], pc cnt
// ].into();
//
// // Reg0 = [sp++]
// IstrTemplateType popTemplate = [
//     universalStep0,
//     [MEM.bout, SP.addr, Reg0Write,
//      PC.cnt].into(),        // write from [sp] into Reg0, pc cnt
//     [Reset, SP.inc].into(), // cntrement after popping value
// ].into();
//
// // mar = imm16
// IstrTemplateType marTemplateImm16 = [
//     loadAddressProcedure[0], loadAddressProcedure[1],
//     loadAddressProcedure[2], loadAddressProcedure[3],
//     loadAddressProcedure[4], [Reset, PC.cnt].into(), // pc cnt
// ].into();
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
// // sp dec
// IstrTemplateType spDecTemplate = [
//     universalStep0,
//     [PC.cnt, SP.dec, Reset].into(), // decrement sp
// ].into();
//
// // sp cnt
// IstrTemplateType spIncTemplate = [
//     universalStep0,
//     [PC.cnt, SP.inc, Reset].into(),
// ].into();
//
// // mar cnt
// IstrTemplateType marCntTemplate = [
//     universalStep0,
//     [PC.cnt, MAR.cnt, Reset].into(), // cntrement mar
// ].into();
//
// // mar <- pc
// IstrTemplateType pcToMarTemplate = [
//     universalStep0,
//     universalStep1,
//     [PC.HI.bout, MAR.HI.write].into(),
//     [PC.LO.bout, MAR.LO.write, Reset].into(),
// ].into();
//
// // mar <- sp
// IstrTemplateType spToMarTemplate = [
//     universalStep0,
//     [SP.HI.bout, MAR.HI.write, PC.cnt].into(),
//     [SP.LO.bout, MAR.LO.write, Reset].into(),
// ].into();
//
// // sp <- mar
// IstrTemplateType marToSpTemplate = [
//     universalStep0,
//     [MAR.HI.bout, SP.HI.write, PC.cnt].into(),
//     [MAR.LO.bout, SP.LO.write, Reset].into(),
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

fn istr2(destination: &mut Vec<NamedInstruction>) {}

pub fn build_all_instructions() -> Vec<NamedInstruction> {
    let mut all_istrs: Vec<NamedInstruction> = Vec::new();
    move_word_reg_instructions(&mut all_istrs);
    move_word_imm_instructions(&mut all_istrs);
    lw_template_mar_instructions(&mut all_istrs);
    lw_template_imm16_instructions(&mut all_istrs);

    all_istrs
}
