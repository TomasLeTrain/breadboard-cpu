use std::{collections::HashMap, error::Error, fmt::Display, sync::Arc};

use miette::{Context, Diagnostic, LabeledSpan, NamedSource, Result, SourceSpan, miette};
use opcode_gen::instructions::{AddressRegister, Register};

use crate::{
    ast::{AstNode, AstSpan, BinaryOp, ExprKind, StatementKind, StatementNode, Expr, UnaryOp},
    types::Type,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EvalValue {
    Int(i32),
    Bool(bool),
    String(String),
    Character(u8),

    Register(Register),
    AddressRegister(AddressRegister),

    Addr(u16),
    Byte(u8),

    Unknown,
}

impl EvalValue {
    fn as_int(&self) -> Option<i32> {
        match self {
            EvalValue::Int(val) => Some(*val),
            EvalValue::Character(val) => Some(*val as i32),
            EvalValue::Addr(val) => Some(*val as i32),
            EvalValue::Byte(val) => Some(*val as i32),
            EvalValue::Register(_)
            | EvalValue::AddressRegister(_)
            | EvalValue::Unknown
            | EvalValue::Bool(_)
            | EvalValue::String(_) => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            EvalValue::Bool(val) => Some(*val),
            EvalValue::Int(_)
            | EvalValue::Character(_)
            | EvalValue::Addr(_)
            | EvalValue::Byte(_)
            | EvalValue::Register(_)
            | EvalValue::AddressRegister(_)
            | EvalValue::Unknown
            | EvalValue::String(_) => None,
        }
    }

    fn apply_int_binary_op(lhs: &Self, rhs: &Self, op: &BinaryOp) -> Self {
        let lhs = lhs.as_int().unwrap();
        let rhs = rhs.as_int().unwrap();

        Self::Int(match op {
            BinaryOp::Add => lhs + rhs,
            BinaryOp::Sub => lhs - rhs,
            BinaryOp::Mul => lhs * rhs,
            BinaryOp::Div => lhs / rhs,
            BinaryOp::Mod => lhs % rhs,
            BinaryOp::Pow => lhs.pow(rhs.try_into().unwrap()),
            BinaryOp::ShiftLeft => lhs << rhs,
            BinaryOp::ShiftRight => lhs >> rhs,
            BinaryOp::BitAnd => lhs & rhs,
            BinaryOp::BitXor => lhs ^ rhs,
            BinaryOp::BitOr => lhs | rhs,
            // TODO: turn into error
            _ => unreachable!(),
        })
    }

    fn apply_bool_binary_op(lhs: &Self, rhs: &Self, op: &BinaryOp) -> Self {
        let lhs = lhs.as_bool().unwrap();
        let rhs = rhs.as_bool().unwrap();

        Self::Bool(match op {
            BinaryOp::And => lhs && rhs,
            BinaryOp::Or => lhs || rhs,
            // TODO: turn into error
            _ => unreachable!(),
        })
    }

    fn apply_comparison_op(lhs: &Self, rhs: &Self, op: &BinaryOp) -> Self {
        // TODO: add support for other comparison types
        let lhs = lhs.as_int().unwrap();
        let rhs = rhs.as_int().unwrap();

        Self::Bool(match op {
            BinaryOp::Lt => lhs < rhs,
            BinaryOp::Gt => lhs > rhs,
            BinaryOp::Le => lhs <= rhs,
            BinaryOp::Ge => lhs >= rhs,
            // TODO: turn into error
            _ => unreachable!(),
        })
    }

    fn apply_equality_op(lhs: &Self, rhs: &Self, op: &BinaryOp) -> Self {
        // TODO: add support for other comparison types
        // TODO: make checking of both types as compatible more robust
        let equal = if lhs.as_int().is_some() {
            lhs.as_int().unwrap() == rhs.as_int().unwrap()
        } else if lhs.as_bool().is_some() {
            lhs.as_bool().unwrap() == rhs.as_bool().unwrap()
        } else {
            // TODO: make into error
            unreachable!();
        };

        Self::Bool(match op {
            BinaryOp::Eq => equal,
            BinaryOp::Ne => !equal,
            // TODO: turn into error
            _ => unreachable!(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EvalSymbol {
    pub name: String,
    pub symbol_type: Type,
    pub value: EvalValue,
    pub span: Option<AstSpan>,
}

// keeps track of symbols by keeping track of their scope as well
// allows reusing one context struct through all operations
pub struct EvalContext {
    symbol_stack: Vec<EvalSymbol>,
    symbols: HashMap<String, EvalSymbol>,
}

impl EvalContext {
    pub fn new() -> Self {
        EvalContext {
            symbol_stack: Vec::new(),
            symbols: HashMap::new(),
        }
    }

    pub fn push(&mut self, symbol: EvalSymbol) -> Result<()> {
        self.symbol_stack.push(symbol.clone());

        let push_result = self.symbols.insert(symbol.clone().name, symbol.clone());

        if let Some(other) = push_result {
            // let source = symbol.span.as_ref().map(|e| e.to_miette_source_code());
            // let current_symbol_span = symbol.span.as_ref().map(AstSpan::to_miette_span);
            // let other_symbol_span = other.span.as_ref().map(AstSpan::to_miette_span);
            Err(miette!("duplicate symbol error"))?

            // Err(DuplicateSymbolError {
            //     name: symbol.name,
            //     type1: other.symbol_type,
            //     type2: symbol.symbol_type,
            //     source,
            //     current_symbol_span,
            //     other_symbol_span,
            // })?
        } else {
            Ok(())
        }
    }

    fn pop(&mut self) -> Result<EvalSymbol> {
        let popped_symbol = self
            .symbol_stack
            .pop()
            .ok_or(miette!("empty stack error"))?;

        let map_symbol = self.symbols.remove(&popped_symbol.name).unwrap();

        Ok(map_symbol)
    }

    fn get(&self, name: &String) -> Option<&EvalSymbol> {
        self.symbols.get(name)
    }

    fn contains(&self, name: &String) -> bool {
        self.symbols.contains_key(name)
    }
}

pub fn eval_program(statements: &mut [StatementNode], ctx: &mut EvalContext) -> Result<()> {
    let mut labels = Vec::new();

    // first find all labels in the current scope (accessible from anywhere in scope)
    for statement in statements.iter() {
        if let StatementKind::Label { name } | StatementKind::BlockLabel { name, .. } =
            statement.inner().inner()
        {
            // push into local scope
            let curr_symbol = EvalSymbol {
                name: name.clone(),
                symbol_type: Type::Label,
                value: EvalValue::Addr(statement.inner().address().unwrap()),
                span: Some(statement.span().clone()),
            };

            ctx.push(curr_symbol.clone())
                .wrap_err("Pushing local label symbol failed.")?;

            labels.push(curr_symbol);
        }
    }

    for statement in statements.iter_mut() {
        match statement.inner_mut().inner_mut() {
            StatementKind::BlockLabel { body, .. } => {
                eval_program(body, ctx)?;
            }
            StatementKind::Instruction(instruction) => {
                for param in instruction.params.iter_mut() {
                    eval_expr(param, ctx)?;
                }
            }
            _ => (),
        };
    }

    // checks that all returned symbols match what was pushed in
    // goes in reverse since pop starts from the last added element
    for label in labels.into_iter().rev() {
        let curr = ctx.pop()?;
        if label != curr {
            return Err(miette!(
                "Popped symbol does not match - original: {:?}, got: {:?}",
                label,
                curr,
            ));
        }
    }

    Ok(())
}

// TODO: make sure int type gets casted to whatever type current expr is (say addr or byte)
fn eval_expr(typed_expr: &mut AstNode<Expr>, ctx: &mut EvalContext) -> Result<()> {
    let inner_span = typed_expr.span().clone();
    let inner = typed_expr.inner_mut();

    match &mut inner.kind {
        // literals already have their value filled in
        ExprKind::Literal => (),
        ExprKind::Identity(name) => {
            // try and find identity in symbols
            if ctx.contains(name) {
                let symbol = ctx.get(name).unwrap();

                if inner.ty != symbol.symbol_type {
                    // Err(EvalExprError::new(
                    //     TypecheckExprErrorKind::IdentityAlreadyTyped((inner_span, inner.ty)),
                    // ))?;
                }

                inner.value = symbol.value.clone();
            } else {
                Err(EvalExprError::new(TypecheckExprErrorKind::SymbolNotFound(
                    EvalSymbol {
                        name: name.to_string(),
                        symbol_type: inner.ty,
                        span: Some(typed_expr.span.clone()),
                        value: EvalValue::Unknown,
                    },
                )))?;
            }
        }
        ExprKind::Unary {
            op,
            expr: unary_expr,
        } => {
            eval_expr(unary_expr, ctx)?;
            // let span = unary_expr.span().clone();
            let unary_expr = unary_expr.inner_mut();

            match op {
                UnaryOp::Neg => {
                    inner.value = EvalValue::Int(-unary_expr.value.as_int().unwrap());
                }
                UnaryOp::BitNegation => {
                    inner.value = EvalValue::Int(!unary_expr.value.as_int().unwrap());
                }
                UnaryOp::Not => {
                    inner.value = EvalValue::Bool(!unary_expr.value.as_bool().unwrap());
                }
            }
        }
        ExprKind::Binary { op, left, right } => {
            eval_expr(left, ctx)?;
            eval_expr(right, ctx)?;

            // let left_span = left.span().clone();
            // let right_span = right.span().clone();

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
                    inner.value = EvalValue::apply_int_binary_op(&left.value, &right.value, op);
                }
                BinaryOp::And | BinaryOp::Or => {
                    inner.value = EvalValue::apply_bool_binary_op(&left.value, &right.value, op);
                }
                BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                    inner.value = EvalValue::apply_comparison_op(&left.value, &right.value, op);
                }
                BinaryOp::Eq | BinaryOp::Ne => {
                    inner.value = EvalValue::apply_equality_op(&left.value, &right.value, op);
                }
            }
        }
    }
    Ok(())
}

#[derive(Diagnostic, Debug)]
pub struct DuplicateSymbolError {
    name: String,
    type1: Type,
    type2: Type,

    #[source_code]
    source: Option<NamedSource<Arc<str>>>,

    #[label(primary, "Current symbol defined here")]
    current_symbol_span: Option<SourceSpan>,

    #[label("Other symbol defined here")]
    other_symbol_span: Option<SourceSpan>,
}

#[derive(Debug)]
pub struct EmptyStackError;

impl Error for DuplicateSymbolError {}
impl Error for EmptyStackError {}

impl Display for DuplicateSymbolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Symbol already in context - name: {:?}, type1: {:?}, type2: {:?}",
            self.name, self.type1, self.type2
        )
    }
}

impl Display for EmptyStackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No symbols in stack.")
    }
}

#[derive(Debug)]
pub enum TypecheckExprErrorKind {
    IdentityAlreadyTyped((AstSpan, Type)),
    SymbolNotFound(EvalSymbol),
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
pub struct EvalExprError {
    #[source_code]
    source: Option<NamedSource<Arc<str>>>,
    kind: TypecheckExprErrorKind,

    #[label(collection, "Defined here")]
    spans: Vec<LabeledSpan>,
}

impl Error for EvalExprError {}

impl Display for EvalExprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl EvalExprError {
    fn new(kind: TypecheckExprErrorKind) -> Self {
        let spans = kind.get_spans();
        let source = kind.get_source();

        EvalExprError {
            spans,
            source,
            kind,
        }
    }
}
