mod register_defs;
use register_defs::*;

mod defs;
use defs::*;

use crate::action::Action::*;

mod instruction_defs;
pub use instruction_defs::build_all_instructions;

pub use defs::SimpleInstruction;

use crate::opcode::Opcode;
use crate::output::Output;

struct Extended<I> {
    instructions: [I; 16],
}

impl<I: OpcodeToOutput> OpcodeToOutput for Extended<I> {
    fn to_output(&self, mut opcode: Opcode) -> Output {
        let step = opcode.step as usize;

        let extended_prelude = [*UNIVERSAL_STEP_0, *UNIVERSAL_STEP_1, *LOAD_IR2];

        if step < extended_prelude.len() {
            return extended_prelude[step].to_output();
        }

        opcode.step -= extended_prelude.len() as u8;

        self.instructions[opcode.ir2 as usize].to_output(opcode)
    }
}

struct Single<I> {
    instruction: I,
}

impl<I: OpcodeToOutput> OpcodeToOutput for Single<I> {
    fn to_output(&self, mut opcode: Opcode) -> Output {
        let step = opcode.step as usize;

        let simple_prelude = [*UNIVERSAL_STEP_0];

        if step < simple_prelude.len() {
            return simple_prelude[step].to_output();
        }

        opcode.step -= simple_prelude.len() as u8;

        self.instruction.to_output(opcode)
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
    Single(Single<InstructionImpl>),
    Extended(Box<Extended<InstructionImpl>>),
}

pub struct IstrSet {
    istrs: [InstructionEntry; 256],
}

pub trait OpcodeToOutput {
    fn to_output(&self, opcode: Opcode) -> Output;
}

impl OpcodeToOutput for IstrSet {
    fn to_output(&self, opcode: Opcode) -> Output {
        match &self.istrs[opcode.ir as usize] {
            InstructionEntry::Single(single) => single.to_output(opcode),
            InstructionEntry::Extended(extended) => extended.to_output(opcode),
        }
    }
}
