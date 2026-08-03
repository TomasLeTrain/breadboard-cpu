use std::error::Error;
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct Output {
    // output only needs 16 bytes, can compactly represent all categories
    // also has bonus of representing same as hardware so converting between is trivial
    data: u16,
    // // 4 bits wide
    // bout: u8,
    // // 4 bits wide
    // write: u8,
    // // 2 bits wide
    // addr: u8,
    // // 3 bits wide
    // misc: u8,
    // // 3 bits wide
    // flag_select: u8,
    // // 1 bit wide
    // pc_cnt: bool,
}

#[derive(Debug)]
enum OutputCategory {
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
                    "Category has intersection - previous value: {:#?}, new value: {:#?}",
                    category, prev_value, new_value
                )
            }
        }
    }
}

// uint32_t result = 0;
// result |= bus_out;                                  // 4 bits wide
// result |= static_cast<uint32_t>(bus_write) << 4;    // 4 bits wide
// result |= static_cast<uint32_t>(addr_out) << 8;     // 2 bits wide
// result |= static_cast<uint32_t>(other) << 10;       // 2 bits wide
// result |= static_cast<uint32_t>(flag_select) << 12; // 3 bits wide
// result |= static_cast<uint32_t>(pc_cnt) << 15;      // 1 bit wide
// return static_cast<uint16_t>(result);

impl Output {
    fn get_bout(&self) -> u8 {
        let mut result = self.data;
        result &= Self::MAX_BOUT_VAL as u16;
        result as u8
    }

    fn get_write(&self) -> u8 {
        let mut result = self.data;
        result >>= Self::MAX_BOUT_VAL;
        result &= Self::MAX_WRITE_VAL as u16;
        result as u8
    }

    fn get_addr(&self) -> u8 {
        let mut result = self.data;
        result >>= Self::MAX_BOUT_VAL;
        result >>= Self::MAX_WRITE_VAL;
        result &= Self::MAX_ADDR_VAL as u16;
        result as u8
    }

    fn get_misc(&self) -> u8 {
        let mut result = self.data;
        result >>= Self::MAX_BOUT_VAL;
        result >>= Self::MAX_WRITE_VAL;
        result >>= Self::MAX_ADDR_VAL;
        result &= Self::MAX_MISC_VAL as u16;
        result as u8
    }

    fn set_bout(&mut self, val: u8) -> Result<(), SetCategoryError> {
        if val > Self::MAX_BOUT_VAL {
            Err(SetCategoryError::CategorySizeExceeded(
                OutputCategory::Bout,
                val,
            ))
        } else if self.get_bout() != 0 && val != 0 {
            Err(SetCategoryError::CategoryIntersection(
                OutputCategory::Bout,
                self.get_bout(),
                val,
            ))
        } else {
            self.data |= (val as u16) << Self::WRITE_BITS;
            Ok(())
        }
    }

    pub fn new() -> Self {
        Self { data: 0 }
    }

    pub fn from_write(val: u8) -> Self {
        let mut result = Self::new();
        result.write = val;
        assert!(
            !result.sizes_exceeded(),
            "write value outside category size"
        );
        result
    }

    pub fn from_bout(val: u8) -> Self {
        let mut result = Self::new();
        result.bout = val;
        assert!(!result.sizes_exceeded(), "bout value outside category size");
        result
    }

    pub fn from_addr(val: u8) -> Self {
        let mut result = Self::new();
        result.addr = val;
        assert!(!result.sizes_exceeded(), "addr value outside category size");
        result
    }

    pub fn from_other(val: u8) -> Self {
        let mut result = Self::new();
        result.misc = val;
        assert!(!result.sizes_exceeded(), "misc value outside category size");
        result
    }

    pub fn from_flag_select(val: u8) -> Self {
        let mut result = Self::new();
        result.flag_select = val;
        assert!(!result.sizes_exceeded());
        assert!(!result.sizes_exceeded(), "misc value outside category size");
        result
    }

    pub fn from_pc_cnt(val: bool) -> Self {
        let mut result = Self::new();
        result.pc_cnt = val;
        result
    }

    pub fn from_arr(arr: &[Self]) -> Output {
        let mut result = Self::new();
        for curr in arr {
            result.merge(curr).unwrap()
        }
        result
    }

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

    // returns true if one of the fields has a value greater than allowed
    fn sizes_exceeded(&self) -> bool {
        self.bout > Self::MAX_BOUT_VAL
            || self.write > Self::MAX_WRITE_VAL
            || self.addr > Self::MAX_ADDR_VAL
            || self.misc > Self::MAX_MISC_VAL
            || self.flag_select > Self::MAX_FLAG_SELECT_VAL
    }

    pub fn intersect(&self, other: &Self) -> bool {
        let category_intersects = |a: u8, b: u8| -> bool { a > 0 && b > 0 };

        category_intersects(self.bout, other.bout)
            || category_intersects(self.write, other.write)
            || category_intersects(self.addr, other.addr)
            || category_intersects(self.flag_select, other.flag_select)
            || category_intersects(self.pc_cnt as u8, other.pc_cnt as u8)
    }

    pub fn merge(&mut self, other: &Self) -> Result<(), SetCategoryError> {
        if self.intersect(other) {
            return Err(SetCategoryError::CategoryIntersection(*self, *other));
        }

        self.bout |= other.bout;
        self.write |= other.write;
        self.addr |= other.addr;
        self.misc |= other.misc;
        self.flag_select |= other.flag_select;
        self.pc_cnt |= other.pc_cnt;
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
