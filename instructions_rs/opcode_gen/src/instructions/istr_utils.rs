use std::cell::RefCell;
use std::rc::Rc;

/// Structures used to represent how instructions are placed in opcodes
use crate::action::Action::*;

use crate::instructions::defs::*;
use crate::instructions::instruction::Instruction;
use crate::opcode::Opcode;
use crate::output::Output;
use crate::step_template::MergingActionsError;

/// Trait defining generating an output for some opcode
pub trait OpcodeToOutput {
    fn opcode_to_output(&self, opcode: Opcode) -> Output;
}

// TODO: could use for opcode query mode
pub trait OpcodeToInstruction {
    fn opcode_to_instruction(&self, opcode: Opcode) -> Option<&Rc<RefCell<Instruction>>>;
}

// TODO: move out hardcoded length to somewhere?
pub struct Extended {
    instructions: [Option<Rc<RefCell<Instruction>>>; 16],
    num_used_istrs: u8,
}

impl std::fmt::Display for Extended {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for istr in self.instructions.iter() {
            let name = istr
                .as_ref()
                .map_or("Empty".to_string(), |e| e.borrow().to_string());
            write!(f, "{name} | ")?;
        }

        Ok(())
    }
}

impl Extended {
    pub fn new() -> Self {
        Self {
            instructions: [const { None }; 16],
            num_used_istrs: 0,
        }
    }

    pub fn is_full(&self) -> bool {
        self.num_used_istrs >= 16
    }

    pub fn get_istr(&self, idx: u8) -> Option<&Rc<RefCell<Instruction>>> {
        self.instructions.get(idx as usize).unwrap().as_ref()
    }

    /// returns extended_idx at which instruction was placed
    pub fn push(&mut self, istr: Rc<RefCell<Instruction>>) -> u8 {
        if self.is_full() {
            panic!();
        }

        let (extended_idx, available_position) = self
            .instructions
            .iter_mut()
            .enumerate()
            .filter(|(_i, e)| e.is_none())
            .take(1)
            .next()
            .unwrap();

        *available_position = Some(istr);
        self.num_used_istrs += 1;

        extended_idx as u8
    }
}

// TODO: determine what to do in empty case
impl OpcodeToOutput for Extended {
    fn opcode_to_output(&self, mut opcode: Opcode) -> Output {
        let step = opcode.step as usize;

        let extended_prelude = [*UNIVERSAL_STEP_0, *UNIVERSAL_STEP_1, *LOAD_IR2];

        if step < extended_prelude.len() {
            // universal steps should not be conflicting!
            return extended_prelude[step].to_output().unwrap();
        }

        opcode.step -= extended_prelude.len() as u8;

        if let Some(istr) = self.get_istr(opcode.ir2) {
            istr.borrow().template_to_output(opcode)
        } else {
            Halt.to_output()
        }
    }
}

impl OpcodeToInstruction for Extended {
    fn opcode_to_instruction(&self, opcode: Opcode) -> Option<&Rc<RefCell<Instruction>>> {
        self.get_istr(opcode.ir2)
    }
}

pub struct Single {
    instruction: Rc<RefCell<Instruction>>,
}

impl Single {
    pub fn new(istr: Rc<RefCell<Instruction>>) -> Self {
        Self { instruction: istr }
    }
}

impl std::fmt::Display for Single {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.instruction.borrow())
    }
}

impl OpcodeToOutput for Single {
    fn opcode_to_output(&self, mut opcode: Opcode) -> Output {
        let step = opcode.step as usize;

        let single_prelude = [*UNIVERSAL_STEP_0];

        if step < single_prelude.len() {
            return single_prelude[step].to_output().unwrap();
        }

        opcode.step -= single_prelude.len() as u8;

        self.instruction.borrow().template_to_output(opcode)
    }
}

impl OpcodeToInstruction for Single {
    fn opcode_to_instruction(&self, _opcode: Opcode) -> Option<&Rc<RefCell<Instruction>>> {
        Some(&self.instruction)
    }
}

#[derive(Debug)]
pub struct VramInstructionTemplate {
    // vram is active on odd numbered instructions (0 indexed)
    pub active_even: InstructionTemplate,
    // vram is active on even numbered instructions (0 indexed)
    pub active_odd: InstructionTemplate,
}

impl VramInstructionTemplate {
    fn to_output(&self, opcode: Opcode) -> Result<Output, MergingActionsError> {
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

#[derive(Debug)]
pub struct InstructionTemplate(pub IstrTemplateVec);

// where the chain ends for simple instructions
impl InstructionTemplate {
    fn to_output(&self, opcode: Opcode) -> Result<Output, MergingActionsError> {
        let step = opcode.step as usize;

        if self.0.len() <= step {
            Ok(Halt.to_output())
        } else {
            self.0[step].to_output()
        }
    }
}

#[derive(Debug)]
pub enum InstructionImpl {
    Simple(InstructionTemplate),
    Vram(VramInstructionTemplate),
}

impl InstructionImpl {
    pub fn to_output(&self, opcode: Opcode) -> Result<Output, MergingActionsError> {
        match self {
            InstructionImpl::Simple(simple_instruction) => simple_instruction.to_output(opcode),
            InstructionImpl::Vram(vram_instruction) => vram_instruction.to_output(opcode),
        }
    }
}

pub enum InstructionEntry {
    Single(Single),
    Extended(Box<Extended>),
    Empty,
}

impl std::fmt::Display for InstructionEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            InstructionEntry::Single(single) => single.to_string(),
            InstructionEntry::Extended(extended) => extended.to_string(),
            InstructionEntry::Empty => "Empty".to_string(),
        };
        write!(f, "{name}")
    }
}
