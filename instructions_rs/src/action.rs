mod defs;

#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub enum Action {
    Halt,
    Nop, // equal to zero outputs
    Reset,

    // addr regs cnt
    PcCnt,
    MarCnt,
    SpDec,
    SpInc,

    // addr regs addr
    PcAddr,
    MarAddr,
    SpAddr,

    // registers bout
    ABout,
    BBout,
    XBout,
    YBout,
    ZBout,
    PcLoBout,
    PcHiBout,
    MarLoBout,
    MarHiBout,
    SpLoBout,
    SpHiBout,
    KeybBout,
    FlagsBout,
    FAluBout,

    // registers write
    AWrite,
    BWrite,
    XWrite,
    YWrite,
    ZWrite,
    PcLoWrite,
    PcHiWrite,
    MarLoWrite,
    MarHiWrite,
    SpLoWrite,
    SpHiWrite,

    // ir regs
    IrWrite,
    Ir2Write,

    // mem read/write
    MemRead,
    MemWrite,

    // vram read/write
    VramRead,
    VramWrite,

    // register shifts
    XShiftLeft,
    YShiftLeft,
    XShiftRight,
    YShiftRight,

    // flags
    FlagDirect,
    FlagCarry,
    FlagEq,
    FlagZero,
    Flag6,
    Flag5,
    Flag7,
    Flag8,

    FlagWriteAlu,

    // placeholder registers
    Reg0Bout,
    Reg1Bout,
    Reg0Write,
    Reg1Write,

    Addr0HiBout,
    Addr0HiWrite,
    Addr0LoBout,
    Addr0LoWrite,
    Addr0Out,

    Addr1HiBout,
    Addr1HiWrite,
    Addr1LoBout,
    Addr1LoWrite,
    Addr1Out,

    OutputFlagsSelector,
}
