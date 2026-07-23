use std::{collections::HashMap, io::Read, ops::Deref};

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Action {
    Halt,
    Nop, // equal to zero outputs
    Reset,

    // addr regs cnt
    PcCnt,
    MarCnt,
    SpDec,
    SpInc,

    // addr regs addr
    PcAddr,
    MarAddr,
    SpAddr,

    // registers bout
    ABout,
    BBout,
    XBout,
    YBout,
    ZBout,
    PcLoBout,
    PcHiBout,
    MarLoBout,
    MarHiBout,
    SpLoBout,
    SpHiBout,
    KeybBout,
    FlagsBout,
    FAluBout,

    // registers write
    AWrite,
    BWrite,
    XWrite,
    YWrite,
    ZWrite,
    PcLoWrite,
    PcHiWrite,
    MarLoWrite,
    MarHiWrite,
    SpLoWrite,
    SpHiWrite,

    // ir regs
    IrWrite,
    Ir2Write,

    // mem read/write
    MemRead,
    MemWrite,

    // vram read/write
    VramRead,
    VramWrite,

    // register shifts
    XShiftLeft,
    YShiftLeft,
    XShiftRight,
    YShiftRight,

    // flags
    FlagDirect,
    FlagCarry,
    FlagEq,
    FlagZero,
    Flag6,
    Flag5,
    Flag7,
    Flag8,

    FlagWriteAlu,

    // placeholder registers
    Reg0Bout,
    Reg1Bout,
    Reg0Write,
    Reg1Write,
    OutputFlagsSelector,
}

use Action::*;
use std::sync::LazyLock;

// TODO: ensure values created are within their max sizes
struct Output {
    bout: u8,
    write: u8,
    addr: u8,
    misc: u8,
    flag_select: u8,
    pc_cnt: u8,
}

impl Output {
    fn new() -> Self {
        Self {
            bout: 0,
            write: 0,
            addr: 0,
            misc: 0,
            flag_select: 0,
            pc_cnt: 0,
        }
    }

    fn from_write(val: u8) -> Self {
        let mut result = Self::new();
        result.write = val;
        result
    }

    fn from_bout(val: u8) -> Self {
        let mut result = Self::new();
        result.bout = val;
        result
    }

    fn from_addr(val: u8) -> Self {
        let mut result = Self::new();
        result.addr = val;
        result
    }

    fn from_other(val: u8) -> Self {
        let mut result = Self::new();
        result.misc = val;
        result
    }

    fn from_flag_select(val: u8) -> Self {
        let mut result = Self::new();
        result.flag_select = val;
        result
    }

    fn from_pc_cnt(val: u8) -> Self {
        let mut result = Self::new();
        result.pc_cnt = val;
        result
    }

    fn from_arr(arr: &[Self]) -> Output {
        let mut result = Self::new();
        for curr in arr {
            result.merge(curr)
        }
        result
    }

    fn intersect(&self, other: &Self) -> bool {
        let category_intersects = |a: u8, b: u8| -> bool { a > 0 && b > 0 };

        category_intersects(self.bout, other.bout)
            || category_intersects(self.write, other.write)
            || category_intersects(self.addr, other.addr)
            || category_intersects(self.flag_select, other.flag_select)
            || category_intersects(self.pc_cnt, other.pc_cnt)
    }

    fn merge(&mut self, other: &Self) {
        // TODO: add intersect assert
        self.bout |= other.bout;
        self.write |= other.write;
        self.addr |= other.addr;
        self.misc |= other.misc;
        self.flag_select |= other.flag_select;
        self.pc_cnt |= other.pc_cnt;
    }
}

impl<const N: usize> From<[Output; N]> for Output {
    fn from(arr: [Output; N]) -> Self {
        let mut result = Self::new();
        for curr in arr.iter() {
            result.merge(curr)
        }
        result
    }
}

static ACTION_TO_OUTPUT_MAP: LazyLock<HashMap<Action, Output>> = LazyLock::new(|| {
    HashMap::from([
        (Halt, Output::from_bout(5)),
        (Nop, Output::new()),
        (Reset, Output::from_other(2)),
        // addr regs cnt
        (PcCnt, Output::from_pc_cnt(1)),
        (MarCnt, Output::from_bout(4)),
        (SpDec, Output::from_bout(7)),
        (SpInc, Output::from_bout(6)),
        // addr regs addr
        (PcAddr, Output::from_addr(1)),
        (MarAddr, Output::from_addr(2)),
        (SpAddr, Output::from_addr(3)),
        // registers bout
        (ABout, Output::from_bout(0b1000)),
        (BBout, Output::from_bout(0b1000 | 1)),
        (XBout, Output::from_bout(0b1000 | 5)),
        (YBout, Output::from_bout(0b1000 | 6)),
        (ZBout, Output::from_bout(0b1000 | 7)),
        (
            PcLoBout,
            [Output::from_addr(1), Output::from_bout(2)].into(),
        ),
        (
            PcHiBout,
            [Output::from_addr(1), Output::from_bout(3)].into(),
        ),
        (
            MarLoBout,
            [Output::from_addr(2), Output::from_bout(2)].into(),
        ),
        (
            MarHiBout,
            [Output::from_addr(2), Output::from_bout(3)].into(),
        ),
        (
            SpLoBout,
            [Output::from_addr(3), Output::from_bout(2)].into(),
        ),
        (
            SpHiBout,
            [Output::from_addr(3), Output::from_bout(3)].into(),
        ),
        (KeybBout, Output::from_bout(0b1000 | 2)),
        (FlagsBout, Output::from_bout(0b1000 | 4)),
        (FAluBout, Output::from_bout(0b1000 | 3)),
        // registers write
        (AWrite, Output::from_write(0b1000)),
        (BWrite, Output::from_write(0b1000 | 1)),
        (XWrite, Output::from_write(0b1000 | 5)),
        (YWrite, Output::from_write(0b1000 | 6)),
        (ZWrite, Output::from_write(7)),
        (PcLoWrite, Output::from_write(0b1000 | 3)),
        (PcHiWrite, Output::from_write(0b1000 | 2)),
        (MarLoWrite, Output::from_write(5)),
        (MarHiWrite, Output::from_write(4)),
        (SpLoWrite, Output::from_write(3)),
        (SpHiWrite, Output::from_write(2)),
        // ir regs
        (IrWrite, Output::from_write(0b1000 | 7)),
        (Ir2Write, Output::from_write(1)),
        // mem read/write
        (MemRead, Output::from_bout(1)),
        (MemWrite, Output::from_write(6)),
        // vram read/write
        (
            VramRead,
            [Output::from_other(3), Output::from_flag_select(5)].into(),
        ),
        (
            VramWrite,
            [Output::from_other(3), Output::from_flag_select(4)].into(),
        ),
        // shift left
        (
            XShiftLeft,
            [Output::from_other(3), Output::from_flag_select(0)].into(),
        ),
        (
            YShiftLeft,
            [Output::from_other(3), Output::from_flag_select(2)].into(),
        ),
        // shift right
        (
            XShiftRight,
            [Output::from_other(3), Output::from_flag_select(1)].into(),
        ),
        (
            YShiftRight,
            [Output::from_other(3), Output::from_flag_select(3)].into(),
        ),
        // flags
        (FlagDirect, Output::from_flag_select(0)),
        (FlagCarry, Output::from_flag_select(1)),
        (FlagEq, Output::from_flag_select(2)),
        (FlagZero, Output::from_flag_select(3)),
        (Flag6, Output::from_flag_select(4)),
        (Flag5, Output::from_flag_select(5)),
        (Flag7, Output::from_flag_select(6)),
        (Flag8, Output::from_flag_select(7)),
        (FlagWriteAlu, Output::from_other(1)),
    ])
});

mod step_template {
    use crate::Action;

    // custom max-capacity runtime-size implementation that fits in 8 bytes
    #[derive(Clone, Copy)]
    pub struct StepTemplate {
        arr: [Action; 7],
        size: u8,
    }

    impl StepTemplate {
        fn new() -> Self {
            Self {
                arr: [Action::Halt; 7],
                size: 0,
            }
        }
        fn push(&mut self, value: Action) {
            self.arr[self.size as usize] = value;
            self.size += 1;
        }

        fn from_arr<const N: usize>(arr: [Action; N]) -> Self {
            assert!(arr.len() <= 7);
            let mut result = Self::new();

            for val in arr {
                result.push(val);
            }

            result
        }

        pub fn iter<'a>(&'a self) -> Iter<'a> {
            Iter {
                iter: self.arr.iter(),
                size: self.size,
                index: 0,
            }
        }

        pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a> {
            IterMut {
                iter: self.arr.iter_mut(),
                size: self.size,
                index: 0,
            }
        }
    }

    impl<const N: usize> From<[Action; N]> for StepTemplate {
        fn from(arr: [Action; N]) -> Self {
            StepTemplate::from_arr(arr)
        }
    }

    pub struct IntoIter {
        istr_template: StepTemplate,
        index: u8,
    }

    pub struct Iter<'a> {
        iter: core::slice::Iter<'a, Action>,
        size: u8,
        index: u8,
    }

    pub struct IterMut<'a> {
        iter: core::slice::IterMut<'a, Action>,
        size: u8,
        index: u8,
    }

    impl Iterator for IntoIter {
        type Item = Action;

        fn next(&mut self) -> Option<Self::Item> {
            if self.index >= self.istr_template.size {
                None
            } else {
                self.index += 1;
                Some(self.istr_template.arr[(self.index - 1) as usize])
            }
        }
    }

    impl<'a> Iterator for Iter<'a> {
        type Item = &'a Action;

        fn next(&mut self) -> Option<Self::Item> {
            if self.index >= self.size {
                None
            } else {
                self.index += 1;
                self.iter.next()
            }
        }
    }

    impl<'a> Iterator for IterMut<'a> {
        type Item = &'a mut Action;

        fn next(&mut self) -> Option<Self::Item> {
            if self.index >= self.size {
                None
            } else {
                self.index += 1;
                self.iter.next()
            }
        }
    }

    impl IntoIterator for StepTemplate {
        type Item = Action;
        type IntoIter = IntoIter;

        fn into_iter(self) -> Self::IntoIter {
            IntoIter {
                istr_template: self,
                index: 0,
            }
        }
    }

    impl<'a> IntoIterator for &'a StepTemplate {
        type Item = &'a Action;
        type IntoIter = Iter<'a>;

        fn into_iter(self) -> Self::IntoIter {
            self.iter()
        }
    }

    impl<'a> IntoIterator for &'a mut StepTemplate {
        type Item = &'a mut Action;
        type IntoIter = IterMut<'a>;

        fn into_iter(self) -> Self::IntoIter {
            self.iter_mut()
        }
    }
}

use crate::step_template::StepTemplate;
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

struct NamedInstruction {
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

fn build_all_instructions() {
    let mut all_istrs: Vec<NamedInstruction> = Vec::new();
    move_word_reg_instructions(&mut all_istrs);
    move_word_imm_instructions(&mut all_istrs);

    for istr in all_istrs {
        println!("istr: {}", istr.name);
    }
}

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

fn action_to_output(action: Action) -> Output {
    Output::new()
}

fn step_template_to_output(step_istr: &StepTemplate) -> Output {
    let mut result = Output::new();

    let actions: Vec<_> = step_istr.into_iter().collect();
    let outputs: Vec<_> = step_istr
        .into_iter()
        .map(|&action| action_to_output(action))
        .collect();

    // loop through all unique pairs
    for i in 1..outputs.len() - 1 {
        for j in i + 1..outputs.len() {
            if outputs[i].intersect(&outputs[j]) {
                eprintln!(
                    "Failed when merging actions {:?} and {:?}",
                    *actions[i], *actions[j]
                );

                // TODO: print error?
                return result;
            }
        }
    }

    for output in &outputs {
        result.merge(output);
    }

    result
}

fn main() {
    println!("Hello, world!");
    build_all_instructions();
}
