mod defs;
mod instruction;
mod instruction_defs;
mod istr_set;
mod istr_utils;
mod register_defs;

use std::{cell::RefCell, rc::Rc, vec::Vec};

pub use defs::{AddressRegister, Register};
pub use instruction::Instruction;
pub use instruction_defs::{ArgumentType, ArgumentValue, InstructionSignature, InstructionType};
pub use istr_set::IstrSet;
pub use istr_utils::{InstructionImpl, OpcodeToInstruction, OpcodeToOutput};

// import all the instruction funcs
use instruction_defs::*;

/// Generates list of instructions and also allocates all opcodes
pub fn build_all_instructions() -> (Vec<Rc<RefCell<Instruction>>>, IstrSet) {
    let mut istr_set = IstrSet::new();
    let mut all_istrs = Vec::new();

    // place some special instructions for specific spots before any others
    for istr in [error_instruction(), halt_instruction(), nop_instruction()] {
        let idx = match istr.istr_type() {
            InstructionType::Error => 0,
            InstructionType::Nop => 254,
            InstructionType::Halt => 255,
            _ => unreachable!(),
        };

        let istr = Rc::new(RefCell::new(istr));
        all_istrs.push(Rc::clone(&istr));

        let istr_opcode = istr_set.place_simple_idx(Rc::clone(&istr), idx).unwrap();

        istr.borrow_mut().set_opcode(Some(istr_opcode));
    }

    // math types can only exist in specific ranges, need to place those first
    let all_math_istrs_iter = math_reg_instructions()
        .into_iter()
        .chain(math_imm_instructions())
        .chain(not_instructions())
        .chain(not_reg_instructions());

    for (istr, math_type) in all_math_istrs_iter {
        let math_given_bits = math_type.as_ir_bits();

        let math_range = math_given_bits << 4;
        let math_end_range = ((math_given_bits + 1) << 4) - 1;
        let first_range = (math_range, math_end_range);
        let second_range = (math_range | 1 << 7, math_end_range | 1 << 7);

        let istr = Rc::new(RefCell::new(istr));

        all_istrs.push(Rc::clone(&istr));

        let istr_opcode = istr_set
            .place_extended_ranges(Rc::clone(&istr), &[first_range, second_range])
            .unwrap();

        istr.borrow_mut().set_opcode(Some(istr_opcode));
    }

    // has approx 120-ish instructions, need to make extended
    for istr in move_word_reg_instructions().into_iter() {
        let istr = Rc::new(RefCell::new(istr));

        all_istrs.push(Rc::clone(&istr));
        let istr_opcode = istr_set.place_extended(Rc::clone(&istr)).unwrap();
        istr.borrow_mut().set_opcode(Some(istr_opcode));
    }

    // set of functions that all abstract over addr_reg. SP variants are placed as extended to save
    // in simple spaces
    let addr_reg_iters = lw_template_addr_reg_instructions()
        .into_iter()
        .chain(lw_template_imm16_instructions())
        .chain(sw_instructions())
        .chain(jnz_reg_instructions())
        .chain(jmp_instructions());

    for (istr, addr_reg) in addr_reg_iters {
        let istr = Rc::new(RefCell::new(istr));

        all_istrs.push(Rc::clone(&istr));

        // sp variant less common, place on extended to save simple slots
        let istr_opcode = match addr_reg {
            defs::AddressRegister::Mar => istr_set.place_simple(Rc::clone(&istr)).unwrap(),
            defs::AddressRegister::Sp => istr_set.place_extended(Rc::clone(&istr)).unwrap(),
        };

        istr.borrow_mut().set_opcode(Some(istr_opcode));
    }

    // from here on there's enough space for all simple
    let simple_istrs = move_word_imm_instructions()
        .into_iter()
        .chain(push_reg_instructions())
        .chain(push_addr_reg_instructions())
        .chain(push_imm8_instructions())
        .chain(pop_reg_instructions())
        .chain(pop_addr_reg_instructions())
        .chain(lda_imm16_instructions())
        .chain(mv_addr_reg_instructions())
        .chain(misc_instructions())
        .chain(vram_read_instructions())
        .chain(vram_write_instructions())
        .chain(shift_instructions());

    for istr in simple_istrs {
        let istr = Rc::new(RefCell::new(istr));
        all_istrs.push(Rc::clone(&istr));
        let istr_opcode = istr_set.place_simple(Rc::clone(&istr)).unwrap();
        istr.borrow_mut().set_opcode(Some(istr_opcode));
    }

    (all_istrs, istr_set)
}
