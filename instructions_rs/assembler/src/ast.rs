use std::sync::Arc;
use std::{fmt::Debug, rc::Rc};

use crate::eval::ExprValue;
use crate::types::{Address, Type};

use miette::SourceSpan;
use pest::{RuleType, Span, iterators::Pair};

pub type Ast = Vec<StatementNode>;
pub type StatementNode = AstNode<Statement>;

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
pub struct Statement {
    statement: StatementKind,
    address: Option<Address>,
}

impl Statement {
    pub fn new(statement: StatementKind) -> Self {
        Self {
            statement,
            address: None,
        }
    }

    pub fn set_address(&mut self, address: Option<Address>) {
        self.address = address;
    }

    pub fn address(&self) -> Option<Address> {
        self.address
    }

    pub fn inner(&self) -> &StatementKind {
        &self.statement
    }

    pub fn inner_mut(&mut self) -> &mut StatementKind {
        &mut self.statement
    }
}

#[derive(Debug, Clone)]
pub struct AstInstruction {
    pub name: String,
    pub params: Vec<AstNode<Expr>>,
    pub instruction: Option<Rc<opcode_gen::instructions::Instruction>>,
}
impl AstInstruction {
    pub fn new(name: String, params: Vec<AstNode<Expr>>) -> Self {
        AstInstruction {
            name,
            params,
            instruction: None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    Label {
        name: String,
    },
    BlockLabel {
        name: String,
        body: Vec<StatementNode>,
    },
    Instruction(AstInstruction),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Type,
    pub value: ExprValue,
}

impl Expr {
    pub fn new(kind: ExprKind, ty: Type, value: ExprValue) -> Self {
        Self { kind, ty, value }
    }

    pub fn unknown(kind: ExprKind) -> Self {
        Expr {
            kind,
            ty: Type::Unknown,
            value: ExprValue::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExprKind {
    /// Literal
    Literal,
    /// Identity (could be var, reg, etc.)
    Identity(String),
    /// Unary operation
    Unary {
        op: UnaryOp,
        expr: Box<AstNode<Expr>>,
    },
    /// Binary operation
    Binary {
        op: BinaryOp,
        left: Box<AstNode<Expr>>,
        right: Box<AstNode<Expr>>,
    },
}

impl ExprKind {
    pub fn as_identity(&self) -> Option<String> {
        if let ExprKind::Identity(name) = self {
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
