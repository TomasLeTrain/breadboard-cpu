//! Definitions of various utils and static vars used mainly for instruction defs

use crate::{
    action::Action::{self, *},
    instructions::register_defs::*,
    step_template::StepTemplate,
};

use std::sync::LazyLock;

pub type IstrTemplateVec = Vec<StepTemplate>;

// PC.cnt needs to happen after these steps
pub static SIMPLE_LOAD_STEPS: LazyLock<Vec<StepTemplate>> = LazyLock::new(|| {
    vec![
        [MEM.bout, PC.addr, IrWrite].into(), // IR = mem[PC]
    ]
});

// PC.cnt needs to happen after these steps
pub static EXTENDED_LOAD_STEPS: LazyLock<Vec<StepTemplate>> = LazyLock::new(|| {
    vec![
        [MEM.bout, PC.addr, IrWrite].into(),           // IR = mem[PC]
        [PC.cnt, MEM.bout, PC.addr, IR2.write].into(), // PC++ -> ir2 = mem[PC]
    ]
});

/// PC should need to be incremented before using
pub static IMM_TO_ADDR_REG: LazyLock<IstrTemplateVec> = LazyLock::new(|| {
    // NOTE: PC should need to be incremented before using
    // NOTE: PC also needs to be increased after using
    vec![
        [PC.cnt, MEM.bout, PC.addr, Addr0HiWrite].into(), // PC++ -> reg.hi = mem[PC]
        [PC.cnt, MEM.bout, PC.addr, Addr0LoWrite].into(), // PC++ -> reg.lo = mem[PC]
    ]
});

/// Replaces all instances of pattern for replacement.
///
/// * `istr_temp`: template to modify
/// * `pattern`: pattern to look for
/// * `replacement`: replacement to replace pattern
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

/// Returns true if no addr related placeholders exist in istr_temp.
///
/// * `istr_temp`: template to verify
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

/// returns true if template has no instances of output flag placeholder.
///
/// * `istr_temp`: template to verify
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

fn replace_addr0(istr_temp: &mut [StepTemplate], reg: &impl AddressRegisterTrait) {
    replace_action(istr_temp, Addr0HiBout, reg.hi().bout());
    replace_action(istr_temp, Addr0HiWrite, reg.hi().write());
    replace_action(istr_temp, Addr0LoBout, reg.lo().bout());
    replace_action(istr_temp, Addr0LoWrite, reg.lo().write());
    replace_action(istr_temp, Addr0Out, reg.addr());
}

fn replace_addr1(istr_temp: &mut [StepTemplate], reg: &impl AddressRegisterTrait) {
    replace_action(istr_temp, Addr1HiBout, reg.hi().bout());
    replace_action(istr_temp, Addr1HiWrite, reg.hi().write());
    replace_action(istr_temp, Addr1LoBout, reg.lo().bout());
    replace_action(istr_temp, Addr1LoWrite, reg.lo().write());
    replace_action(istr_temp, Addr1Out, reg.addr());
}

impl AddressRegister {
    pub fn fill_addr_reg0(&self, istr_temp: &mut [StepTemplate]) {
        match self.to_reg_impl() {
            AddressRegisterImpl::Mar(mar_register) => replace_addr0(istr_temp, mar_register),
            AddressRegisterImpl::Sp(sp_register) => replace_addr0(istr_temp, sp_register),
        };
    }

    pub fn fill_addr_reg1(&self, istr_temp: &mut [StepTemplate]) {
        match self.to_reg_impl() {
            AddressRegisterImpl::Mar(mar_register) => replace_addr1(istr_temp, mar_register),
            AddressRegisterImpl::Sp(sp_register) => replace_addr1(istr_temp, sp_register),
        };
    }
}

#[derive(PartialEq, Debug, Clone)]
#[allow(dead_code)]
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

#[derive(PartialEq, Clone, Copy, Debug, Hash, Eq)]
pub enum Register {
    A,
    B,
    X,
    Y,
    Z,
    MarLo,
    MarHi,
    PcLo,
    PcHi,
    SpLo,
    SpHi,
    Flags,
    Keyb,
}

impl Register {
    pub fn fill_reg0(&self, istr_temp: &mut [StepTemplate]) {
        match self.to_reg_impl() {
            RegisterImpl::BoutWrite(reg) => set_reg0(istr_temp, reg),
            RegisterImpl::Write(write_reg) => set_reg0_write(istr_temp, write_reg),
            RegisterImpl::Bout(bout_reg) => set_reg0_bout(istr_temp, bout_reg),
        };
    }

    pub fn fill_reg1(&self, istr_temp: &mut [StepTemplate]) {
        match self.to_reg_impl() {
            RegisterImpl::BoutWrite(reg) => set_reg1(istr_temp, reg),
            RegisterImpl::Write(write_reg) => set_reg1_write(istr_temp, write_reg),
            RegisterImpl::Bout(bout_reg) => set_reg1_bout(istr_temp, bout_reg),
        };
    }

    pub fn name(&self) -> &str {
        match self {
            Register::A => "A",
            Register::B => "B",
            Register::X => "X",
            Register::Y => "Y",
            Register::Z => "Z",
            Register::MarLo => "MAR.lo",
            Register::MarHi => "MAR.hi",
            Register::PcLo => "PC.lo",
            Register::PcHi => "PC.hi",
            Register::SpLo => "SP.lo",
            Register::SpHi => "SP.hi",
            Register::Flags => "FLAGS",
            Register::Keyb => "KEYB",
        }
    }

    pub fn to_reg_impl(&self) -> RegisterImpl<'_> {
        match self {
            Register::A => RegisterImpl::BoutWrite(&A),
            Register::B => RegisterImpl::BoutWrite(&B),
            Register::X => RegisterImpl::BoutWrite(&X.register),
            Register::Y => RegisterImpl::BoutWrite(&Y.register),
            Register::Z => RegisterImpl::BoutWrite(&Z),
            Register::MarLo => RegisterImpl::BoutWrite(&MAR.lo),
            Register::MarHi => RegisterImpl::BoutWrite(&MAR.hi),
            Register::PcLo => RegisterImpl::BoutWrite(&PC.lo),
            Register::PcHi => RegisterImpl::BoutWrite(&PC.hi),
            Register::SpLo => RegisterImpl::BoutWrite(&SP.lo),
            Register::SpHi => RegisterImpl::BoutWrite(&SP.hi),
            Register::Flags => RegisterImpl::Bout(&FLAGS),
            Register::Keyb => RegisterImpl::Bout(&KEYB),
        }
    }

    pub fn iterator() -> std::slice::Iter<'static, Register> {
        static ALL_REGISTERS: [Register; 13] = [
            Register::A,
            Register::B,
            Register::X,
            Register::Y,
            Register::Z,
            Register::MarLo,
            Register::MarHi,
            Register::PcLo,
            Register::PcHi,
            Register::SpLo,
            Register::SpHi,
            Register::Flags,
            Register::Keyb,
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

#[derive(PartialEq, Clone, Copy, Debug, Hash, Eq)]
pub enum AddressRegister {
    Mar,
    Sp,
}

impl AddressRegister {
    pub fn name(&self) -> &str {
        match self {
            AddressRegister::Mar => "MAR",
            AddressRegister::Sp => "SP",
        }
    }

    pub fn to_reg_impl(&self) -> AddressRegisterImpl<'_> {
        match self {
            AddressRegister::Mar => AddressRegisterImpl::Mar(&MAR),
            AddressRegister::Sp => AddressRegisterImpl::Sp(&SP),
        }
    }

    pub fn iterator() -> std::slice::Iter<'static, AddressRegister> {
        static ALL_REGISTERS: [AddressRegister; 2] = [AddressRegister::Mar, AddressRegister::Sp];
        ALL_REGISTERS.iter()
    }
}

// in order of associated bits (0 is first, 7 is last)
#[derive(Clone, Copy, Debug, PartialEq)]
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
        write!(f, "{}", self.istr_name())
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

    pub fn as_action(&self) -> Action {
        if matches!(self, MathIstrTypes::SubCarry | MathIstrTypes::AddCarry) {
            FlagCarry
        } else {
            FlagDirect
        }
    }

    pub fn as_ir_bits(&self) -> u8 {
        match self {
            // the only difference between these is the flag_select (direct vs. carry flag)
            // both have their carry inverted
            MathIstrTypes::SubNoCarry | MathIstrTypes::SubCarry => 0,
            // this op does not have its carry inverted but the decoded alu select is the same as
            // sub
            // TODO: could support subNoCarry and SubCarry without carry inverted?
            MathIstrTypes::Cmp => 1,
            // TODO: could technically support add with inverted carry
            // (i.e. have it invert)
            MathIstrTypes::AddNoCarry => 2,
            MathIstrTypes::AddCarry => 3,
            MathIstrTypes::Not => 4,
            MathIstrTypes::Xor => 5,
            MathIstrTypes::Or => 6,
            MathIstrTypes::And => 7,
        }
    }

    pub fn istr_name(&self) -> &str {
        match self {
            MathIstrTypes::SubNoCarry => "sub",
            MathIstrTypes::SubCarry => "sbb",
            MathIstrTypes::AddNoCarry => "add",
            MathIstrTypes::AddCarry => "adc",
            MathIstrTypes::Not => "not",
            MathIstrTypes::Xor => "xor",
            MathIstrTypes::Or => "or",
            MathIstrTypes::And => "and",
            MathIstrTypes::Cmp => "cmp",
        }
    }
}

// in order of associated bits (0 is first, 7 is last)
#[derive(Clone, Copy, Debug, PartialEq)]
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
