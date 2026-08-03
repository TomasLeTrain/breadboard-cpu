use crate::types::Type;

/// A program is a list of top-level items (classes and statements)
pub type Program = Vec<Statement>;

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    /// Class definition
    Label {
        name: String,
    },
    BlockLabel {
        name: String,
        body: Vec<Statement>,
    },
    /// Statement (function definition or expression)
    Instruction {
        name: String,
        params: Vec<TypedExpr>,
    },
}

/// A typed expression: expression + its inferred type
#[derive(Debug, Clone, PartialEq)]
pub struct TypedExpr {
    pub expr: Expr,
    pub ty: Type,
}

/// Expressions in our language
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Integer literal
    Int(i64),
    /// Boolean literal
    Bool(bool),
    /// Variable reference
    Var(String),
    /// Self reference (inside methods)
    SelfRef,
    /// Unary operation
    Unary { op: UnaryOp, expr: Box<TypedExpr> },
    /// Binary operation
    Binary {
        op: BinaryOp,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
    },
    /// Block
    Block(Vec<Statement>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg, // -
    Not, // !
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Lt,
    Gt,
    Le,
    Ge,
    Eq,
    Ne,
}
