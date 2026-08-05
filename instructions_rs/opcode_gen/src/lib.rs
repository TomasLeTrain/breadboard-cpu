use std::{cell::RefCell, rc::Rc};

use crate::instructions::{Instruction, IstrSet};

pub mod action;
pub mod instructions;
pub mod opcode;
mod output;
mod step_template;

/// vec full of all instructions as well as IstrSet struct that allows lookup of instructions from opcodes
pub fn get_instruction_set() -> (Vec<Rc<RefCell<Instruction>>>, IstrSet) {
    instructions::build_all_instructions()
}

/// returns vec of all instructions existant in the instruction set
pub fn get_instruction_list() -> Vec<Instruction> {
    let (all_istrs, istr_set) = instructions::build_all_instructions();

    // drop istr_set to remove all its references
    drop(istr_set);

    // here the only references that should exist should be the all_istrs vec, so should be safe to
    // unwrap all values

    let all_istrs: Vec<Instruction> = all_istrs
        .into_iter()
        .map(|e| {
            let inner = Rc::try_unwrap(e).unwrap();
            inner.into_inner()
        })
        .collect();

    all_istrs
}
