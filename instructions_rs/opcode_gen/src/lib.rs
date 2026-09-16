use std::{cell::RefCell, rc::Rc};

use crate::{
    instructions::{Instruction, IstrSet, OpcodeToInstruction, OpcodeToOutput},
    opcode::Opcode,
    output::Output,
};

pub mod action;
pub mod instructions;
pub mod opcode;
pub mod output;
mod step_template;

fn pretty_print_opcode(opcode: Opcode, is_extended: bool) -> String {
    match opcode.step {
        0 => "Step 0, Ir: Unknown".to_string(),
        1 => {
            if is_extended {
                format!("Step 1, Ir: {}, Ir2: Unknown", opcode.ir)
            } else {
                format!("Step 1, Template Step 0, Ir: {}", opcode.ir)
            }
        }
        _ => {
            if is_extended {
                format!(
                    "Step {}, Template Step {}, Ir: {}, Ir2: {}",
                    opcode.step,
                    opcode.step - 2,
                    opcode.ir,
                    opcode.ir2
                )
            } else {
                format!(
                    "Step {}, Template Step {}, Ir: {}",
                    opcode.step,
                    opcode.step - 1,
                    opcode.ir
                )
            }
        }
    }
}

fn pretty_print_output(output: Output) -> String {
    let mut res = Vec::new();
    for (cat, byte) in output.get_printable_data() {
        if byte != 0 {
            res.push(format!("{cat}: {byte}"));
        }
    }
    res.join(", ")
}

pub fn diagnostic_from_addr(addr: u32, istr_set: &IstrSet) {
    let opcode = Opcode::from_addr(addr);

    let istr = istr_set.opcode_to_instruction(opcode);
    let output = istr_set.opcode_to_output(opcode);
    let raw_data = istr_set.opcode_to_output(opcode).get_output_data();

    let is_extended = istr
        .map(|e| e.borrow().opcode().as_ref().unwrap().ir2.is_some())
        .unwrap_or(false);

    println!("opcode: {}", pretty_print_opcode(opcode, is_extended));

    if let Some(inner) = istr {
        println!("istr: {:#?}", inner.borrow());
    } else {
        println!("No instruction found on this opcode");
    }

    println!("output: {}", pretty_print_output(output));
    println!("raw_data: {raw_data:#?}");
}

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
