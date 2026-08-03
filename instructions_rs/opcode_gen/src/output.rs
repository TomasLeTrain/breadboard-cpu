//! Implements how the output of the roms is represented and stored.

use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct Output {
    // output only needs 16 bytes, can compactly represent all categories
    // also has bonus of representing same as hardware so converting between is trivial
    data: u16,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputCategory {
    Bout,
    Write,
    Addr,
    Misc,
    FlagSelect,
    PcCnt,
}

#[derive(Debug)]
pub enum SetCategoryError {
    CategorySizeExceeded(OutputCategory, u8),
    CategoryIntersection(OutputCategory, u8, u8),
}

impl Error for SetCategoryError {}

impl fmt::Display for SetCategoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SetCategoryError::CategorySizeExceeded(category, value) => {
                write!(
                    f,
                    "Category {:#?} had its size exceeded by value {:#?}",
                    category, value
                )
            }
            SetCategoryError::CategoryIntersection(category, prev_value, new_value) => {
                write!(
                    f,
                    "Category {:#?} has intersection - previous value: {:#?}, new value: {:#?}",
                    category, prev_value, new_value
                )
            }
        }
    }
}

impl Output {
    const BOUT_BITS: u8 = 4;
    const WRITE_BITS: u8 = 4;
    const ADDR_BITS: u8 = 2;
    const MISC_BITS: u8 = 2;
    const FLAG_SELECT_BITS: u8 = 3;

    // these also function as masks
    const MAX_BOUT_VAL: u8 = (1 << Self::BOUT_BITS) - 1;
    const MAX_WRITE_VAL: u8 = (1 << Self::WRITE_BITS) - 1;
    const MAX_ADDR_VAL: u8 = (1 << Self::ADDR_BITS) - 1;
    const MAX_MISC_VAL: u8 = (1 << Self::MISC_BITS) - 1;
    const MAX_FLAG_SELECT_VAL: u8 = (1 << Self::FLAG_SELECT_BITS) - 1;
    const MAX_PC_CNT_VAL: u8 = 1;

    fn get_max_value(category: OutputCategory) -> u8 {
        match category {
            OutputCategory::Bout => Self::MAX_BOUT_VAL,
            OutputCategory::Write => Self::MAX_WRITE_VAL,
            OutputCategory::Addr => Self::MAX_ADDR_VAL,
            OutputCategory::Misc => Self::MAX_MISC_VAL,
            OutputCategory::FlagSelect => Self::MAX_FLAG_SELECT_VAL,
            OutputCategory::PcCnt => Self::MAX_PC_CNT_VAL,
        }
    }

    /// Gets offset in bits at which the category starts in data.
    fn get_output_offset(category: OutputCategory) -> u16 {
        let mut result = 0;

        if matches!(category, OutputCategory::Bout) {
            return result;
        }
        result += Self::BOUT_BITS as u16;

        if matches!(category, OutputCategory::Write) {
            return result;
        }
        result += Self::WRITE_BITS as u16;

        if matches!(category, OutputCategory::Addr) {
            return result;
        }
        result += Self::ADDR_BITS as u16;

        if matches!(category, OutputCategory::Misc) {
            return result;
        }
        result += Self::MISC_BITS as u16;

        if matches!(category, OutputCategory::FlagSelect) {
            return result;
        }
        result += Self::FLAG_SELECT_BITS as u16;

        if matches!(category, OutputCategory::PcCnt) {
            return result;
        }

        unreachable!()
    }

    /// Gets category data from the output.
    fn get_category(&self, category: OutputCategory) -> u8 {
        let mut result = self.data;
        result >>= Self::get_output_offset(category);
        result &= Self::get_max_value(category) as u16;
        result as u8
    }

    /// attempts to set/merge the value for a given category
    fn merge_category(
        &mut self,
        val: u8,
        category: OutputCategory,
    ) -> Result<(), SetCategoryError> {
        // no need to merge anything
        if val == 0 {
            return Ok(());
        }

        if val > Self::get_max_value(category) {
            Err(SetCategoryError::CategorySizeExceeded(category, val))
        } else if self.get_category(category) != 0 {
            Err(SetCategoryError::CategoryIntersection(
                category,
                self.get_category(category),
                val,
            ))
        } else {
            self.data |= (val as u16) << Self::get_output_offset(category);
            Ok(())
        }
    }

    fn get_bout(&self) -> u8 {
        self.get_category(OutputCategory::Bout)
    }

    fn get_write(&self) -> u8 {
        self.get_category(OutputCategory::Write)
    }

    fn get_addr(&self) -> u8 {
        self.get_category(OutputCategory::Addr)
    }

    fn get_misc(&self) -> u8 {
        self.get_category(OutputCategory::Misc)
    }

    fn get_flag_select(&self) -> u8 {
        self.get_category(OutputCategory::FlagSelect)
    }

    fn get_pc_cnt(&self) -> bool {
        self.get_category(OutputCategory::PcCnt) != 0
    }

    fn merge_bout(&mut self, val: u8) -> Result<(), SetCategoryError> {
        self.merge_category(val, OutputCategory::Bout)
    }

    fn merge_write(&mut self, val: u8) -> Result<(), SetCategoryError> {
        self.merge_category(val, OutputCategory::Write)
    }

    fn merge_addr(&mut self, val: u8) -> Result<(), SetCategoryError> {
        self.merge_category(val, OutputCategory::Addr)
    }

    fn merge_misc(&mut self, val: u8) -> Result<(), SetCategoryError> {
        self.merge_category(val, OutputCategory::Misc)
    }

    fn merge_flag_select(&mut self, val: u8) -> Result<(), SetCategoryError> {
        self.merge_category(val, OutputCategory::FlagSelect)
    }

    fn merge_pc_cnt(&mut self, val: bool) -> Result<(), SetCategoryError> {
        self.merge_category(val as u8, OutputCategory::PcCnt)
    }

    pub fn new() -> Self {
        Self { data: 0 }
    }

    pub fn from_bout(val: u8) -> Self {
        let mut result = Self::new();
        result.merge_bout(val).unwrap();
        result
    }

    pub fn from_write(val: u8) -> Self {
        let mut result = Self::new();
        result.merge_write(val).unwrap();
        result
    }

    pub fn from_addr(val: u8) -> Self {
        let mut result = Self::new();
        result.merge_addr(val).unwrap();
        result
    }

    pub fn from_other(val: u8) -> Self {
        let mut result = Self::new();
        result.merge_misc(val).unwrap();
        result
    }

    pub fn from_flag_select(val: u8) -> Self {
        let mut result = Self::new();
        result.merge_flag_select(val).unwrap();
        result
    }

    pub fn from_pc_cnt(val: bool) -> Self {
        let mut result = Self::new();
        result.merge_pc_cnt(val).unwrap();
        result
    }

    pub fn from_arr(arr: &[Self]) -> Output {
        let mut result = Self::new();
        for curr in arr {
            result.merge(curr).unwrap()
        }
        result
    }

    /// Returns true if any categories intersect.
    ///
    /// * `other`: Output to compare to.
    pub fn intersect(&self, other: &Self) -> bool {
        for category in [
            OutputCategory::Bout,
            OutputCategory::Write,
            OutputCategory::Addr,
            OutputCategory::Misc,
            OutputCategory::FlagSelect,
            OutputCategory::PcCnt,
        ] {
            if self.get_category(category) > 0 && other.get_category(category) > 0 {
                return true;
            }
        }
        false
    }

    pub fn merge(&mut self, other: &Self) -> Result<(), SetCategoryError> {
        self.merge_bout(other.get_bout())?;
        self.merge_write(other.get_write())?;
        self.merge_addr(other.get_addr())?;
        self.merge_misc(other.get_misc())?;
        self.merge_flag_select(other.get_flag_select())?;
        self.merge_pc_cnt(other.get_pc_cnt())?;
        Ok(())
    }
}

impl<const N: usize> From<[Output; N]> for Output {
    fn from(arr: [Output; N]) -> Self {
        let mut result = Self::new();
        for curr in arr.iter() {
            result.merge(curr).unwrap()
        }
        result
    }
}
