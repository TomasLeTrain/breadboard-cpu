#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Integer type (64-bit signed)
    Int,
    /// Boolean type
    Bool,
    /// Unit type (for statements with no value)
    Unit,
    String,
    Character,
    /// Unknown type (for type inference)
    Unknown,
}
