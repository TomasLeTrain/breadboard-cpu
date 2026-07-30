mod action;
mod instructions;
mod opcode;
mod output;
mod step_template;

use action::Action;

use crate::instructions::InstructionImpl;

fn print_istrs(istrs: &Vec<InstructionImpl>) {
    for istr in istrs {
        println!("istr: {}", istr.name());
        // istr.to_output(opcode)
        // istrfirst().unwrap().to_output();
    }
}

fn main() {
    println!("Hello, world!");
    let istrs = instructions::build_all_instructions();
    print_istrs(&istrs);
    println!("total num instructions: {}", istrs.len());

    // let num_single = istrs
    //     .iter()
    //     .filter(|e| matches!(e.istr_type, InstructionType::Single))
    //     .count();
    //
    // let num_extended = istrs
    //     .iter()
    //     .filter(|e| matches!(e.istr_type, InstructionType::Extended))
    //     .count();

    // println!("total num single istrs: {}", num_single);
    // println!("total num extended istrs: {}", num_extended);
}
