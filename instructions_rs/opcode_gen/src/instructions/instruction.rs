/// public interface for interacting with instructions
use crate::{
    action::Action,
    instructions::{
        InstructionImpl,
        instruction_defs::{ArgumentValue, InstructionType},
    },
    opcode::{InstructionOpcode, Opcode},
    output::Output,
};

#[derive(Debug)]
pub enum Imm {
    Byte,
    Addr,
    None,
}

/// Possible registers that can get overwritten by instructions
#[derive(Debug)]
pub enum OverrideBehavior {
    A,
    B,
    Mar,
    Sp,
    Flag,
}

#[derive(Debug)]
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

    pub fn istr_type(&self) -> &InstructionType {
        &self.istr_type
    }

    pub fn opcode(&self) -> &Option<InstructionOpcode> {
        &self.opcode
    }

    pub fn imm(&self) -> &Imm {
        &self.imm
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn template(&self) -> &InstructionImpl {
        &self.template
    }

    pub fn overrides(&self) -> &[OverrideBehavior] {
        &self.overrides
    }

    pub fn set_opcode(&mut self, opcode: Option<InstructionOpcode>) {
        self.opcode = opcode;
    }

    pub fn with_overrides(mut self, overrides: Vec<OverrideBehavior>) -> Self {
        self.overrides = overrides;
        self
    }

    pub fn get_byte_size(&self) -> usize {
        // TODO: wrap in result in case opcode is not set
        self.opcode().as_ref().unwrap().byte_size() + self.istr_type.get_imm_byte_size()
    }

    // TODO: wrap in result for various possible error cases
    pub fn get_asm_bytes(&self, arg_values: Vec<ArgumentValue>) -> Vec<u8> {
        let mut res = Vec::new();

        res.append(&mut self.opcode().as_ref().unwrap().get_opcode_bytes());
        res.append(&mut self.istr_type.get_imm_bytes(arg_values));

        res
    }
}

impl Instruction {
    // at this point the step from opcode should be modified to include the prelude
    // trait not implemented to make this more clear
    pub fn template_to_output(&self, opcode: Opcode) -> Output {
        self.template.to_output(opcode).unwrap_or_else(|err| {
            eprintln!("Got error on instruction \"{}\": {}", self.name, err);
            Action::Halt.to_output()
        })
    }
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
