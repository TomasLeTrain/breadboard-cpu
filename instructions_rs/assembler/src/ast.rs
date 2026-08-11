use std::fmt::Debug;
use std::{path::Path, rc::Rc};

use crate::types::Type;

use opcode_gen::instructions::InstructionSignature;
use pest::{RuleType, Span, iterators::Pair};

type Address = u16;

type Ast<'a> = Vec<AstNode<'a, Statement<'a>>>;

pub struct AstSpan {
    start: usize,
    len: usize,
    source: Rc<str>,
}

#[derive(Debug)]
pub struct FileAst<'a> {
    statements: Ast<'a>,

    // TODO: should be taking ownership of source?
    source: &'a str,
    file_path: &'a Path,
}

impl<'a> FileAst<'a> {
    pub fn new(source: &'a str, file_path: &'a Path) -> Self {
        Self {
            statements: Ast::new(),
            source,
            file_path,
        }
    }

    pub fn statements_mut(&mut self) -> &mut Ast<'a> {
        &mut self.statements
    }
}

/// wraps T with additional information tied to each token (ex. parent file, span, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AstNode<'a, T> {
    pub inner: T,
    pub span: Span<'a>,
}

impl<'a, T> AstNode<'a, T> {
    pub fn new(inner: T, span: Span<'a>) -> Self {
        Self { inner, span }
    }

    pub fn from_pair<R: RuleType>(inner: T, pair: Pair<'a, R>) -> Self {
        Self::new(inner, pair.as_span())
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn span(&'a self) -> Span<'a> {
        self.span
    }
}

#[derive(Debug, Clone)]
pub struct AddressedStatement<'a> {
    statement: Statement<'a>,
    address: Option<Address>,
}

impl<'a> AddressedStatement<'a> {
    pub fn statement(&'a self) -> &'a Statement<'a> {
        &self.statement
    }
}

#[derive(Debug, Clone)]
pub struct AstInstruction<'a> {
    pub name: String,
    pub params: Vec<AstNode<'a, TypedExpr<'a>>>,
    pub istr_signature: Option<InstructionSignature>,
    pub instruction: Option<Rc<opcode_gen::instructions::Instruction>>,
}
impl<'a> AstInstruction<'a> {
    pub fn new(name: String, params: Vec<AstNode<'a, TypedExpr<'a>>>) -> Self {
        AstInstruction {
            name,
            params,
            istr_signature: None,
            instruction: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Statement<'a> {
    Label {
        name: String,
    },
    BlockLabel {
        name: String,
        body: Vec<AstNode<'a, Statement<'a>>>,
    },
    Instruction(AstInstruction<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedExpr<'a> {
    pub expr: Expr<'a>,
    pub ty: Type,
}

impl<'a> TypedExpr<'a> {
    pub fn new(expr: Expr<'a>, ty: Type) -> Self {
        Self { expr, ty }
    }

    pub fn unknown(expr: Expr<'a>) -> Self {
        TypedExpr {
            expr,
            ty: Type::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr<'a> {
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
    Unary {
        op: UnaryOp,
        expr: Box<AstNode<'a, TypedExpr<'a>>>,
    },
    /// Binary operation
    Binary {
        op: BinaryOp,
        left: Box<AstNode<'a, TypedExpr<'a>>>,
        right: Box<AstNode<'a, TypedExpr<'a>>>,
    },
}

impl<'a> Expr<'a> {
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
