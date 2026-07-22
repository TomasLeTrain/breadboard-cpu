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

struct Output {
    bout: u8,
    write: u8,
    addr: u8,
    misc: u8,
    flag_select: u8,
    pc_cnt: u8,
}

impl Output {
    fn create_empty() -> Self {
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
        let mut result = Self::create_empty();
        result.write = val;
        result
    }

    fn from_bout(val: u8) -> Self {
        let mut result = Self::create_empty();
        result.bout = val;
        result
    }

    fn from_addr(val: u8) -> Self {
        let mut result = Self::create_empty();
        result.addr = val;
        result
    }

    fn from_other(val: u8) -> Self {
        let mut result = Self::create_empty();
        result.misc = val;
        result
    }

    fn from_flag_select(val: u8) -> Self {
        let mut result = Self::create_empty();
        result.flag_select = val;
        result
    }

    fn from_pc_cnt(val: u8) -> Self {
        let mut result = Self::create_empty();
        result.pc_cnt = val;
        result
    }

    fn from(arr: &[Self]) -> Output {
        let mut result = Self::create_empty();
        for curr in arr {
            result.merge(curr)
        }
        result
    }

    fn intersect(&self, other: &Self) -> bool {
        if (self.bout > 0) && (other.bout > 0) {
            true
        } else if (self.write > 0) && (other.write > 0) {
            true
        } else if (self.addr > 0) && (other.addr > 0) {
            true
        } else if (self.flag_select > 0) && (other.flag_select > 0) {
            true
        } else if (self.pc_cnt > 0) && (other.pc_cnt > 0) {
            true
        } else {
            false
        }
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

fn init_action_output_map() -> HashMap<Action, Output> {
    let map = HashMap::from([
        (Action::Halt, Output::from_bout(5)),
        (Action::Nop, Output::create_empty()),
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
        (Action::ABout, Output::from_bout(0b1000 | 0)),
        (Action::BBout, Output::from_bout(0b1000 | 1)),
        (Action::XBout, Output::from_bout(0b1000 | 5)),
        (Action::YBout, Output::from_bout(0b1000 | 6)),
        (Action::ZBout, Output::from_bout(0b1000 | 7)),
        (
            Action::PcLoBout,
            Output::from(&[Output::from_addr(1), Output::from_bout(2)]),
        ),
        (
            Action::PcHiBout,
            Output::from(&[Output::from_addr(1), Output::from_bout(3)]),
        ),
        (
            Action::MarLoBout,
            Output::from(&[Output::from_addr(2), Output::from_bout(2)]),
        ),
        (
            Action::MarHiBout,
            Output::from(&[Output::from_addr(2), Output::from_bout(3)]),
        ),
        (
            Action::SpLoBout,
            Output::from(&[Output::from_addr(3), Output::from_bout(2)]),
        ),
        (
            Action::SpHiBout,
            Output::from(&[Output::from_addr(3), Output::from_bout(3)]),
        ),
        (Action::KeybBout, Output::from_bout(0b1000 | 2)),
        (Action::FlagsBout, Output::from_bout(0b1000 | 4)),
        (Action::FAluBout, Output::from_bout(0b1000 | 3)),
        // registers write
        (Action::AWrite, Output::from_write(0b1000 | 0)),
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
            Output::from(&[Output::from_other(3), Output::from_flag_select(5)]),
        ),
        (
            Action::VramWrite,
            Output::from(&[Output::from_other(3), Output::from_flag_select(4)]),
        ),
        // shift left
        (
            Action::XShiftLeft,
            Output::from(&[Output::from_other(3), Output::from_flag_select(0)]),
        ),
        (
            Action::YShiftLeft,
            Output::from(&[Output::from_other(3), Output::from_flag_select(2)]),
        ),
        // shift right
        (
            Action::XShiftRight,
            Output::from(&[Output::from_other(3), Output::from_flag_select(1)]),
        ),
        (
            Action::YShiftRight,
            Output::from(&[Output::from_other(3), Output::from_flag_select(3)]),
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
    ]);
    map
}

// custom max-capacity runtime-size implementation that fits in 8 bytes
struct IstrTemplate {
    arr: [Action; 7],
    size: u8,
}

impl IstrTemplate {
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

    fn from(arr: &[Action]) -> Self {
        assert!(arr.len() <= 7);
        let mut result = Self::new();

        for val in arr {
            result.push(*val);
        }
        result
    }
}

// impl IntoIterator for IstrTemplate {
//     type Item = Action;
//     type IntoIter = IstrTemplateIterator;
//
//     fn into_iter(self) -> Self::IntoIter {
//         IstrTemplateIterator {
//             istr_template: self,
//             index: 0,
//         }
//     }
// }

struct IstrTemplateIterator<'a> {
    istr_template: &'a IstrTemplate,
    index: u8,
}

impl<'a> Iterator for IstrTemplateIterator<'a> {
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

impl<'a> IntoIterator for &'a IstrTemplate {
    type Item = Action;
    type IntoIter = IstrTemplateIterator<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IstrTemplateIterator {
            istr_template: self,
            index: 0,
        }
    }
}

fn test() {
    // let thing: IstrTemplate = vec![vec![]];
}

// IstrTemplateType load_address_procedure = {
//     universal_step_0,
//     {pc::cnt},
//     {mem::read, pc::addr, mar::hi::write}, // first byte has msb
//     {pc::cnt},                             // pc cnt
//     {mem::read, pc::addr, mar::lo::write}, // second byte has lsb
// };
//

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

// struct StepTemplate {}
struct istr {}
fn actionToOutput(action: &Action) -> Output {
    Output::create_empty()
}

// type StepTemplate = Vec<Action>;

fn step_template_to_output(step_istr: &IstrTemplate) -> Output {
    let mut output = Output::create_empty();

    for action_i in step_istr {
        let output_i = actionToOutput(&action_i);
        for action_j in *step_istr {
            let output_j = actionToOutput(&action_j);
            if output_i.intersect(&output_j) {
                // TODO: output error
                return output;
            }
        }
    }

    // perform merging logic
    for action in *step_istr {
        output.merge(&actionToOutput(&action));
    }

    output
}

fn main() {
    println!("Hello, world!");
}
