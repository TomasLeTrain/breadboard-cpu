pub mod action;
pub mod instructions;
pub mod opcode;
mod output;
mod step_template;

fn get_instruction_list(){
    let istr_set = instructions::build_all_instructions();
}
