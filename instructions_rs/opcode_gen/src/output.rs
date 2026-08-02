use core::fmt;
use std::error::Error;

// TODO: ensure values created are within their max sizes
#[derive(Clone, Copy, Debug)]
pub struct Output {
    bout: u8,
    write: u8,
    addr: u8,
    misc: u8,
    flag_select: u8,
    pc_cnt: bool,
}

#[derive(Debug)]
pub enum MergeOutputError {
    CategorySizeExceeded(Output),
    CategoryIntersection(Output, Output),
}

impl Error for MergeOutputError {}

impl fmt::Display for MergeOutputError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MergeOutputError::CategorySizeExceeded(output) => {
                write!(f, "Output exceeds category size: {:#?}", output)
            }
            MergeOutputError::CategoryIntersection(output1, output2) => {
                write!(f, "Outputs intersect: {:#?}, {:#?}", output1, output2)
            }
        }
    }
}

impl Output {
    pub fn new() -> Self {
        Self {
            bout: 0,
            write: 0,
            addr: 0,
            misc: 0,
            flag_select: 0,
            pc_cnt: false,
        }
    }

    pub fn from_write(val: u8) -> Self {
        let mut result = Self::new();
        result.write = val;
        result
    }

    pub fn from_bout(val: u8) -> Self {
        let mut result = Self::new();
        result.bout = val;
        result
    }

    pub fn from_addr(val: u8) -> Self {
        let mut result = Self::new();
        result.addr = val;
        result
    }

    pub fn from_other(val: u8) -> Self {
        let mut result = Self::new();
        result.misc = val;
        result
    }

    pub fn from_flag_select(val: u8) -> Self {
        let mut result = Self::new();
        result.flag_select = val;
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

    // returns true if one of the fields has a value greater than allowed

    const MAX_BOUT_VAL: u8 = (1 << 4) - 1;
    const MAX_WRITE_VAL: u8 = (1 << 4) - 1;
    const MAX_ADDR_VAL: u8 = (1 << 2) - 1;
    const MAX_MISC_VAL: u8 = (1 << 2) - 1;
    const MAX_FLAG_SELECT_VAL: u8 = (1 << 3) - 1;

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

    pub fn merge(&mut self, other: &Self) -> Result<(), MergeOutputError> {
        if self.sizes_exceeded() {
            return Err(MergeOutputError::CategorySizeExceeded(*self));
        }

        if other.sizes_exceeded() {
            return Err(MergeOutputError::CategorySizeExceeded(*other));
        }

        if self.intersect(other) {
            return Err(MergeOutputError::CategoryIntersection(*self, *other));
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
