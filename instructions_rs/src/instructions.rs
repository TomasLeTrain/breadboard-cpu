mod register_defs;
use register_defs::*;

mod defs;
use defs::*;

use crate::action::Action::*;

mod instruction_defs;
pub use instruction_defs::build_all_instructions;

use crate::opcode::Opcode;
use crate::output::Output;

// TODO: move out hardcoded length to somewhere?
struct Extended<I> {
    instructions: [Option<I>; 16],
    num_used_istrs: u8,
}

impl<I> Extended<I> {
    fn new() -> Self {
        Self {
            instructions: [const { None }; 16],
            num_used_istrs: 0,
        }
    }

    fn is_full(&self) -> bool {
        self.num_used_istrs >= 16
    }

    fn get_istr(&self, idx: u8) -> &Option<I> {
        self.instructions.get(idx as usize).unwrap()
    }

    fn get_istr_mut(&mut self, idx: u8) -> &mut Option<I> {
        self.instructions.get_mut(idx as usize).unwrap()
    }
}

// TODO: determine what to do in empty case
impl<I: OpcodeToOutput> OpcodeToOutput for Extended<I> {
    fn to_output(&self, mut opcode: Opcode) -> Output {
        let step = opcode.step as usize;

        let extended_prelude = [*UNIVERSAL_STEP_0, *UNIVERSAL_STEP_1, *LOAD_IR2];

        if step < extended_prelude.len() {
            return extended_prelude[step].to_output();
        }

        opcode.step -= extended_prelude.len() as u8;

        if let Some(istr) = self.get_istr(opcode.ir2) {
            istr.to_output(opcode)
        } else {
            Halt.to_output()
        }
    }
}

struct Single<I> {
    instruction: I,
}

impl<I: OpcodeToOutput> OpcodeToOutput for Single<I> {
    fn to_output(&self, mut opcode: Opcode) -> Output {
        let step = opcode.step as usize;

        let single_prelude = [*UNIVERSAL_STEP_0];

        if step < single_prelude.len() {
            return single_prelude[step].to_output();
        }

        opcode.step -= single_prelude.len() as u8;

        self.instruction.to_output(opcode)
    }
}

impl<I> Single<I> {
    fn get_istr(&self) -> &I {
        &self.instruction
    }
    fn get_istr_mut(&mut self) -> &mut I {
        &mut self.instruction
    }

    fn new(istr: I) -> Self {
        Self { instruction: istr }
    }
}

pub struct VramInstruction {
    // vram is active on odd numbered instructions (0 indexed)
    active_even: SimpleInstruction,
    // vram is active on even numbered instructions (0 indexed)
    active_odd: SimpleInstruction,
    name: String,
}

impl OpcodeToOutput for VramInstruction {
    fn to_output(&self, opcode: Opcode) -> Output {
        let step = opcode.step as usize;
        let vram_active = opcode.not_vram_active as usize;

        if vram_active == step % 2 {
            // case active even:
            // 0: false
            // 1: true
            // 2: false
            &self.active_even
        } else {
            // case active odd:
            // 0: true
            // 1: false
            // 2: true
            &self.active_odd
        }
        .to_output(opcode)
    }
}

pub struct SimpleInstruction {
    pub istr: IstrTemplate,
    pub name: String,
}

// where the chain ends for simple instructions
impl OpcodeToOutput for SimpleInstruction {
    fn to_output(&self, opcode: Opcode) -> Output {
        let step = opcode.step as usize;

        if self.istr.len() < step {
            Halt.to_output()
        } else {
            self.istr[step].to_output()
        }
    }
}

pub enum InstructionImpl {
    Simple(SimpleInstruction),
    Vram(VramInstruction),
}

impl InstructionImpl {
    pub fn name(&self) -> &String {
        match self {
            InstructionImpl::Simple(simple_instruction) => &simple_instruction.name,
            InstructionImpl::Vram(vram_instruction) => &vram_instruction.name,
        }
    }
}

impl OpcodeToOutput for InstructionImpl {
    fn to_output(&self, opcode: Opcode) -> Output {
        match self {
            InstructionImpl::Simple(simple_instruction) => simple_instruction.to_output(opcode),
            InstructionImpl::Vram(vram_instruction) => vram_instruction.to_output(opcode),
        }
    }
}

enum InstructionEntry {
    Single(Box<Single<InstructionImpl>>),
    Extended(Box<Extended<InstructionImpl>>),
    Empty,
}

pub struct IstrSet {
    istrs: [InstructionEntry; 256],
}

pub trait OpcodeToOutput {
    fn to_output(&self, opcode: Opcode) -> Output;
}

// TODO: determine how to handle empty case
impl OpcodeToOutput for IstrSet {
    fn to_output(&self, opcode: Opcode) -> Output {
        match self.get_istr(opcode.ir) {
            InstructionEntry::Single(single) => single.to_output(opcode),
            InstructionEntry::Extended(extended) => extended.to_output(opcode),
            InstructionEntry::Empty => Halt.to_output(),
        }
    }
}

impl IstrSet {
    pub fn new() -> IstrSet {
        IstrSet {
            istrs: [const { InstructionEntry::Empty }; 256],
        }
    }

    pub fn get_istr(&self, idx: u8) -> &InstructionEntry {
        self.istrs.get(idx as usize).unwrap()
    }

    pub fn get_istr_mut(&mut self, idx: u8) -> &mut InstructionEntry {
        self.istrs.get_mut(idx as usize).unwrap()
    }

    pub fn is_empty(&self, idx: u8) -> bool {
        matches!(self.get_istr(idx), InstructionEntry::Empty)
    }

    pub fn is_extended(&self, idx: u8) -> bool {
        matches!(self.get_istr(idx), InstructionEntry::Extended(_))
    }

    pub fn is_single(&self, idx: u8) -> bool {
        matches!(self.get_istr(idx), InstructionEntry::Single(_))
    }
}
