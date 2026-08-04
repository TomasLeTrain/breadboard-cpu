use crate::types::Type;

/// A program is a list of top-level items (classes and statements)
pub type Program = Vec<Statement>;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Label {
        name: String,
    },
    BlockLabel {
        name: String,
        body: Vec<Statement>,
    },
    Instruction {
        name: String,
        params: Vec<TypedExpr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,         // -
    Not,         // !
    BitNegation, // ~
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
