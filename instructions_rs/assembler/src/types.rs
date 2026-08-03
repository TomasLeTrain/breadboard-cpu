// ANCHOR: type_enum
/// Types in our language
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// Integer type (64-bit signed)
    Int,
    /// Boolean type
    Bool,
    /// Unit type (for statements with no value)
    Unit,
    /// Unknown type (for type inference)
    Unknown,
}
// ANCHOR_END: type_enum
