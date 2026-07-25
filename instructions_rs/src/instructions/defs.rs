use crate::{
    action::Action::{self, *},
    instructions::register_defs::*,
    step_template::StepTemplate,
    step_template_to_output,
};

use std::{
    ops::{Deref, DerefMut},
    str::Matches,
    sync::LazyLock,
};

pub type IstrTemplate = Vec<StepTemplate>;

pub static UNIVERSAL_STEP_0: LazyLock<StepTemplate> =
    LazyLock::new(|| [MEM.bout, PC.addr, IrWrite].into());
pub static UNIVERSAL_STEP_1: LazyLock<StepTemplate> = LazyLock::new(|| [PC.cnt].into());
pub static LOAD_IR2: LazyLock<StepTemplate> =
    LazyLock::new(|| [MEM.bout, PC.addr, IR2.write].into());

pub static IMM_TO_ADDR_REG: LazyLock<IstrTemplate> = LazyLock::new(|| {
    vec![
        *UNIVERSAL_STEP_0,
        [PC.cnt].into(),
        [MEM.bout, PC.addr, Addr0HiWrite].into(), // first byte has msb
        [PC.cnt].into(),                          // pc cnt
        [MEM.bout, PC.addr, Addr0LoWrite].into(), // second byte has lsb
    ]
});

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

pub fn fill_reg0(istr_temp: &mut [StepTemplate], reg: &NamedRegister) {
    match reg.reg {
        Registers::BoutWrite(reg) => set_reg0(istr_temp, reg),
        Registers::Write(write_reg) => set_reg0_write(istr_temp, write_reg),
        Registers::Bout(bout_reg) => set_reg0_bout(istr_temp, bout_reg),
    };
}

pub fn fill_reg1(istr_temp: &mut [StepTemplate], reg: &NamedRegister) {
    match reg.reg {
        Registers::BoutWrite(reg) => set_reg1(istr_temp, reg),
        Registers::Write(write_reg) => set_reg1_write(istr_temp, write_reg),
        Registers::Bout(bout_reg) => set_reg1_bout(istr_temp, bout_reg),
    };
}

pub fn all_regs_filled(istr_temp: &[StepTemplate]) -> bool {
    for step in istr_temp {
        for action in step.iter() {
            if matches!(action, Reg0Bout | Reg0Write | Reg1Bout | Reg1Write) {
                return false;
            }
        }
    }
    true
}

pub fn addr_reg_filled(istr_temp: &[StepTemplate]) -> bool {
    for step in istr_temp {
        for action in step.iter() {
            if matches!(
                action,
                Addr0HiBout
                    | Addr0HiWrite
                    | Addr0LoBout
                    | Addr0LoWrite
                    | Addr0Out
                    | Addr1HiBout
                    | Addr1HiWrite
                    | Addr1LoBout
                    | Addr1LoWrite
                    | Addr1Out
            ) {
                return false;
            }
        }
    }
    true
}

fn replace_addr0(istr_temp: &mut [StepTemplate], reg: &impl AddressRegister) {
    replace_action(istr_temp, Addr0HiBout, reg.hi().bout());
    replace_action(istr_temp, Addr0HiWrite, reg.hi().write());
    replace_action(istr_temp, Addr0LoBout, reg.lo().bout());
    replace_action(istr_temp, Addr0LoWrite, reg.lo().write());
    replace_action(istr_temp, Addr0Out, reg.addr());
}

fn replace_addr1(istr_temp: &mut [StepTemplate], reg: &impl AddressRegister) {
    replace_action(istr_temp, Addr1HiBout, reg.hi().bout());
    replace_action(istr_temp, Addr1HiWrite, reg.hi().write());
    replace_action(istr_temp, Addr1LoBout, reg.lo().bout());
    replace_action(istr_temp, Addr1LoWrite, reg.lo().write());
    replace_action(istr_temp, Addr1Out, reg.addr());
}

pub fn fill_addr_reg0(istr_temp: &mut [StepTemplate], reg: &NamedAddressRegister) {
    match reg.reg {
        AddressRegisters::Mar(mar_register) => replace_addr0(istr_temp, mar_register),
        AddressRegisters::Sp(sp_register) => replace_addr0(istr_temp, sp_register),
    };
}

pub fn fill_addr_reg1(istr_temp: &mut [StepTemplate], reg: &NamedAddressRegister) {
    match reg.reg {
        AddressRegisters::Mar(mar_register) => replace_addr1(istr_temp, mar_register),
        AddressRegisters::Sp(sp_register) => replace_addr1(istr_temp, sp_register),
    };
}

#[derive(PartialEq, Debug, Clone)]
pub enum Registers<'a> {
    BoutWrite(&'a BoutWriteRegister),
    Write(&'a WriteRegister),
    Bout(&'a BoutRegister),
}

#[derive(Clone)]
pub enum AddressRegisters<'a> {
    Mar(&'a MarRegister),
    Sp(&'a SpRegister),
}

#[derive(Clone, Copy)]
pub enum InstructionType {
    Single,
    Extended,
}

#[derive(Clone)]
pub struct NamedRegister<'a> {
    pub reg: Registers<'a>,
    pub name: &'a str,
}

#[derive(Clone)]
pub struct NamedAddressRegister<'a> {
    pub reg: AddressRegisters<'a>,
    pub name: &'a str,
}

pub struct NamedInstruction {
    pub istr: IstrTemplate,
    pub name: String,
    pub istr_type: InstructionType,
}

pub static ALL_REGISTERS: LazyLock<Vec<NamedRegister>> = LazyLock::new(|| {
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

pub static READ_REGISTERS: LazyLock<Vec<NamedRegister>> = LazyLock::new(|| {
    ALL_REGISTERS
        .clone()
        .into_iter()
        .filter(|e| matches!(e.reg, Registers::BoutWrite(_) | Registers::Bout(_)))
        .collect()
});

pub static WRITE_REGISTERS: LazyLock<Vec<NamedRegister>> = LazyLock::new(|| {
    ALL_REGISTERS
        .clone()
        .into_iter()
        .filter(|e| matches!(e.reg, Registers::BoutWrite(_) | Registers::Write(_)))
        .collect()
});

pub static ADDR_REGISTERS: LazyLock<Vec<NamedAddressRegister>> = LazyLock::new(|| {
    vec![
        NamedAddressRegister {
            reg: AddressRegisters::Sp(&SP),
            name: "SP",
        },
        NamedAddressRegister {
            reg: AddressRegisters::Mar(&MAR),
            name: "MAR",
        },
    ]
});
