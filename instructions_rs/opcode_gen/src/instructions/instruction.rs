/// public interface for interacting with instructions
use crate::{
    action::Action,
    instructions::{InstructionImpl, OpcodeToOutput, instruction_defs::InstructionType},
    opcode::{self, InstructionOpcode, Opcode},
    output::{self, Output},
};

pub enum Imm {
    Byte,
    Addr,
    None,
}

pub enum OverrideBehavior {
    A,
    B,
    Mar,
    Sp,
}

pub struct Instruction {
    istr_type: InstructionType,
    opcode: Option<InstructionOpcode>,
    imm: Imm,
    overrides: Vec<OverrideBehavior>,
    name: String,
    template: InstructionImpl,
}

impl Instruction {
    pub fn new(
        istr_type: InstructionType,
        imm: Imm,
        name: String,
        template: InstructionImpl,
    ) -> Self {
        Instruction {
            istr_type,
            name,
            imm,
            template,
            opcode: None,
            overrides: Vec::new(),
        }
    }

    fn istr_type(&self) -> &InstructionType {
        &self.istr_type
    }

    fn opcode(&self) -> &Option<InstructionOpcode> {
        &self.opcode
    }

    fn imm(&self) -> &Imm {
        &self.imm
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn template(&self) -> &InstructionImpl {
        &self.template
    }

    fn overrides(&self) -> &[OverrideBehavior] {
        &self.overrides
    }

    fn set_opcode(&mut self, opcode: Option<InstructionOpcode>) {
        self.opcode = opcode;
    }

    pub fn with_overrides(mut self, overrides: Vec<OverrideBehavior>) -> Self {
        self.overrides = overrides;
        self
    }
}

impl OpcodeToOutput for Instruction {
    fn to_output(&self, opcode: Opcode) -> Output {
        match self.template.to_output(opcode) {
            Ok(result) => result,
            Err(err) => {
                eprintln!("Got error on instruction \"{}\": {}", self.name, err);
                Action::Halt.to_output()
            }
        }
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
