use crate::{
    action::Action::{self, *},
    instructions::register_defs::*,
    step_template::StepTemplate,
};

use std::sync::LazyLock;

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

pub fn replace_action(istr_temp: &mut [StepTemplate], pattern: Action, replacement: Action) {
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
        RegisterImpl::BoutWrite(reg) => set_reg0(istr_temp, reg),
        RegisterImpl::Write(write_reg) => set_reg0_write(istr_temp, write_reg),
        RegisterImpl::Bout(bout_reg) => set_reg0_bout(istr_temp, bout_reg),
    };
}

pub fn fill_reg1(istr_temp: &mut [StepTemplate], reg: &NamedRegister) {
    match reg.reg {
        RegisterImpl::BoutWrite(reg) => set_reg1(istr_temp, reg),
        RegisterImpl::Write(write_reg) => set_reg1_write(istr_temp, write_reg),
        RegisterImpl::Bout(bout_reg) => set_reg1_bout(istr_temp, bout_reg),
    };
}

pub fn fill_flag_select(istr_temp: &mut [StepTemplate], flag: Action) {
    replace_action(istr_temp, OutputFlagsSelector, flag);
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

pub fn flag_select_filled(istr_temp: &[StepTemplate]) -> bool {
    for step in istr_temp {
        for action in step.iter() {
            if matches!(action, OutputFlagsSelector) {
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
        AddressRegisterImpl::Mar(mar_register) => replace_addr0(istr_temp, mar_register),
        AddressRegisterImpl::Sp(sp_register) => replace_addr0(istr_temp, sp_register),
    };
}

pub fn fill_addr_reg1(istr_temp: &mut [StepTemplate], reg: &NamedAddressRegister) {
    match reg.reg {
        AddressRegisterImpl::Mar(mar_register) => replace_addr1(istr_temp, mar_register),
        AddressRegisterImpl::Sp(sp_register) => replace_addr1(istr_temp, sp_register),
    };
}

#[derive(PartialEq, Debug, Clone)]
pub enum RegisterImpl<'a> {
    BoutWrite(&'a BoutWriteRegister),
    Write(&'a WriteRegister),
    Bout(&'a BoutRegister),
}

impl<'a> RegisterImpl<'a> {
    fn can_bout(&self) -> bool {
        matches!(self, RegisterImpl::BoutWrite(_) | RegisterImpl::Bout(_))
    }
    fn can_write(&self) -> bool {
        matches!(self, RegisterImpl::BoutWrite(_) | RegisterImpl::Write(_))
    }
}

#[derive(Clone)]
pub enum AddressRegisterImpl<'a> {
    Mar(&'a MarRegister),
    Sp(&'a SpRegister),
}

#[derive(Clone)]
pub struct NamedRegister<'a> {
    pub reg: RegisterImpl<'a>,
    pub name: &'a str,
}

#[derive(Clone)]
pub struct NamedAddressRegister<'a> {
    pub reg: AddressRegisterImpl<'a>,
    pub name: &'a str,
}

#[derive(PartialEq)]
pub enum Register {
    A,
    B,
    X,
    Y,
    Z,
    MAR_lo,
    MAR_hi,
    PC_lo,
    PC_hi,
    SP_lo,
    SP_hi,
    FLAGS,
    KEYB,
}

impl Register {
    pub fn name(&self) -> &str {
        match self {
            Register::A => "A",
            Register::B => "B",
            Register::X => "X",
            Register::Y => "Y",
            Register::Z => "Z",
            Register::MAR_lo => "MAR.lo",
            Register::MAR_hi => "MAR.hi",
            Register::PC_lo => "PC.lo",
            Register::PC_hi => "PC.hi",
            Register::SP_lo => "SP.lo",
            Register::SP_hi => "SP.hi",
            Register::FLAGS => "FLAGS",
            Register::KEYB => "KEYB",
        }
    }

    pub fn to_reg_impl(&self) -> RegisterImpl {
        match self {
            Register::A => RegisterImpl::BoutWrite(&A),
            Register::B => RegisterImpl::BoutWrite(&B),
            Register::X => RegisterImpl::BoutWrite(&X.register),
            Register::Y => RegisterImpl::BoutWrite(&Y.register),
            Register::Z => RegisterImpl::BoutWrite(&Z),
            Register::MAR_lo => RegisterImpl::BoutWrite(&MAR.lo),
            Register::MAR_hi => RegisterImpl::BoutWrite(&MAR.hi),
            Register::PC_lo => RegisterImpl::BoutWrite(&PC.lo),
            Register::PC_hi => RegisterImpl::BoutWrite(&PC.hi),
            Register::SP_lo => RegisterImpl::BoutWrite(&SP.lo),
            Register::SP_hi => RegisterImpl::BoutWrite(&SP.hi),
            Register::FLAGS => RegisterImpl::Bout(&FLAGS),
            Register::KEYB => RegisterImpl::Bout(&KEYB),
        }
    }

    pub fn to_reg(&self) -> NamedRegister {
        NamedRegister {
            reg: self.to_reg_impl(),
            name: self.name(),
        }
    }

    pub fn iterator() -> std::slice::Iter<'static, Register> {
        static ALL_REGISTERS: [Register; 13] = [
            Register::A,
            Register::B,
            Register::X,
            Register::Y,
            Register::Z,
            Register::MAR_lo,
            Register::MAR_hi,
            Register::PC_lo,
            Register::PC_hi,
            Register::SP_lo,
            Register::SP_hi,
            Register::FLAGS,
            Register::KEYB,
        ];
        ALL_REGISTERS.iter()
    }

    pub fn read_iterator() -> impl Iterator<Item = &'static Register> {
        Register::iterator().filter(|e| e.to_reg_impl().can_bout())
    }

    pub fn write_iterator() -> impl Iterator<Item = &'static Register> {
        Register::iterator().filter(|e| e.to_reg_impl().can_write())
    }
}

#[derive(PartialEq)]
pub enum AddressRegisterEnum {
    Mar,
    Sp,
}

impl AddressRegisterEnum {
    pub fn name(&self) -> &str {
        match self {
            AddressRegisterEnum::Mar => "MAR",
            AddressRegisterEnum::Sp => "SP",
        }
    }

    pub fn to_reg_impl(&self) -> AddressRegisterImpl {
        match self {
            AddressRegisterEnum::Mar => AddressRegisterImpl::Mar(&MAR),
            AddressRegisterEnum::Sp => AddressRegisterImpl::Sp(&SP),
        }
    }

    pub fn to_reg(&self) -> NamedAddressRegister {
        NamedAddressRegister {
            reg: self.to_reg_impl(),
            name: self.name(),
        }
    }

    pub fn iterator() -> std::slice::Iter<'static, AddressRegisterEnum> {
        static ALL_REGISTERS: [AddressRegisterEnum; 2] =
            [AddressRegisterEnum::Mar, AddressRegisterEnum::Sp];
        ALL_REGISTERS.iter()
    }
}

pub static ALL_REGISTERS: LazyLock<Vec<NamedRegister>> = LazyLock::new(|| {
    vec![
        NamedRegister {
            reg: RegisterImpl::BoutWrite(&A),
            name: "A",
        },
        NamedRegister {
            reg: RegisterImpl::BoutWrite(&B),
            name: "B",
        },
        NamedRegister {
            reg: RegisterImpl::BoutWrite(&X.register),
            name: "X",
        },
        NamedRegister {
            reg: RegisterImpl::BoutWrite(&Y.register),
            name: "Y",
        },
        NamedRegister {
            reg: RegisterImpl::BoutWrite(&Z),
            name: "Z",
        },
        NamedRegister {
            reg: RegisterImpl::BoutWrite(&MAR.lo),
            name: "MAR.lo",
        },
        NamedRegister {
            reg: RegisterImpl::BoutWrite(&MAR.hi),
            name: "MAR.hi",
        },
        NamedRegister {
            reg: RegisterImpl::BoutWrite(&PC.lo),
            name: "PC.lo",
        },
        NamedRegister {
            reg: RegisterImpl::BoutWrite(&PC.hi),
            name: "PC.hi",
        },
        NamedRegister {
            reg: RegisterImpl::BoutWrite(&SP.lo),
            name: "SP.lo",
        },
        NamedRegister {
            reg: RegisterImpl::BoutWrite(&SP.hi),
            name: "SP.hi",
        },
        NamedRegister {
            reg: RegisterImpl::Bout(&FLAGS),
            name: "FLAGS",
        },
        NamedRegister {
            reg: RegisterImpl::Bout(&KEYB),
            name: "KEYB",
        },
    ]
});

pub static READ_REGISTERS: LazyLock<Vec<NamedRegister>> = LazyLock::new(|| {
    ALL_REGISTERS
        .clone()
        .into_iter()
        .filter(|e| matches!(e.reg, RegisterImpl::BoutWrite(_) | RegisterImpl::Bout(_)))
        .collect()
});

pub static WRITE_REGISTERS: LazyLock<Vec<NamedRegister>> = LazyLock::new(|| {
    ALL_REGISTERS
        .clone()
        .into_iter()
        .filter(|e| matches!(e.reg, RegisterImpl::BoutWrite(_) | RegisterImpl::Write(_)))
        .collect()
});

pub static ADDR_REGISTERS: LazyLock<Vec<NamedAddressRegister>> = LazyLock::new(|| {
    vec![
        NamedAddressRegister {
            reg: AddressRegisterImpl::Sp(&SP),
            name: "SP",
        },
        NamedAddressRegister {
            reg: AddressRegisterImpl::Mar(&MAR),
            name: "MAR",
        },
    ]
});

// in order of associated bits (0 is first, 7 is last)
pub enum MathIstrTypes {
    SubNoCarry,
    SubCarry,
    AddNoCarry,
    AddCarry,
    Not,
    Xor,
    Or,
    And,
    Cmp, // not different than subNoCarry in associated bits, but nice to include in enum
}

impl std::fmt::Display for MathIstrTypes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            MathIstrTypes::SubNoCarry => "sub",
            MathIstrTypes::SubCarry => "sbb",
            MathIstrTypes::AddNoCarry => "add",
            MathIstrTypes::AddCarry => "adc",
            MathIstrTypes::Not => "not",
            MathIstrTypes::Xor => "xor",
            MathIstrTypes::Or => "or",
            MathIstrTypes::And => "and",
            MathIstrTypes::Cmp => "cmp",
        };
        write!(f, "{name}")
    }
}

impl MathIstrTypes {
    pub fn iterator() -> std::slice::Iter<'static, MathIstrTypes> {
        static DIRECTIONS: [MathIstrTypes; 9] = [
            MathIstrTypes::SubNoCarry,
            MathIstrTypes::SubCarry,
            MathIstrTypes::AddNoCarry,
            MathIstrTypes::AddCarry,
            MathIstrTypes::Not,
            MathIstrTypes::Xor,
            MathIstrTypes::Or,
            MathIstrTypes::And,
            MathIstrTypes::Cmp,
        ];
        DIRECTIONS.iter()
    }

    pub fn get_action(&self) -> Action {
        if matches!(self, MathIstrTypes::SubCarry | MathIstrTypes::AddCarry) {
            FlagCarry
        } else {
            FlagDirect
        }
    }
}

// in order of associated bits (0 is first, 7 is last)
pub enum OutputFlags {
    Direct,
    Carry,
    Eq,
    NotZero,
    F5,
    F6,
    F7,
    F8,
}

impl OutputFlags {
    pub fn iterator() -> std::slice::Iter<'static, OutputFlags> {
        static OUTPUT_FLAGS: [OutputFlags; 8] = [
            OutputFlags::Direct,
            OutputFlags::Carry,
            OutputFlags::Eq,
            OutputFlags::NotZero,
            OutputFlags::F5,
            OutputFlags::F6,
            OutputFlags::F7,
            OutputFlags::F8,
        ];
        OUTPUT_FLAGS.iter()
    }

    pub fn get_action(&self) -> Action {
        match self {
            OutputFlags::Direct => FlagDirect,
            OutputFlags::Carry => FlagCarry,
            OutputFlags::Eq => FlagEq,
            OutputFlags::NotZero => FlagNotZero,
            OutputFlags::F5 => Flag5,
            OutputFlags::F6 => Flag6,
            OutputFlags::F7 => Flag7,
            OutputFlags::F8 => Flag8,
        }
    }

    pub fn get_jump_name(&self) -> &str {
        match self {
            OutputFlags::Direct => "jmp",
            OutputFlags::Carry => "jc",
            OutputFlags::Eq => "jeq",
            OutputFlags::NotZero => "jnz",
            OutputFlags::F5 => "j5",
            OutputFlags::F6 => "j6",
            OutputFlags::F7 => "j7",
            OutputFlags::F8 => "j8",
        }
    }
}
