mod defs;
mod instruction_defs;
mod istr_utils;
mod istr_writer;
mod register_defs;

pub use istr_utils::{InstructionImpl, OpcodeToOutput};
pub use istr_writer::build_all_instructions;
