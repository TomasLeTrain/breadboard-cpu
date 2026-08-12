use std::sync::Arc;
use std::{fmt::Debug, rc::Rc};

use crate::types::Type;

use miette::SourceSpan;
use opcode_gen::instructions::InstructionSignature;
use pest::{RuleType, Span, iterators::Pair};

type Address = u16;

pub type Ast = Vec<AstNode<Statement>>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedSourceFile {
    source: Arc<str>,
    file_name: String,
}

impl NamedSourceFile {
    pub fn new(source: String, file_name: String) -> Self {
        Self {
            source: Arc::from(source),
            file_name,
        }
    }

    pub fn source_str(&self) -> &str {
        &self.source
    }

    pub fn source(&self) -> &Arc<str> {
        &self.source
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }
}

pub type Source = Arc<NamedSourceFile>;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AstSpan {
    start: usize,
    end: usize,
    source: Source,
}

impl From<AstSpan> for SourceSpan {
    fn from(value: AstSpan) -> Self {
        value.to_miette_span()
    }
}

impl From<&AstSpan> for SourceSpan {
    fn from(value: &AstSpan) -> Self {
        value.to_miette_span()
    }
}

impl Debug for AstSpan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AstSpan")
            .field("str", &self.get_str())
            .field("range", &format!("{}..{}", self.start, self.end))
            .finish()
    }
}

impl AstSpan {
    pub fn new(start: usize, end: usize, source: Source) -> Self {
        Self { start, end, source }
    }

    pub fn from_span<'a>(span: Span<'a>, source: &Source) -> Self {
        Self {
            start: span.start(),
            end: span.end(),
            source: Arc::clone(source),
        }
    }

    pub fn from_pair<R: RuleType>(pair: Pair<R>, source: &Source) -> Self {
        Self::from_span(pair.as_span(), source)
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    pub fn set_span(&mut self, start: usize, end: usize) {
        self.start = start;
        self.end = end;
    }

    pub fn source(&self) -> &Arc<NamedSourceFile> {
        &self.source
    }

    pub fn source_ref(&self) -> &NamedSourceFile {
        &self.source
    }

    pub fn to_span<'a>(&'a self) -> Span<'a> {
        Span::new(self.source().source(), self.start(), self.end()).unwrap()
    }

    pub fn to_miette_span(&self) -> SourceSpan {
        SourceSpan::new(self.start().into(), self.end() - self.start())
    }

    pub fn to_miette_source_code(&self) -> miette::NamedSource<Arc<str>> {
        miette::NamedSource::new(
            self.source().file_name(),
            Arc::clone(self.source().source()),
        )
    }

    pub fn get_str(&self) -> &str {
        &self.source.source()[self.start..self.end]
    }
}

/// wraps T with additional information tied to each token (ex. parent file, span, etc.)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AstNode<T> {
    pub inner: T,
    pub span: AstSpan,
}

impl<T> AstNode<T> {
    pub fn new(inner: T, span: AstSpan) -> Self {
        Self { inner, span }
    }

    pub fn from_pair<R: RuleType>(inner: T, pair: Pair<R>, source: &Source) -> Self {
        Self::new(inner, AstSpan::from_pair(pair, source))
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    pub fn span(&self) -> &AstSpan {
        &self.span
    }
}

#[derive(Debug, Clone)]
pub struct AddressedStatement {
    statement: Statement,
    address: Option<Address>,
}

impl AddressedStatement {
    pub fn statement<'a>(&'a self) -> &'a Statement {
        &self.statement
    }
}

#[derive(Debug, Clone)]
pub struct AstInstruction {
    pub name: String,
    pub params: Vec<AstNode<TypedExpr>>,
    pub istr_signature: Option<InstructionSignature>,
    pub instruction: Option<Rc<opcode_gen::instructions::Instruction>>,
}
impl AstInstruction {
    pub fn new(name: String, params: Vec<AstNode<TypedExpr>>) -> Self {
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
    Label {
        name: String,
    },
    BlockLabel {
        name: String,
        body: Vec<AstNode<Statement>>,
    },
    Instruction(AstInstruction),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypedExpr {
    pub expr: Expr,
    pub ty: Type,
}

impl TypedExpr {
    pub fn new(expr: Expr, ty: Type) -> Self {
        Self { expr, ty }
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
    Unary {
        op: UnaryOp,
        expr: Box<AstNode<TypedExpr>>,
    },
    /// Binary operation
    Binary {
        op: BinaryOp,
        left: Box<AstNode<TypedExpr>>,
        right: Box<AstNode<TypedExpr>>,
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
