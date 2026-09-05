use std::{collections::HashMap, error::Error, fmt::Display, sync::Arc};

use crate::ast::{
    AstNode, AstSpan, BinaryOp, Expr, ExprKind, FunctionCall, ReturnKind, StatementKind,
    StatementNode, UnaryOp,
};
use miette::{Context, Diagnostic, IntoDiagnostic, LabeledSpan, NamedSource, Result, miette};

pub type Address = u16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Addr,
    Byte,

    Bool,
    String,

    Register,
    AddressRegister,

    Label,
    Function,

    Block,

    Unknown,
}

impl Type {
    pub fn int_operable(&self) -> bool {
        matches!(self, Type::Int | Type::Label | Type::Byte | Type::Addr)
    }

    pub fn bool_operable(&self) -> bool {
        matches!(self, Type::Bool)
    }

    // returns true if types are comparable to each other
    pub fn comparable(lhs: &Type, rhs: &Type) -> bool {
        Self::int_binary_operable(lhs, rhs) || Self::bool_binary_operable(lhs, rhs)
    }

    pub fn int_binary_operable(lhs: &Type, rhs: &Type) -> bool {
        lhs.int_operable() && rhs.int_operable()
    }

    pub fn bool_binary_operable(lhs: &Type, rhs: &Type) -> bool {
        lhs.bool_operable() && rhs.bool_operable()
    }

    pub fn unify(&self, other: &Type) -> Option<Type> {
        if Self::int_binary_operable(self, other) {
            Some(Type::Int)
        } else if Self::bool_binary_operable(self, other) {
            Some(Type::Bool)
        } else if let (Type::Unknown, t) | (t, Type::Unknown) = (self, other) {
            Some(*t)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: Type,
    pub span: Option<AstSpan>,
}

// keeps track of symbols by keeping track of their scope as well
// allows reusing one context struct through all operations
pub struct SymbolTypeContext {
    symbol_stack: Vec<Symbol>,
    symbols: HashMap<String, Symbol>,
}

impl SymbolTypeContext {
    pub fn new() -> Self {
        SymbolTypeContext {
            symbol_stack: Vec::new(),
            symbols: HashMap::new(),
        }
    }

    pub fn push(&mut self, symbol: Symbol) -> Result<()> {
        self.symbol_stack.push(symbol.clone());

        let push_result = self.symbols.insert(symbol.clone().name, symbol.clone());

        if let Some(other) = push_result {
            let mut spans = Vec::new();

            let source = symbol.span.as_ref().map(|e| e.to_miette_source_code());

            if let Some(ast_span) = symbol.span.as_ref() {
                spans.push(LabeledSpan::new_with_span(
                    Some(format!(
                        "Symbol of type \"{:?}\" defined here",
                        symbol.symbol_type
                    )),
                    ast_span.to_miette_span(),
                ));
            };

            if let Some(ast_span) = other.span.as_ref() {
                spans.push(LabeledSpan::new_with_span(
                    Some(format!(
                        "Symbol of type \"{:?}\" defined here",
                        other.symbol_type
                    )),
                    ast_span.to_miette_span(),
                ));
            };

            Err(DuplicateSymbolError {
                name: symbol.name,
                source,
                spans,
            })?
        } else {
            Ok(())
        }
    }

    fn pop(&mut self) -> Result<Symbol> {
        let popped_symbol = self
            .symbol_stack
            .pop()
            .ok_or(EmptyStackError {})
            .into_diagnostic()?;

        let map_symbol = self.symbols.remove(&popped_symbol.name).unwrap();

        Ok(map_symbol)
    }

    fn get(&self, name: &String) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    fn contains(&self, name: &String) -> bool {
        self.symbols.contains_key(name)
    }
}

pub fn typecheck(statements: &mut [StatementNode], symbols: &mut SymbolTypeContext) -> Result<()> {
    let mut labels = Vec::new();

    // must first typecheck functions to get their return type if not specified
    // only then can its symbol be constructed
    for statement in statements.iter_mut() {
        if let StatementKind::Function(function) = statement.inner_mut().inner_mut() {
            // first typecheck body
            typecheck(&mut function.body, symbols)?;

            // then find type of return statement
            let return_statement = function
                .body
                .iter()
                .find(|e| matches!(e.inner().inner(), StatementKind::Return { .. }));

            if let Some(statement) = return_statement {
                if let StatementKind::Return(return_kind) = statement.inner().inner() {
                    function.return_type = match return_kind {
                        ReturnKind::Expr(expr) => expr.inner().ty,
                        ReturnKind::Block(_) => Type::Block,
                    };
                } else {
                    unreachable!()
                }
            } else {
                // TODO: turn into detailed error
                return Err(miette!("No return statement inside function!"));
            }
        }
    }

    // first find all labels and functions in the current scope (accessible from anywhere in scope)
    for statement in statements.iter() {
        match statement.inner().inner() {
            StatementKind::Label { name } | StatementKind::BlockLabel { name, .. } => {
                // push into local scope
                let curr_symbol = Symbol {
                    name: name.clone(),
                    symbol_type: Type::Label,
                    span: Some(statement.span().clone()),
                };

                symbols
                    .push(curr_symbol.clone())
                    .wrap_err("Pushing local label symbol failed.")?;

                labels.push(curr_symbol);
            }
            StatementKind::Function(function) => {
                // push into local scope
                let curr_symbol = Symbol {
                    name: function.name.clone(),
                    symbol_type: function.return_type,
                    span: Some(statement.span().clone()),
                };

                symbols
                    .push(curr_symbol.clone())
                    .wrap_err("Pushing function symbol failed.")?;

                labels.push(curr_symbol);
            }
            _ => (),
        }
    }

    for statement in statements.iter_mut() {
        // NOTE: functions were already typechecked
        match statement.inner_mut().inner_mut() {
            StatementKind::BlockLabel { body, .. } | StatementKind::Block { body } => {
                typecheck(body, symbols)?;
            }
            StatementKind::Return(kind) => match kind {
                ReturnKind::Expr(expr) => typecheck_expr(expr, symbols)?,
                ReturnKind::Block(block) => typecheck(block, symbols)?,
            },
            StatementKind::Instruction(instruction) => {
                for param in instruction.params.iter_mut() {
                    typecheck_expr(param, symbols)?;
                }
            }
            _ => (),
        };
    }

    // Checks that all returned symbols match what was pushed in.
    // Goes in reverse since pop starts from the last added element
    for label in labels.into_iter().rev() {
        let curr = symbols.pop()?;
        if label != curr {
            // TODO: make specific type for error with more details
            return Err(miette!(
                "Popped symbol does not match - original: {:?}, got: {:?}",
                label,
                curr,
            ));
        }
    }

    Ok(())
}

fn typecheck_expr(typed_expr: &mut AstNode<Expr>, symbols: &SymbolTypeContext) -> Result<()> {
    let inner_span = typed_expr.span().clone();
    let inner = typed_expr.inner_mut();

    match &mut inner.kind {
        // literals already have their typed filled in
        ExprKind::Literal => (),
        // NOTE: assumes unique function names
        ExprKind::FunctionCall(FunctionCall { name, .. }) | ExprKind::Identity(name) => {
            // try and find identity in symbols
            if symbols.contains(name) {
                if !matches!(inner.ty, Type::Unknown) {
                    Err(TypecheckExprError::new(
                        TypecheckExprErrorKind::IdentityAlreadyTyped((inner_span, inner.ty)),
                    ))?;
                }

                inner.ty = symbols.get(name).unwrap().symbol_type;
            } else {
                Err(TypecheckExprError::new(
                    TypecheckExprErrorKind::SymbolNotFound(Symbol {
                        name: name.to_string(),
                        symbol_type: inner.ty,
                        span: Some(typed_expr.span.clone()),
                    }),
                ))?;
            }
        }
        ExprKind::Unary {
            op,
            expr: unary_expr,
        } => {
            typecheck_expr(unary_expr, symbols)?;
            let span = unary_expr.span().clone();
            let unary_expr = unary_expr.inner_mut();

            match op {
                UnaryOp::Neg | UnaryOp::BitNegation => {
                    if !unary_expr.ty.int_operable() {
                        Err(TypecheckExprError::new(
                            TypecheckExprErrorKind::InvalidUnaryOpType((span, unary_expr.ty), *op),
                        ))?;
                    }
                    inner.ty = Type::Int;
                }
                UnaryOp::Not => {
                    if !unary_expr.ty.bool_operable() {
                        Err(TypecheckExprError::new(
                            TypecheckExprErrorKind::InvalidUnaryOpType((span, unary_expr.ty), *op),
                        ))?;
                    }
                    inner.ty = Type::Bool;
                }
            }
        }
        ExprKind::Binary { op, left, right } => {
            typecheck_expr(left, symbols)?;
            typecheck_expr(right, symbols)?;

            let left_span = left.span().clone();
            let right_span = right.span().clone();

            let left = left.inner_mut();
            let right = right.inner_mut();

            match op {
                BinaryOp::Add
                | BinaryOp::Sub
                | BinaryOp::Mul
                | BinaryOp::Div
                | BinaryOp::Mod
                | BinaryOp::Pow
                | BinaryOp::ShiftLeft
                | BinaryOp::ShiftRight
                | BinaryOp::BitAnd
                | BinaryOp::BitXor
                | BinaryOp::BitOr => {
                    if !Type::int_binary_operable(&left.ty, &right.ty) {
                        Err(TypecheckExprError::new(
                            TypecheckExprErrorKind::InvalidBinaryOpTypes(
                                (left_span, left.ty),
                                (right_span, right.ty),
                                *op,
                            ),
                        ))?;
                    }
                    inner.ty = left.ty.unify(&right.ty).unwrap();
                }
                BinaryOp::And | BinaryOp::Or => {
                    if !Type::bool_binary_operable(&left.ty, &right.ty) {
                        Err(TypecheckExprError::new(
                            TypecheckExprErrorKind::InvalidBinaryOpTypes(
                                (left_span, left.ty),
                                (right_span, right.ty),
                                *op,
                            ),
                        ))?;
                    }
                    inner.ty = left.ty.unify(&right.ty).unwrap();
                }
                BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                    if !Type::comparable(&left.ty, &right.ty) {
                        Err(TypecheckExprError::new(
                            TypecheckExprErrorKind::InvalidComparisonTypes(
                                (left_span, left.ty),
                                (right_span, right.ty),
                            ),
                        ))?;
                    }
                    inner.ty = Type::Bool;
                }
                BinaryOp::Eq | BinaryOp::Ne => {
                    if left.ty.unify(&right.ty).is_none() {
                        Err(TypecheckExprError::new(
                            TypecheckExprErrorKind::InvalidEqualityTypes(
                                (left_span, left.ty),
                                (right_span, right.ty),
                            ),
                        ))?;
                    }
                    inner.ty = Type::Bool;
                }
            }
        }
    }
    Ok(())
}

#[derive(Diagnostic, Debug)]
#[diagnostic(code(eval::duplicate_symbol))]
pub struct DuplicateSymbolError {
    name: String,

    #[source_code]
    source: Option<NamedSource<Arc<str>>>,

    #[label(collection, "Defined here")]
    spans: Vec<LabeledSpan>,
}

impl Error for DuplicateSymbolError {}

impl Display for DuplicateSymbolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Symbol \"{}\" already in context", self.name)
    }
}

#[derive(Debug)]
pub struct EmptyStackError;

impl Error for EmptyStackError {}

impl Display for EmptyStackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No symbols in stack.")
    }
}

#[derive(Debug)]
pub enum TypecheckExprErrorKind {
    IdentityAlreadyTyped((AstSpan, Type)),
    SymbolNotFound(Symbol),
    InvalidBinaryOpTypes((AstSpan, Type), (AstSpan, Type), BinaryOp),
    InvalidComparisonTypes((AstSpan, Type), (AstSpan, Type)),
    InvalidEqualityTypes((AstSpan, Type), (AstSpan, Type)),
    InvalidUnaryOpType((AstSpan, Type), UnaryOp),
}

impl TypecheckExprErrorKind {
    fn get_spans(&self) -> Vec<LabeledSpan> {
        match self {
            TypecheckExprErrorKind::IdentityAlreadyTyped((span, ty)) => {
                vec![LabeledSpan::new_with_span(
                    Some(format!("Identity of type \"{:?}\" defined here", ty)),
                    span,
                )]
            }
            TypecheckExprErrorKind::SymbolNotFound(symbol) => {
                if let Some(span) = &symbol.span {
                    vec![LabeledSpan::new_with_span(
                        Some("Symbol defined here".to_string()),
                        span,
                    )]
                } else {
                    vec![]
                }
            }
            TypecheckExprErrorKind::InvalidBinaryOpTypes((span1, ty1), (span2, ty2), _)
            | TypecheckExprErrorKind::InvalidEqualityTypes((span1, ty1), (span2, ty2))
            | TypecheckExprErrorKind::InvalidComparisonTypes((span1, ty1), (span2, ty2)) => {
                vec![
                    LabeledSpan::new_with_span(
                        Some(format!("Defined with type \"{:?}\" here", ty1)),
                        span1,
                    ),
                    LabeledSpan::new_with_span(
                        Some(format!("Defined with type \"{:?}\" here", ty2)),
                        span2,
                    ),
                ]
            }
            TypecheckExprErrorKind::InvalidUnaryOpType((span, ty), _) => {
                vec![LabeledSpan::new_with_span(
                    Some(format!("Defined with type \"{:?}\" here", ty)),
                    span,
                )]
            }
        }
    }

    fn get_source(&self) -> Option<NamedSource<Arc<str>>> {
        match self {
            TypecheckExprErrorKind::InvalidBinaryOpTypes((ast_span, _), _, _)
            | TypecheckExprErrorKind::InvalidComparisonTypes((ast_span, _), _)
            | TypecheckExprErrorKind::InvalidEqualityTypes((ast_span, _), _)
            | TypecheckExprErrorKind::InvalidUnaryOpType((ast_span, _), _)
            | TypecheckExprErrorKind::IdentityAlreadyTyped((ast_span, _)) => {
                Some(ast_span.to_miette_source_code())
            }
            TypecheckExprErrorKind::SymbolNotFound(symbol) => {
                symbol.span.as_ref().map(AstSpan::to_miette_source_code)
            }
        }
    }
}

impl Display for TypecheckExprErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypecheckExprErrorKind::IdentityAlreadyTyped((_, ty)) => {
                write!(f, "Identity already has type \"{:?}\"", ty)
            }
            TypecheckExprErrorKind::SymbolNotFound(symbol) => {
                write!(f, "Symbol \"{}\" not found", symbol.name)
            }
            TypecheckExprErrorKind::InvalidBinaryOpTypes((_, ty1), (_, ty2), op) => {
                write!(
                    f,
                    "Cannot perform operation \"{:?}\" on types \"{:?}\" and \"{:?}\"",
                    op, ty1, ty2
                )
            }
            TypecheckExprErrorKind::InvalidComparisonTypes((_, ty1), (_, ty2)) => {
                write!(f, "Cannot compare types \"{:?}\" and \"{:?}\"", ty1, ty2)
            }
            TypecheckExprErrorKind::InvalidEqualityTypes((_, ty1), (_, ty2)) => {
                write!(
                    f,
                    "Cannot check types for equality \"{:?}\" and \"{:?}\"",
                    ty1, ty2
                )
            }
            TypecheckExprErrorKind::InvalidUnaryOpType((_, ty), op) => {
                write!(
                    f,
                    "Cannot perform operation \"{:?}\" on type \"{:?}\"",
                    op, ty
                )
            }
        }
    }
}

#[derive(Diagnostic, Debug)]
pub struct TypecheckExprError {
    #[source_code]
    source: Option<NamedSource<Arc<str>>>,
    kind: TypecheckExprErrorKind,

    #[label(collection, "Defined here")]
    spans: Vec<LabeledSpan>,
}

impl Error for TypecheckExprError {}

impl Display for TypecheckExprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl TypecheckExprError {
    fn new(kind: TypecheckExprErrorKind) -> Self {
        let spans = kind.get_spans();
        let source = kind.get_source();

        TypecheckExprError {
            spans,
            source,
            kind,
        }
    }
}
