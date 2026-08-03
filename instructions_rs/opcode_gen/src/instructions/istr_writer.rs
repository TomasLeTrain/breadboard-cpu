//! Utils for placing instructions in an instruction set while following given constraints.

use crate::instructions::InstructionImpl;
use crate::instructions::istr_utils::{Extended, InstructionEntry, IstrSet, Single};

use std::error::Error;
use std::fmt;

/// Responsible for placing instructions in opcodes while following constraints
///
/// * `istr_set`: Instruction set to modify
/// * `simple_istr_ptr`: index at which its guaranteed all instructions before it are filled. gets used to avoid unnecessary looping through already used indexes.
/// * `extended_istr_ptr`: index at which its guaranteed all instructions before it are filled including all extended instruction slots.
pub struct InstructionWriter<'a> {
    istr_set: &'a mut IstrSet,
    simple_istr_ptr: u16,
    extended_istr_ptr: u16,
}

// all functions are greedy, meaning they allocated the first spots they can given their constraints
// this means that the caller must use the functions in order of importance (for example if certain
// instructions require a specific order place those first)
//
// caller also does not need to worry about the allocation of extended or simple instructions, only
// in the case the constraints cannot be satisfied (which should crash the program)
impl<'a> InstructionWriter<'a> {
    pub fn new(istr_set: &mut IstrSet) -> InstructionWriter<'_> {
        InstructionWriter {
            istr_set,
            simple_istr_ptr: 0,
            extended_istr_ptr: 0,
        }
    }

    /// True if no instruction exists at ir index
    fn is_empty(&self, idx: u8) -> bool {
        self.istr_set.is_empty(idx)
    }

    /// True if can place simple instruction at ir index (i.e. the opcode is empty)
    fn simple_available(&self, idx: u8) -> bool {
        self.is_empty(idx)
    }

    /// Allocates an extended instruction at the specified ir index
    fn allocate_extended_idx(&mut self, idx: u8) -> Result<(), AllocationError> {
        if !self.is_empty(idx) {
            return Err(AllocationError);
        }

        *self.istr_set.get_istr_mut(idx) = InstructionEntry::Extended(Box::new(Extended::new()));
        Ok(())
    }

    /// Returns Ok if idx is empty OR idx is an extended istr and there are spots available
    /// Otherwise, returns error determining why spot is not available
    fn extended_available(&self, idx: u8) -> Result<(), ExtPlacementError> {
        match self.istr_set.get_istr(idx) {
            InstructionEntry::Single(_) => Err(ExtPlacementError::IndexNotFree(idx)),
            InstructionEntry::Extended(extended) => {
                if extended.is_full() {
                    Err(ExtPlacementError::IndexFull(idx))
                } else {
                    Ok(())
                }
            }
            InstructionEntry::Empty => Ok(()),
        }
    }

    // Attempts to place extended at specified ir idx in first spot in the extended instruction
    // returning if the operation was successful.
    ///
    /// * `istr`: Instruction to place
    /// * `idx`: ir index
    fn place_extended_idx(
        &mut self,
        istr: InstructionImpl,
        idx: u8,
    ) -> Result<(), ExtPlacementError> {
        // make sure the idx is available
        self.extended_available(idx)?;

        // allocate first if needed
        if self.is_empty(idx) {
            // allocation expected to work
            self.allocate_extended_idx(idx).unwrap();
        }

        if let InstructionEntry::Extended(extended) = self.istr_set.get_istr_mut(idx) {
            extended.push(istr);
            Ok(())
        } else {
            unreachable!()
        }
    }

    // attempts to place simple at specified ir idx
    fn place_simple_idx(&mut self, istr: InstructionImpl, idx: u8) -> Result<(), AllocationError> {
        if !self.simple_available(idx) {
            return Err(AllocationError);
        }

        *self.istr_set.get_istr_mut(idx) = InstructionEntry::Single(Box::new(Single::new(istr)));
        Ok(())
    }

    /// places given instructions in specified ranges of IR, if possible
    /// all instructions are extended
    ///
    /// removes all instructions placed from the given vector in a front to back order.
    ///
    /// * `istr`: Instruction to place
    /// * `ranges`: Inclusive ranges [start, end] of ir indexes where istr is allowed to be placed
    pub fn place_extended_ranges(
        &mut self,
        istr: InstructionImpl,
        ranges: &[(u8, u8)],
    ) -> Result<(), FilledRangesError> {
        for &(start, end) in ranges.iter() {
            for idx in start..=end {
                if self.extended_available(idx).is_ok() {
                    // expected to not fail
                    self.place_extended_idx(istr, idx).unwrap();
                    return Ok(());
                }
            }
        }
        Err(FilledRangesError)
    }

    // places simple instruction in first available slot
    // if none available returns ?
    pub fn place_simple(&mut self, istr: InstructionImpl) -> Result<(), PlacementError> {
        // TODO: move hardcoded values elsewhere

        // finds smallest index at which a valid spot is available
        while self.simple_istr_ptr < 256 && !self.simple_available(self.simple_istr_ptr as u8) {
            self.simple_istr_ptr += 1;
        }

        // no available slots
        if self.simple_istr_ptr >= 256 {
            return Err(PlacementError);
        }

        // expected not to fail
        self.place_simple_idx(istr, self.simple_istr_ptr as u8)
            .unwrap();

        Ok(())
    }

    // places extended in first available slot
    // returns none if no spots available
    pub fn place_extended(&mut self, istr: InstructionImpl) -> Result<(), PlacementError> {
        // TODO: move hardcoded values elsewhere

        // finds smallest index at which a valid spot is available
        while self.extended_istr_ptr < 256
            && self
                .extended_available(self.extended_istr_ptr as u8)
                .is_err()
        {
            self.extended_istr_ptr += 1;
        }

        // no available slots
        if self.extended_istr_ptr >= 256 {
            return Err(PlacementError);
        }

        // expected not to fail
        self.place_extended_idx(istr, self.extended_istr_ptr as u8)
            .unwrap();

        Ok(())
    }
}

#[derive(Debug)]
enum ExtPlacementError {
    IndexNotFree(u8),
    IndexFull(u8),
}

impl Error for ExtPlacementError {}

impl fmt::Display for ExtPlacementError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExtPlacementError::IndexNotFree(idx) => {
                write!(f, "Opcode is already taken at ir: {}", idx)
            }
            ExtPlacementError::IndexFull(idx) => {
                write!(f, "All extended instructions filled at ir: {}", idx)
            }
        }
    }
}

#[derive(Debug)]
pub struct FilledRangesError;

impl Error for FilledRangesError {}

impl fmt::Display for FilledRangesError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No opcodes available in given ranges!")
    }
}

#[derive(Debug)]
struct AllocationError;

impl Error for AllocationError {}

impl fmt::Display for AllocationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "index is not empty!")
    }
}

#[derive(Debug)]
pub struct PlacementError;

impl Error for PlacementError {}

impl fmt::Display for PlacementError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "No opcodes available!")
    }
}
