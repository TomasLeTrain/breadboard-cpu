use crate::{
    action::Action::{self, *},
    step_template::StepTemplate,
    step_template_to_output,
};

use std::sync::LazyLock;

#[derive(PartialEq, Debug)]
struct BoutRegister {
    bout: Action,
}
#[derive(PartialEq, Debug)]
struct WriteRegister {
    write: Action,
}
#[derive(PartialEq, Debug)]
struct BoutWriteRegister {
    bout: Action,
    write: Action,
}

struct ShiftRegister {
    register: BoutWriteRegister,
    shift_left: Action,
    shift_right: Action,
}

struct PcRegister {
    addr: Action,
    lo: BoutWriteRegister,
    hi: BoutWriteRegister,
    cnt: Action,
}

struct MarRegister {
    addr: Action,
    lo: BoutWriteRegister,
    hi: BoutWriteRegister,
    cnt: Action,
}

struct SpRegister {
    addr: Action,
    lo: BoutWriteRegister,
    hi: BoutWriteRegister,
    inc: Action,
    dec: Action,
}

struct MemStruct {
    bout: Action,
    write: Action,
}

static PC: LazyLock<PcRegister> = LazyLock::new(|| PcRegister {
    addr: PcAddr,
    lo: BoutWriteRegister {
        bout: PcLoBout,
        write: PcLoWrite,
    },
    hi: BoutWriteRegister {
        bout: PcHiBout,
        write: PcHiWrite,
    },
    cnt: PcCnt,
});

static MAR: LazyLock<MarRegister> = LazyLock::new(|| MarRegister {
    addr: MarAddr,
    lo: BoutWriteRegister {
        bout: MarLoBout,
        write: MarLoWrite,
    },
    hi: BoutWriteRegister {
        bout: MarHiBout,
        write: MarHiWrite,
    },
    cnt: MarCnt,
});

static SP: LazyLock<SpRegister> = LazyLock::new(|| SpRegister {
    addr: SpAddr,
    lo: BoutWriteRegister {
        bout: SpLoBout,
        write: SpLoWrite,
    },
    hi: BoutWriteRegister {
        bout: SpHiBout,
        write: SpHiWrite,
    },
    inc: SpInc,
    dec: SpDec,
});

static MEM: LazyLock<MemStruct> = LazyLock::new(|| MemStruct {
    bout: MemRead,
    write: MemWrite,
});

static A: LazyLock<BoutWriteRegister> = LazyLock::new(|| BoutWriteRegister {
    bout: ABout,
    write: AWrite,
});
static B: LazyLock<BoutWriteRegister> = LazyLock::new(|| BoutWriteRegister {
    bout: BBout,
    write: BWrite,
});

static X: LazyLock<ShiftRegister> = LazyLock::new(|| ShiftRegister {
    register: BoutWriteRegister {
        bout: XBout,
        write: XWrite,
    },
    shift_left: XShiftLeft,
    shift_right: XShiftRight,
});

static Y: LazyLock<ShiftRegister> = LazyLock::new(|| ShiftRegister {
    register: BoutWriteRegister {
        bout: YBout,
        write: YWrite,
    },
    shift_left: YShiftLeft,
    shift_right: YShiftRight,
});

static Z: LazyLock<BoutWriteRegister> = LazyLock::new(|| BoutWriteRegister {
    bout: ZBout,
    write: ZWrite,
});

static IR2: LazyLock<WriteRegister> = LazyLock::new(|| WriteRegister { write: Ir2Write });

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
    istr: IstrTemplate,
    name: String,
}

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

    let common = [
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
    ];

    //
    let lhs = common.clone();

    // FIXME: add flags_reg,  keyb_reg
    let rhs = common.clone();

    //
    for l in &lhs {
        for r in &rhs {
            // avoid duplicates
            if l.name == r.name {
                continue;
            }

            let mut current = {
                if l.name == "PC.lo" || l.name == "PC.hi" {
                    &base_template_pc
                } else {
                    &base_template
                }
            }
            .clone();

            match l.reg {
                Registers::BoutWrite(reg) => set_reg0(&mut current, reg),
                Registers::Write(write_reg) => set_reg0_write(&mut current, write_reg),
                Registers::Bout(bout_reg) => set_reg0_bout(&mut current, bout_reg),
            };
            match r.reg {
                Registers::BoutWrite(reg) => set_reg1(&mut current, reg),
                Registers::Write(write_reg) => set_reg1_write(&mut current, write_reg),
                Registers::Bout(bout_reg) => set_reg1_bout(&mut current, bout_reg),
            };

            destination.push(NamedInstruction {
                istr: current,
                name: format!("mv {}, {}", l.name, r.name),
            });
        }
    }

    // TODO: add somewhere a check that all template actions are filled
}

// move register to register (reg0 = reg1)
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

    let common = [
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
    ];

    //
    let regs = common.clone();

    //
    for reg in &regs {
        let mut current = {
            if reg.name == "PC.lo" || reg.name == "PC.hi" {
                &base_template_pc
            } else {
                &base_template
            }
        }
        .clone();

        match reg.reg {
            Registers::BoutWrite(reg) => set_reg0(&mut current, reg),
            Registers::Write(write_reg) => set_reg0_write(&mut current, write_reg),
            Registers::Bout(bout_reg) => set_reg0_bout(&mut current, bout_reg),
        };

        destination.push(NamedInstruction {
            istr: current,
            name: format!("mv {}, imm8", reg.name),
        });
    }

    // TODO: add somewhere a check that all template actions are filled
}

pub fn build_all_instructions() {
    let mut all_istrs: Vec<NamedInstruction> = Vec::new();
    move_word_reg_instructions(&mut all_istrs);
    move_word_imm_instructions(&mut all_istrs);

    for istr in all_istrs {
        println!("istr: {}", istr.name);
        step_template_to_output(istr.istr.first().unwrap());
    }
}
