use crate::action::Action::*;

use crate::instructions::defs::*;
use crate::opcode::Opcode;
use crate::output::Output;

// TODO: move out hardcoded length to somewhere?
pub struct Extended<I> {
    instructions: [Option<I>; 16],
    num_used_istrs: u8,
}

impl<I: std::fmt::Display> std::fmt::Display for Extended<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for istr in self.instructions.iter() {
            let name = istr.as_ref().map_or("Empty".to_string(), |e| e.to_string());
            write!(f, "{name} | ")?;
        }

        Ok(())
    }
}

impl<I> Extended<I> {
    pub fn new() -> Self {
        Self {
            instructions: [const { None }; 16],
            num_used_istrs: 0,
        }
    }

    pub fn is_full(&self) -> bool {
        self.num_used_istrs >= 16
    }

    pub fn get_istr(&self, idx: u8) -> &Option<I> {
        self.instructions.get(idx as usize).unwrap()
    }

    pub fn push(&mut self, istr: I) {
        if self.is_full() {
            panic!();
        }

        let available_position = self
            .instructions
            .iter_mut()
            .filter(|e| e.is_none())
            .take(1)
            .next()
            .unwrap();

        *available_position = Some(istr);
        self.num_used_istrs += 1;
    }
}

// TODO: determine what to do in empty case
impl<I: OpcodeToOutput> OpcodeToOutput for Extended<I> {
    fn to_output(&self, mut opcode: Opcode) -> Output {
        let step = opcode.step as usize;

        let extended_prelude = [*UNIVERSAL_STEP_0, *UNIVERSAL_STEP_1, *LOAD_IR2];

        if step < extended_prelude.len() {
            // universal steps should not be conflicting!
            return extended_prelude[step].to_output().unwrap();
        }

        opcode.step -= extended_prelude.len() as u8;

        if let Some(istr) = self.get_istr(opcode.ir2) {
            istr.to_output(opcode)
        } else {
            Halt.to_output()
        }
    }
}

pub struct Single<I> {
    instruction: I,
}

impl<I: std::fmt::Display> std::fmt::Display for Single<I> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.instruction)
    }
}

impl<I: OpcodeToOutput> OpcodeToOutput for Single<I> {
    fn to_output(&self, mut opcode: Opcode) -> Output {
        let step = opcode.step as usize;

        let single_prelude = [*UNIVERSAL_STEP_0];

        if step < single_prelude.len() {
            return single_prelude[step].to_output().unwrap();
        }

        opcode.step -= single_prelude.len() as u8;

        self.instruction.to_output(opcode)
    }
}

impl<I> Single<I> {
    pub fn new(istr: I) -> Self {
        Self { instruction: istr }
    }
}

pub struct VramInstructionTemplate {
    // vram is active on odd numbered instructions (0 indexed)
    pub active_even: NamedInstructionTemplate,
    // vram is active on even numbered instructions (0 indexed)
    pub active_odd: NamedInstructionTemplate,
    pub name: String,
}

impl OpcodeToOutput for VramInstructionTemplate {
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

pub struct NamedInstructionTemplate {
    pub istr: IstrTemplate,
    pub name: String,
}

// where the chain ends for simple instructions
impl OpcodeToOutput for NamedInstructionTemplate {
    fn to_output(&self, opcode: Opcode) -> Output {
        let step = opcode.step as usize;

        if self.istr.len() <= step {
            Halt.to_output()
        } else {
            match self.istr[step].to_output() {
                Ok(result) => result,
                Err(err) => {
                    eprintln!("Got error on instruction \"{}\": {}", self.name, err);
                    Halt.to_output()
                }
            }
        }
    }
}

pub enum InstructionImpl {
    Simple(NamedInstructionTemplate),
    Vram(VramInstructionTemplate),
}

impl std::fmt::Display for InstructionImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
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

pub enum InstructionEntry {
    Single(Box<Single<InstructionImpl>>),
    Extended(Box<Extended<InstructionImpl>>),
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
}

impl std::fmt::Display for IstrSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, istr) in self.istrs.iter().enumerate() {
            writeln!(f, "{i}: {istr}\n\n")?;
        }
        Ok(())
    }
}
