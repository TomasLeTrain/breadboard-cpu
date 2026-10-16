use std::fmt::Debug;
use std::sync::Arc;

use crate::eval::ExprValue;
use crate::types::{Address, Type};

use miette::{MietteError, MietteSpanContents, NamedSource, SourceCode, SourceSpan, SpanContents};
use pest::{RuleType, Span, iterators::Pair};

pub type Ast = Vec<StatementNode>;
pub type StatementNode = AstNode<Statement>;

/// cannot use NamedSource since it does not expose a &str, which is needed for converting span to pest (used for reusing pest errors)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedSourceFile {
    source: String,
    file_name: String,
}

impl NamedSourceFile {
    pub fn new(source: String, file_name: String) -> Self {
        Self {
            // source: Arc::from(source),
            source,
            file_name,
        }
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
pub struct MultiFileSource(Vec<NamedSourceFile>);

impl MultiFileSource {
    pub fn new(named_source_files: Vec<NamedSourceFile>) -> Self {
        Self(named_source_files)
    }

    fn to_local_span<'a>(
        &'a self,
        span: &AstSpan,
    ) -> Result<(&'a String, (usize, usize)), MietteError> {
        let mut start = 0;
        let span_range = span.start()..(span.start() + span.len());

        for content in self.0.iter() {
            let end = start + content.inner().len();

            // Check if the span lands in this file's global range
            if start <= span_range.start && span_range.end <= end {
                let local_start = span.start() - start;
                let local_end = local_start + span.len();
                return Ok((content.inner(), (local_start, local_end)));
            }

            start = end;
        }

        Err(MietteError::OutOfBounds)
    }

    pub fn add_file(&mut self, source: String, file_name: String) {
        self.0.push(NamedSourceFile::new(source, file_name));
    }

    // appends all sources into one string
    // pub fn whole_source(&self) -> String {
    //     let mut result = String::new();
    //
    //     for NamedSourceFile { source, .. } in &self.0 {
    //         result.push_str(source);
    //     }
    //
    //     result
    // }
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

        for content in self.0.iter() {
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
                    local_contents.name().unwrap_or_default().to_string(),
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

// pub type Source = Arc<MultiFileSource>;
pub type Source = MultiFileSource;

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
            source: source.clone(),
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

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn set_span(&mut self, start: usize, end: usize) {
        self.start = start;
        self.end = end;
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    pub fn to_span<'a>(&'a self) -> Span<'a> {
        let (local_source, local_span) = self.source.to_local_span(self).unwrap();

        Span::new(local_source, local_span.0, local_span.1).unwrap()
    }

    pub fn to_miette_span(&self) -> SourceSpan {
        SourceSpan::new(self.start().into(), self.len())
    }

    pub fn to_miette_source_code(&self) -> MultiFileSource {
        // (*self.source).clone()
        self.source().clone()
    }

    pub fn get_str(&self) -> &str {
        let (local_source, local_span) = self.source.to_local_span(self).unwrap();
        &local_source[local_span.0..local_span.1]
    }

    pub fn get_line_str(&self) -> &str {
        self.get_str().lines().next().unwrap()
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

    pub fn into_inner(self) -> T {
        self.inner
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

    pub fn into_inner(self) -> StatementKind {
        self.statement
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
pub struct Variable {
    pub name: String,
    pub expr_kind: VariableExprKind,
    pub ty: Type,
}

impl Variable {
    pub fn new(name: String, expr_kind: VariableExprKind, ty: Type) -> Self {
        Self {
            name,
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
