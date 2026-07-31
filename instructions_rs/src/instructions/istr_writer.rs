use crate::instructions::{
    instruction_defs::*,
    istr_utils::{Extended, InstructionEntry, IstrSet, Single},
    *,
};

use std::option::Option;

// responsible for placing instructions in opcodes based on constraints
struct InstructionWriter<'a> {
    istr_set: &'a mut IstrSet,
}

// all functions are greedy, meaning they allocated the first spots they can given their constraints
// this means that the caller must use the functions in order of importance (for example if certain
// instructions require a specific order place those first)
//
// caller also does not need to worry about the allocation of extended or simple instructions, only
// in the case the constraints cannot be satisfied (which should crash the program)
impl<'a> InstructionWriter<'a> {
    fn new(istr_set: &mut IstrSet) -> InstructionWriter<'_> {
        InstructionWriter { istr_set }
    }

    fn is_empty(&self, idx: u8) -> bool {
        self.istr_set.is_empty(idx)
    }

    fn simple_available(&self, idx: u8) -> bool {
        self.is_empty(idx)
    }

    // allocates an extended instruction at the specified ir idx
    fn allocate_extended_idx(&mut self, idx: u8) {
        assert!(self.is_empty(idx));

        *self.istr_set.get_istr_mut(idx) = InstructionEntry::Extended(Box::new(Extended::new()));
    }

    // returns true if idx is free OR idx is an extended istr and there are spots available
    fn extended_available(&self, idx: u8) -> bool {
        match self.istr_set.get_istr(idx) {
            InstructionEntry::Single(_) => false,
            InstructionEntry::Extended(extended) => !extended.is_full(),
            InstructionEntry::Empty => true,
        }
    }

    // attempts to place extended at specified ir idx in first spot in the extended instruction
    // returns true if operation succeeded
    fn place_extended_idx(&mut self, istr: InstructionImpl, idx: u8) -> Option<()> {
        // not allocated or no spaces available here
        if !self.extended_available(idx) {
            return None;
        }

        // allocate first if needed
        if self.is_empty(idx) {
            self.allocate_extended_idx(idx);
        }

        if let InstructionEntry::Extended(extended) = self.istr_set.get_istr_mut(idx) {
            extended.push(istr);
            Some(())
        } else {
            unreachable!()
        }
    }

    // attempts to place simple at specified ir idx
    fn place_simple_idx(&mut self, istr: InstructionImpl, idx: u8) -> Option<()> {
        if !self.simple_available(idx) {
            return None;
        }
        *self.istr_set.get_istr_mut(idx) = InstructionEntry::Single(Box::new(Single::new(istr)));
        Some(())
    }

    // places given instructions in specified ranges of IR, if possible
    // all instructions are extended
    //
    // removes all instructions placed from the given vector in a front to back order.
    fn place_extended_ranges(&mut self, istrs: InstructionImpl, ranges: &[(u8, u8)]) {
        for &(start, end) in ranges.iter() {
            for idx in start..=end {
                if self.extended_available(idx) {
                    let res = self.place_extended_idx(istrs, idx);
                    assert!(res.is_some());
                    return;
                }
            }
        }
    }

    // places simple instruction in first available slot
    // if none available returns ?
    fn place_simple(&mut self, istr: InstructionImpl) -> Option<()> {
        // TODO: optimize by saving smallest valid pointers
        // TODO: move hardcoded values elsewhere
        for idx in 0..=255 {
            if self.simple_available(idx) {
                return self.place_simple_idx(istr, idx);
            }
        }
        None
    }

    // places extended in first available slot
    // returns none if no spots available
    fn place_extended(&mut self, istr: InstructionImpl) -> Option<()> {
        // TODO: optimize by saving smallest valid pointers
        // TODO: move hardcoded values elsewhere
        for idx in 0..=255 {
            if self.extended_available(idx) {
                let res = self.place_extended_idx(istr, idx);

                assert!(res.is_some());

                // found a spot to place the istr
                return Some(());
            }
        }
        None
    }
}

pub fn build_all_instructions() -> IstrSet {
    let mut istr_set = IstrSet::new();
    let mut writer = InstructionWriter::new(&mut istr_set);

    // math types can only exist in specific ranges, need to place those first
    let all_math_istrs_iter = math_reg_instructions()
        .into_iter()
        .chain(math_imm_instructions())
        .chain(not_instructions())
        .chain(not_reg_instructions());

    for (istr, math_type) in all_math_istrs_iter {
        let math_given_bits = math_type.to_ir_bits();

        let math_range = (math_given_bits << 4) as u8;
        let math_end_range = (((math_given_bits + 1) << 4) - 1) as u8;
        let first_range = (math_range, math_end_range);
        let second_range = (math_range | 1 << 7, math_end_range | 1 << 7);

        writer.place_extended_ranges(istr, &[first_range, second_range]);
    }

    // has approx 120-ish instructions, need to make extended
    for istr in move_word_reg_instructions().into_iter() {
        writer.place_extended(istr).unwrap();
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
        // sp variant less common, place on extended to save simple slots
        match addr_reg {
            defs::AddressRegisterEnum::Mar => writer.place_simple(istr).unwrap(),
            defs::AddressRegisterEnum::Sp => writer.place_extended(istr).unwrap(),
        }
    }

    // from here on there's enough space for all simple
    let simple_istrs = move_word_imm_instructions()
        .into_iter()
        .chain(push_reg_instructions())
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
        writer.place_simple(istr).unwrap();
    }

    istr_set
}
