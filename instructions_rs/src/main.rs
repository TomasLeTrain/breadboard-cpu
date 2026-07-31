mod action;
mod instructions;
mod opcode;
mod output;
mod step_template;

use action::Action;

use crate::instructions::InstructionImpl;

fn main() {
    let istr_set = instructions::build_all_instructions();
    println!("{istr_set}");
}
