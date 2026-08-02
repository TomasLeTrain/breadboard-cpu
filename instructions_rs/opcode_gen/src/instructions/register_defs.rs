use std::sync::LazyLock;

use crate::action::Action::{self, *};

#[derive(PartialEq, Debug)]
pub struct BoutRegister {
    pub bout: Action,
}
#[derive(PartialEq, Debug)]
pub struct WriteRegister {
    pub write: Action,
}
#[derive(PartialEq, Debug)]
pub struct BoutWriteRegister {
    pub bout: Action,
    pub write: Action,
}

pub struct ShiftRegister {
    pub register: BoutWriteRegister,
    pub shift_left: Action,
    pub shift_right: Action,
}

pub struct PcRegister {
    pub addr: Action,
    pub lo: BoutWriteRegister,
    pub hi: BoutWriteRegister,
    pub cnt: Action,
}

pub struct MarRegister {
    pub addr: Action,
    pub lo: BoutWriteRegister,
    pub hi: BoutWriteRegister,
    pub cnt: Action,
}

pub struct SpRegister {
    pub addr: Action,
    pub lo: BoutWriteRegister,
    pub hi: BoutWriteRegister,
    pub inc: Action,
    pub dec: Action,
}

pub struct MemStruct {
    pub bout: Action,
    pub write: Action,
}

pub static PC: LazyLock<PcRegister> = LazyLock::new(|| PcRegister {
    addr: PcAddr,
    lo: BoutWriteRegister {
        bout: PcLoBout,
        write: PcLoWrite,
    },
    hi: BoutWriteRegister {
        bout: PcHiBout,
        write: PcHiWrite,
    },
    cnt: PcCnt,
});

pub static MAR: LazyLock<MarRegister> = LazyLock::new(|| MarRegister {
    addr: MarAddr,
    lo: BoutWriteRegister {
        bout: MarLoBout,
        write: MarLoWrite,
    },
    hi: BoutWriteRegister {
        bout: MarHiBout,
        write: MarHiWrite,
    },
    cnt: MarCnt,
});

pub static SP: LazyLock<SpRegister> = LazyLock::new(|| SpRegister {
    addr: SpAddr,
    lo: BoutWriteRegister {
        bout: SpLoBout,
        write: SpLoWrite,
    },
    hi: BoutWriteRegister {
        bout: SpHiBout,
        write: SpHiWrite,
    },
    inc: SpInc,
    dec: SpDec,
});

pub static MEM: LazyLock<MemStruct> = LazyLock::new(|| MemStruct {
    bout: MemRead,
    write: MemWrite,
});

pub static A: LazyLock<BoutWriteRegister> = LazyLock::new(|| BoutWriteRegister {
    bout: ABout,
    write: AWrite,
});
pub static B: LazyLock<BoutWriteRegister> = LazyLock::new(|| BoutWriteRegister {
    bout: BBout,
    write: BWrite,
});

pub static X: LazyLock<ShiftRegister> = LazyLock::new(|| ShiftRegister {
    register: BoutWriteRegister {
        bout: XBout,
        write: XWrite,
    },
    shift_left: XShiftLeft,
    shift_right: XShiftRight,
});

pub static Y: LazyLock<ShiftRegister> = LazyLock::new(|| ShiftRegister {
    register: BoutWriteRegister {
        bout: YBout,
        write: YWrite,
    },
    shift_left: YShiftLeft,
    shift_right: YShiftRight,
});

pub static Z: LazyLock<BoutWriteRegister> = LazyLock::new(|| BoutWriteRegister {
    bout: ZBout,
    write: ZWrite,
});

pub static IR2: LazyLock<WriteRegister> = LazyLock::new(|| WriteRegister { write: Ir2Write });

pub static KEYB: LazyLock<BoutRegister> = LazyLock::new(|| BoutRegister { bout: KeybBout });
pub static FLAGS: LazyLock<BoutRegister> = LazyLock::new(|| BoutRegister { bout: FlagsBout });

pub trait Bout {
    fn bout(&self) -> Action;
}
pub trait Write {
    fn write(&self) -> Action;
}

pub trait AddressRegister {
    fn hi(&self) -> &(impl Bout + Write);
    fn lo(&self) -> &(impl Bout + Write);
    fn addr(&self) -> Action;
}

impl Bout for BoutWriteRegister {
    fn bout(&self) -> Action {
        self.bout
    }
}

impl Write for BoutWriteRegister {
    fn write(&self) -> Action {
        self.write
    }
}

impl Bout for BoutRegister {
    fn bout(&self) -> Action {
        self.bout
    }
}

impl Write for WriteRegister {
    fn write(&self) -> Action {
        self.write
    }
}

impl AddressRegister for PcRegister {
    fn hi(&self) -> &(impl Bout + Write) {
        &self.hi
    }
    fn lo(&self) -> &(impl Bout + Write) {
        &self.lo
    }
    fn addr(&self) -> Action {
        self.addr
    }
}

impl AddressRegister for SpRegister {
    fn hi(&self) -> &(impl Bout + Write) {
        &self.hi
    }
    fn lo(&self) -> &(impl Bout + Write) {
        &self.lo
    }
    fn addr(&self) -> Action {
        self.addr
    }
}

impl AddressRegister for MarRegister {
    fn hi(&self) -> &(impl Bout + Write) {
        &self.hi
    }
    fn lo(&self) -> &(impl Bout + Write) {
        &self.lo
    }
    fn addr(&self) -> Action {
        self.addr
    }
}
