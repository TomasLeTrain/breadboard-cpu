use std::{cell::RefCell, rc::Rc};

use crate::types::Type;

pub type Program = Vec<Statement>;

use opcode_gen::instructions::InstructionSignature;

type Address = u16;

#[derive(Debug, Clone)]
pub struct AddressedStatement {
    statement: Statement,
    address: Option<Address>,
}

#[derive(Debug, Clone)]
pub struct AstInstruction {
    pub name: String,
    pub params: Vec<TypedExpr>,
    pub istr_signature: Option<InstructionSignature>,
    pub instruction: Option<Rc<opcode_gen::instructions::Instruction>>,
}
impl AstInstruction {
    pub fn new(name: String, params: Vec<TypedExpr>) -> Self {
        AstInstruction {
            name,
            params,
            istr_signature: None,
            instruction: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Label { name: String },
    BlockLabel { name: String, body: Vec<Statement> },
    Instruction(AstInstruction),
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

impl Expr {
    pub fn as_identity(&self) -> Option<String> {
        if let Expr::Identity(name) = self {
            Some(name.clone())
        } else {
            None
        }
    }
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
