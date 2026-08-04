#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    Address,
    /// Unknown type
    Unknown,
}
