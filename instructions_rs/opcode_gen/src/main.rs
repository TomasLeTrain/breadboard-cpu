mod action;
mod instructions;
mod opcode;
mod output;
mod step_template;

use crate::instructions::OpcodeToOutput;

fn main() {
    let istr_set = instructions::build_all_instructions();
    println!("{istr_set}");

    for i in 0..(1 << 17) {
        let opcode = opcode::addr_to_opcode(i);
        istr_set.to_output(opcode);
    }
}
