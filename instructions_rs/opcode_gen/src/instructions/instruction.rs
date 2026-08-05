/// public interface for interacting with instructions
use crate::{
    instructions::{InstructionImpl, OpcodeToOutput, instruction_defs::InstructionType},
    opcode::InstructionOpcode,
};

enum Imm {
    Imm8,
    ImmAddr,
    None,
}

pub struct Instruction {
    istr_type: InstructionType,
    opcode: InstructionOpcode,
    imm: Imm,
    name: String,
    template: InstructionImpl,
}

impl Instruction {
    fn istr_type(&self) -> &InstructionType {
        &self.istr_type
    }

    fn opcode(&self) -> &InstructionOpcode {
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
}

impl OpcodeToOutput for Instruction {
    fn to_output(&self, opcode: crate::opcode::Opcode) -> crate::output::Output {
        self.template.to_output(opcode)
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
