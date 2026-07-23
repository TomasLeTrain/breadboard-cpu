// TODO: ensure values created are within their max sizes
#[derive(Clone, Copy)]
pub struct Output {
    bout: u8,
    write: u8,
    addr: u8,
    misc: u8,
    flag_select: u8,
    pc_cnt: u8,
}

impl Output {
    pub fn new() -> Self {
        Self {
            bout: 0,
            write: 0,
            addr: 0,
            misc: 0,
            flag_select: 0,
            pc_cnt: 0,
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

    pub fn from_pc_cnt(val: u8) -> Self {
        let mut result = Self::new();
        result.pc_cnt = val;
        result
    }

    pub fn from_arr(arr: &[Self]) -> Output {
        let mut result = Self::new();
        for curr in arr {
            result.merge(curr)
        }
        result
    }

    pub fn intersect(&self, other: &Self) -> bool {
        let category_intersects = |a: u8, b: u8| -> bool { a > 0 && b > 0 };

        category_intersects(self.bout, other.bout)
            || category_intersects(self.write, other.write)
            || category_intersects(self.addr, other.addr)
            || category_intersects(self.flag_select, other.flag_select)
            || category_intersects(self.pc_cnt, other.pc_cnt)
    }

    pub fn merge(&mut self, other: &Self) {
        // TODO: add intersect assert
        self.bout |= other.bout;
        self.write |= other.write;
        self.addr |= other.addr;
        self.misc |= other.misc;
        self.flag_select |= other.flag_select;
        self.pc_cnt |= other.pc_cnt;
    }
}

impl<const N: usize> From<[Output; N]> for Output {
    fn from(arr: [Output; N]) -> Self {
        let mut result = Self::new();
        for curr in arr.iter() {
            result.merge(curr)
        }
        result
    }
}
