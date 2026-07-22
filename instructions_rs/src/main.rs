use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
enum Action {
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
        for curr in arr {
            result.merge(&curr)
        }
        result
    }
}

fn init_action_output_map() -> HashMap<Action, Output> {
    HashMap::from([
        (Action::Halt, Output::from_bout(5)),
        (Action::Nop, Output::new()),
        (Action::Reset, Output::from_other(2)),
        // addr regs cnt
        (Action::PcCnt, Output::from_pc_cnt(1)),
        (Action::MarCnt, Output::from_bout(4)),
        (Action::SpDec, Output::from_bout(7)),
        (Action::SpInc, Output::from_bout(6)),
        // addr regs addr
        (Action::PcAddr, Output::from_addr(1)),
        (Action::MarAddr, Output::from_addr(2)),
        (Action::SpAddr, Output::from_addr(3)),
        // registers bout
        (Action::ABout, Output::from_bout(0b1000)),
        (Action::BBout, Output::from_bout(0b1000 | 1)),
        (Action::XBout, Output::from_bout(0b1000 | 5)),
        (Action::YBout, Output::from_bout(0b1000 | 6)),
        (Action::ZBout, Output::from_bout(0b1000 | 7)),
        (
            Action::PcLoBout,
            [Output::from_addr(1), Output::from_bout(2)].into(),
        ),
        (
            Action::PcHiBout,
            [Output::from_addr(1), Output::from_bout(3)].into(),
        ),
        (
            Action::MarLoBout,
            [Output::from_addr(2), Output::from_bout(2)].into(),
        ),
        (
            Action::MarHiBout,
            [Output::from_addr(2), Output::from_bout(3)].into(),
        ),
        (
            Action::SpLoBout,
            [Output::from_addr(3), Output::from_bout(2)].into(),
        ),
        (
            Action::SpHiBout,
            [Output::from_addr(3), Output::from_bout(3)].into(),
        ),
        (Action::KeybBout, Output::from_bout(0b1000 | 2)),
        (Action::FlagsBout, Output::from_bout(0b1000 | 4)),
        (Action::FAluBout, Output::from_bout(0b1000 | 3)),
        // registers write
        (Action::AWrite, Output::from_write(0b1000)),
        (Action::BWrite, Output::from_write(0b1000 | 1)),
        (Action::XWrite, Output::from_write(0b1000 | 5)),
        (Action::YWrite, Output::from_write(0b1000 | 6)),
        (Action::ZWrite, Output::from_write(7)),
        (Action::PcLoWrite, Output::from_write(0b1000 | 3)),
        (Action::PcHiWrite, Output::from_write(0b1000 | 2)),
        (Action::MarLoWrite, Output::from_write(5)),
        (Action::MarHiWrite, Output::from_write(4)),
        (Action::SpLoWrite, Output::from_write(3)),
        (Action::SpHiWrite, Output::from_write(2)),
        // ir regs
        (Action::IrWrite, Output::from_write(0b1000 | 7)),
        (Action::Ir2Write, Output::from_write(1)),
        // mem read/write
        (Action::MemRead, Output::from_bout(1)),
        (Action::MemWrite, Output::from_write(6)),
        // vram read/write
        (
            Action::VramRead,
            [Output::from_other(3), Output::from_flag_select(5)].into(),
        ),
        (
            Action::VramWrite,
            [Output::from_other(3), Output::from_flag_select(4)].into(),
        ),
        // shift left
        (
            Action::XShiftLeft,
            [Output::from_other(3), Output::from_flag_select(0)].into(),
        ),
        (
            Action::YShiftLeft,
            [Output::from_other(3), Output::from_flag_select(2)].into(),
        ),
        // shift right
        (
            Action::XShiftRight,
            [Output::from_other(3), Output::from_flag_select(1)].into(),
        ),
        (
            Action::YShiftRight,
            [Output::from_other(3), Output::from_flag_select(3)].into(),
        ),
        // flags
        (Action::FlagDirect, Output::from_flag_select(0)),
        (Action::FlagCarry, Output::from_flag_select(1)),
        (Action::FlagEq, Output::from_flag_select(2)),
        (Action::FlagZero, Output::from_flag_select(3)),
        (Action::Flag6, Output::from_flag_select(4)),
        (Action::Flag5, Output::from_flag_select(5)),
        (Action::Flag7, Output::from_flag_select(6)),
        (Action::Flag8, Output::from_flag_select(7)),
        (Action::FlagWriteAlu, Output::from_other(1)),
    ])
}

// custom max-capacity runtime-size implementation that fits in 8 bytes
struct StepTemplate {
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
}

impl<const N: usize> From<[Action; N]> for StepTemplate {
    fn from(arr: [Action; N]) -> Self {
        StepTemplate::from_arr(arr)
    }
}

struct StepTemplateIterator<'a> {
    istr_template: &'a StepTemplate,
    index: u8,
}

impl<'a> Iterator for StepTemplateIterator<'a> {
    // we will be counting with usize
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

impl<'a> IntoIterator for &'a StepTemplate {
    type Item = Action;
    type IntoIter = StepTemplateIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        StepTemplateIterator {
            istr_template: self,
            index: 0,
        }
    }
}

type IstrTemplate = Vec<StepTemplate>;

fn universal_step_0() -> StepTemplate {
    [Action::MemRead, Action::PcAddr, Action::IrWrite].into()
}

fn load_address_procedure() -> IstrTemplate {
    vec![
        universal_step_0(),
        [Action::PcCnt].into(),
        [Action::PcCnt, Action::PcAddr, Action::MarHiWrite].into(), // first byte has msb
        [Action::PcCnt].into(),                                     // pc cnt
        [Action::MemRead, Action::PcAddr, Action::MarLoWrite].into(), // second byte has lsb
    ]
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

struct istr {}

fn actionToOutput(action: &Action) -> Output {
    Output::new()
}

fn step_template_to_output(step_istr: &StepTemplate) -> Output {
    let mut output = Output::new();

    for action_i in step_istr {
        let output_i = actionToOutput(&action_i);
        for action_j in step_istr {
            let output_j = actionToOutput(&action_j);
            if output_i.intersect(&output_j) {
                // TODO: output error
                return output;
            }
        }
    }

    // perform merging logic
    for action in step_istr {
        output.merge(&actionToOutput(&action));
    }

    output
}

fn main() {
    println!("Hello, world!");
}
