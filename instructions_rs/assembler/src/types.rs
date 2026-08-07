#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    /// Integer type (64-bit signed)
    Int,
    /// Boolean type
    Bool,
    Unit,
    String,
    Character,

    Register,
    AddressRegister,

    Byte,
    Address,

    Label,

    /// Unknown type
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: Type,
}
