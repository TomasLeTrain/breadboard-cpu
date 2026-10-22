use std::fmt::Debug;
use std::sync::Arc;

use crate::eval::ExprValue;
use crate::parser::ParseContext;
use crate::types::{Address, Type};

use miette::{
    MietteError, MietteSpanContents, Result, SourceCode, SourceSpan, SpanContents, miette,
};
use pest::{RuleType, Span, iterators::Pair};

pub type Ast = Vec<StatementNode>;
pub type StatementNode = AstNode<Statement>;

// each span holds source index, can be converted to global easily, can get source/name easily, easy
// to store

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedSourceFile {
    source: String,
    file_name: String,
}

impl NamedSourceFile {
    pub fn new(source: String, file_name: String) -> Self {
        Self { source, file_name }
    }

    pub fn source_str(&self) -> &str {
        &self.source
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    fn inner(&self) -> &String {
        &self.source
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MultiFileSource {
    inner: Vec<NamedSourceFile>,
}

impl MultiFileSource {
    pub fn new(named_source_files: Vec<NamedSourceFile>) -> Self {
        Self {
            inner: named_source_files,
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    // returns global (start, end)
    pub fn to_global_span(
        &self,
        local_start: usize,
        local_end: usize,
        source_idx: usize,
    ) -> Result<(usize, usize)> {
        if source_idx >= self.inner.len() {
            Err(miette!("source_idx out of bounds"))?;
        }
        let source_offset: usize = self
            .inner
            .iter()
            .map(|e| e.inner().len())
            .take(source_idx)
            .sum();

        Ok((source_offset + local_start, source_offset + local_end))
    }

    // returns local source and local span (start, end)
    pub fn get_local_source<'a>(
        &'a self,
        span: &AstSpan,
    ) -> Result<(&'a String, (usize, usize)), MietteError> {
        let mut start = 0;
        let span_range = span.start()..(span.start() + span.len());

        for content in self.inner.iter() {
            let end = start + content.inner().len();

            // Check if the span lands in this file's global range
            if start <= span_range.start && span_range.end <= end {
                // Map to local span
                return Ok((content.inner(), (span.start() - start, span.end() - start)));
            }

            start = end;
        }

        Err(MietteError::OutOfBounds)
    }

    pub fn add_file(&mut self, source: String, file_name: String) {
        self.inner.push(NamedSourceFile::new(source, file_name));
    }
}

// part of impl from
// https://github.com/zkat/miette/issues/193#issuecomment-4262812575
impl SourceCode for MultiFileSource {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        let mut start = 0;
        let span_range = span.offset()..(span.offset() + span.len());

        for content in self.inner.iter() {
            let end = start + content.inner().len();

            // Check if the span lands in this file's global range
            if start <= span_range.start && span_range.end <= end {
                // Map to local span
                let local_span = SourceSpan::from((span.offset() - start, span.len()));
                // Get span
                let local_contents = content.inner().read_span(
                    &local_span,
                    context_lines_before,
                    context_lines_after,
                )?;

                // Remap to global span
                let global_span = SourceSpan::from((
                    start + local_contents.span().offset(),
                    local_contents.span().len(),
                ));

                return Ok(Box::new(MietteSpanContents::new_named(
                    content.file_name.clone(),
                    local_contents.data(),
                    global_span,
                    local_contents.line(),
                    local_contents.column(),
                    local_contents.line_count(),
                )));
            }

            start = end;
        }

        Err(MietteError::OutOfBounds)
    }
}

pub type Source = MultiFileSource;

// stores global span, indicates its source with source_idx
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct AstSpan {
    start: usize,
    end: usize,
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
            .field("range", &format!("{}..{}", self.start, self.end))
            .finish()
    }
}

impl AstSpan {
    pub fn new(local_start: usize, local_end: usize, source: &ParseContext) -> Self {
        let (start, end) = source
            .inner()
            .to_global_span(local_start, local_end, source.source_idx())
            .unwrap();
        Self { start, end }
    }

    pub fn from_span<'a>(span: Span<'a>, source: &ParseContext) -> Self {
        Self::new(span.start(), span.end(), source)
    }

    pub fn from_pair<R: RuleType>(pair: Pair<R>, source: &ParseContext) -> Self {
        Self::from_span(pair.as_span(), source)
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }

    // TODO: off by one errors somewhere?
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn set_span(&mut self, start: usize, end: usize) {
        self.start = start;
        self.end = end;
    }

    pub fn to_span<'a>(&'a self, source: &'a Source) -> Span<'a> {
        let (local_source, local_span) = source.get_local_source(self).unwrap();
        Span::new(local_source, local_span.0, local_span.1).unwrap()
    }

    pub fn to_miette_span(&self) -> SourceSpan {
        // already a global span
        SourceSpan::new(self.start().into(), self.len())
    }

    pub fn get_str<'a>(&self, source: &'a Source) -> &'a str {
        let (local_source, local_span) = source.get_local_source(self).unwrap();
        &local_source[local_span.0..local_span.1]
    }

    pub fn get_line_str<'a>(&self, source: &'a Source) -> &'a str {
        self.get_str(source).lines().next().unwrap()
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

    pub fn from_pair<R: RuleType>(inner: T, pair: Pair<R>, source: &ParseContext) -> Self {
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
    pub instruction: Option<Arc<opcode_gen::instructions::Instruction>>,
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
pub enum VariableVisibilityKind {
    Global,
    Local,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: String,
    pub visibility: VariableVisibilityKind,
    pub expr_kind: VariableExprKind,
    pub ty: Type,
}

impl Variable {
    pub fn new(
        name: String,
        visibility: VariableVisibilityKind,
        expr_kind: VariableExprKind,
        ty: Type,
    ) -> Self {
        Self {
            name,
            visibility,
            expr_kind,
            ty,
        }
    }
}

#[derive(Debug, Clone)]
pub enum VariableExprKind {
    Expr(AstNode<Expr>),
    Block(Vec<StatementNode>),
}

#[derive(Debug, Clone)]
pub enum ReturnKind {
    Expr(AstNode<Expr>),
    Block(Vec<StatementNode>),
}

#[derive(Debug, Clone)]
pub enum StatementKind {
    Function(Function),
    Directive(Directive),
    // generated by include directive, gets flattened/deleted eventually
    InlinedBlock(Vec<StatementNode>),
    Return(ReturnKind),
    // this kind of call is always a macro (not an expr) so not wrapped in type/value
    FunctionCall(FunctionCall),
    InlinedFunctionCall(InlinedFunctionCall),
    Label {
        name: String,
    },
    BlockLabel {
        name: String,
        body: Vec<StatementNode>,
    },
    Block {
        body: Vec<StatementNode>,
    },
    Instruction(AstInstruction),
    Variable(Variable),
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub params: Vec<AstNode<Expr>>,
}

impl FunctionCall {
    pub fn new(name: String, params: Vec<AstNode<Expr>>) -> Self {
        Self { name, params }
    }
}

#[derive(Debug, Clone)]
pub struct InlinedFunctionCall {
    pub name: String,
    pub params: Vec<AstNode<Expr>>,
    pub original_body: Vec<StatementNode>,
    pub body: Vec<StatementNode>,
}

impl InlinedFunctionCall {
    pub fn new(
        name: String,
        params: Vec<AstNode<Expr>>,
        original_body: Vec<StatementNode>,
        body: Vec<StatementNode>,
    ) -> Self {
        Self {
            name,
            params,
            original_body,
            body,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Directive {
    pub name: String,
    pub params: Vec<AstNode<Expr>>,
}

impl Directive {
    pub fn new(name: String, params: Vec<AstNode<Expr>>) -> Self {
        Self { name, params }
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<AstNode<TypedParameter>>,
    pub body: Vec<StatementNode>,
    pub return_ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionSignature {
    pub name: String,
    pub params: Vec<Type>,
}

impl Function {
    pub fn new(
        name: String,
        params: Vec<AstNode<TypedParameter>>,
        body: Vec<StatementNode>,
    ) -> Self {
        Function {
            name,
            params,
            body,
            return_ty: Type::Unknown,
        }
    }

    pub fn as_signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: self.name.clone(),
            params: self.params.iter().map(|e| e.inner.ty.clone()).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypedParameter {
    pub name: String,
    pub ty: Type,
}

impl TypedParameter {
    pub fn new(name: String, ty: Type) -> Self {
        Self { name, ty }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum ExprKind {
    /// Literal
    Literal,
    /// Identity (could be var, reg, etc.)
    Identity(String),
    FunctionCall(FunctionCall),

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
