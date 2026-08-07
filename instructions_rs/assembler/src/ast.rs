use crate::types::Type;

pub type Program = Vec<Statement>;

use opcode_gen::instructions::InstructionSignature;

type Address = u16;

#[derive(Debug, Clone, PartialEq)]
pub struct AddressedStatement {
    statement: Statement,
    address: Option<Address>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction {
    pub name: String,
    pub params: Vec<TypedExpr>,
    pub istr_signature: Option<InstructionSignature>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Label { name: String },
    BlockLabel { name: String, body: Vec<Statement> },
    Instruction(Instruction),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedExpr {
    pub expr: Expr,
    pub ty: Type,
}

impl TypedExpr {
    pub fn new(expr: Expr, ty: Type) -> Self {
        TypedExpr { expr, ty }
    }

    pub fn unknown(expr: Expr) -> Self {
        TypedExpr {
            expr,
            ty: Type::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    /// Integer literal
    Int(i64),
    /// Boolean literal
    Bool(bool),
    /// String literal
    String(String),
    /// Char literal
    Char(u8),
    /// Identity (could be var, reg, etc.)
    Identity(String),
    /// Unary operation
    Unary { op: UnaryOp, expr: Box<TypedExpr> },
    /// Binary operation
    Binary {
        op: BinaryOp,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UnaryOp {
    Neg,         // -
    Not,         // !
    BitNegation, // ~
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,

    ShiftLeft,
    ShiftRight,
    BitAnd,
    BitXor,
    BitOr,

    // Comparison
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,

    // bool operations
    And,
    Or,
}
