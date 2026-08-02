use std::{collections::HashMap, sync::LazyLock};

use crate::{
    action::Action::{self, *},
    output::Output,
};

static ACTION_TO_OUTPUT_MAP: LazyLock<HashMap<Action, Output>> = LazyLock::new(|| {
    HashMap::from([
        (Halt, Output::from_bout(5)),
        (Nop, Output::new()),
        (Reset, Output::from_other(2)),
        // addr regs cnt
        (PcCnt, Output::from_pc_cnt(true)),
        (MarCnt, Output::from_bout(4)),
        (SpDec, Output::from_bout(7)),
        (SpInc, Output::from_bout(6)),
        // addr regs addr
        (PcAddr, Output::from_addr(1)),
        (MarAddr, Output::from_addr(2)),
        (SpAddr, Output::from_addr(3)),
        // registers bout
        (ABout, Output::from_bout(0b1000)),
        (BBout, Output::from_bout(0b1000 | 1)),
        (XBout, Output::from_bout(0b1000 | 5)),
        (YBout, Output::from_bout(0b1000 | 6)),
        (ZBout, Output::from_bout(0b1000 | 7)),
        (
            PcLoBout,
            [Output::from_addr(1), Output::from_bout(2)].into(),
        ),
        (
            PcHiBout,
            [Output::from_addr(1), Output::from_bout(3)].into(),
        ),
        (
            MarLoBout,
            [Output::from_addr(2), Output::from_bout(2)].into(),
        ),
        (
            MarHiBout,
            [Output::from_addr(2), Output::from_bout(3)].into(),
        ),
        (
            SpLoBout,
            [Output::from_addr(3), Output::from_bout(2)].into(),
        ),
        (
            SpHiBout,
            [Output::from_addr(3), Output::from_bout(3)].into(),
        ),
        (KeybBout, Output::from_bout(0b1000 | 2)),
        (FlagsBout, Output::from_bout(0b1000 | 4)),
        (FAluBout, Output::from_bout(0b1000 | 3)),
        // registers write
        (AWrite, Output::from_write(0b1000)),
        (BWrite, Output::from_write(0b1000 | 1)),
        (XWrite, Output::from_write(0b1000 | 5)),
        (YWrite, Output::from_write(0b1000 | 6)),
        (ZWrite, Output::from_write(7)),
        (PcLoWrite, Output::from_write(0b1000 | 3)),
        (PcHiWrite, Output::from_write(0b1000 | 2)),
        (MarLoWrite, Output::from_write(5)),
        (MarHiWrite, Output::from_write(4)),
        (SpLoWrite, Output::from_write(3)),
        (SpHiWrite, Output::from_write(2)),
        // ir regs
        (IrWrite, Output::from_write(0b1000 | 7)),
        (Ir2Write, Output::from_write(1)),
        // mem read/write
        (MemRead, Output::from_bout(1)),
        (MemWrite, Output::from_write(6)),
        // vram read/write
        (
            VramRead,
            [Output::from_other(3), Output::from_flag_select(5)].into(),
        ),
        (
            VramWrite,
            [Output::from_other(3), Output::from_flag_select(4)].into(),
        ),
        // shift left
        (
            XShiftLeft,
            [Output::from_other(3), Output::from_flag_select(0)].into(),
        ),
        (
            YShiftLeft,
            [Output::from_other(3), Output::from_flag_select(2)].into(),
        ),
        // shift right
        (
            XShiftRight,
            [Output::from_other(3), Output::from_flag_select(1)].into(),
        ),
        (
            YShiftRight,
            [Output::from_other(3), Output::from_flag_select(3)].into(),
        ),
        // flags
        (FlagDirect, Output::from_flag_select(0)),
        (FlagCarry, Output::from_flag_select(1)),
        (FlagEq, Output::from_flag_select(2)),
        (FlagNotZero, Output::from_flag_select(3)),
        (Flag5, Output::from_flag_select(4)),
        (Flag6, Output::from_flag_select(5)),
        (Flag7, Output::from_flag_select(6)),
        (Flag8, Output::from_flag_select(7)),
        (FlagWriteAlu, Output::from_other(1)),
    ])
});

impl Action {
    pub fn to_output(&self) -> Output {
        *ACTION_TO_OUTPUT_MAP.get(self).unwrap()
    }
}
