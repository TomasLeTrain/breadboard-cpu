use std::{collections::HashMap, error::Error, fmt::Display, sync::Arc};

use miette::{Context, Diagnostic, IntoDiagnostic, LabeledSpan, NamedSource, Result, miette};
use opcode_gen::instructions::{AddressRegister, ArgumentValue, Register};

use crate::{
    ast::{AstNode, AstSpan, BinaryOp, Expr, ExprKind, StatementKind, StatementNode, UnaryOp},
    types::{Address, Type},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExprValue {
    Int(i32),
    Bool(bool),
    String(String),

    Register(Register),
    AddressRegister(AddressRegister),

    Addr(Address),
    Byte(u8),

    Unknown,
}

impl ExprValue {
    fn as_int(&self) -> Option<i32> {
        match self {
            ExprValue::Int(val) => Some(*val),
            ExprValue::Addr(val) => Some(*val as i32),
            ExprValue::Byte(val) => Some(*val as i32),
            ExprValue::Register(_)
            | ExprValue::AddressRegister(_)
            | ExprValue::Unknown
            | ExprValue::Bool(_)
            | ExprValue::String(_) => None,
        }
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            ExprValue::Bool(val) => Some(*val),
            ExprValue::Int(_)
            | ExprValue::Addr(_)
            | ExprValue::Byte(_)
            | ExprValue::Register(_)
            | ExprValue::AddressRegister(_)
            | ExprValue::Unknown
            | ExprValue::String(_) => None,
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

    fn cast_value(self, ty: &Type) -> Self {
        match ty {
            Type::Bool => {
                assert!(matches!(self, Self::Bool(_)));
                self
            }

            Type::Int => {
                // casting from any int-able value
                Self::Int(self.as_int().unwrap())
            }
            Type::Addr => {
                // casting from any int-able value
                Self::Addr(self.as_int().unwrap().try_into().unwrap())
            }
            Type::Byte => {
                // casting from any int-able value
                Self::Byte(self.as_int().unwrap().try_into().unwrap())
            }

            Type::Label => {
                assert!(matches!(self, Self::Addr(_)));
                self
            }

            Type::Register => {
                assert!(matches!(self, Self::Register(_)));
                self
            }
            Type::AddressRegister => {
                assert!(matches!(self, Self::AddressRegister(_)));
                self
            }

            Type::String => {
                assert!(matches!(self, Self::String(_)));
                self
            }

            Type::Unknown => unreachable!(),
        }
    }

    pub fn as_istr_arg_value(&self) -> ArgumentValue {
        match self {
            ExprValue::Register(register) => ArgumentValue::Reg(*register),
            ExprValue::AddressRegister(address_register) => {
                ArgumentValue::AddrReg(*address_register)
            }
            ExprValue::Addr(addr) => ArgumentValue::Addr(*addr),
            ExprValue::Byte(byte) => ArgumentValue::Byte(*byte),

            // TODO: add errors
            ExprValue::Unknown => panic!(),
            ExprValue::Int(_) => panic!(),
            ExprValue::Bool(_) => panic!(),
            ExprValue::String(_) => panic!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EvalSymbol {
    pub name: String,
    pub symbol_type: Type,
    pub value: ExprValue,
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
            let mut spans = Vec::new();
            let source = symbol.span.as_ref().map(|e| e.to_miette_source_code());

            if let Some(ast_span) = symbol.span.as_ref() {
                spans.push(LabeledSpan::new_with_span(
                    Some(format!(
                        "Symbol of type \"{:?}\" with value \"{:?}\" defined here",
                        symbol.symbol_type, symbol.value
                    )),
                    ast_span.to_miette_span(),
                ));
            };
            if let Some(ast_span) = other.span.as_ref() {
                spans.push(LabeledSpan::new_with_span(
                    Some(format!(
                        "Symbol of type \"{:?}\" with value \"{:?}\" defined here",
                        other.symbol_type, other.value
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

    fn pop(&mut self) -> Result<EvalSymbol> {
        let popped_symbol = self
            .symbol_stack
            .pop()
            .ok_or(EmptyStackError {})
            .into_diagnostic()?;

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
                value: ExprValue::Addr(statement.inner().address().unwrap()),
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

                // TODO: already valued error
                if inner.ty != symbol.symbol_type {
                    // Err(EvalExprError::new(
                    //     TypecheckExprErrorKind::IdentityAlreadyTyped((inner_span, inner.ty)),
                    // ))?;
                }

                inner.value = symbol.value.clone().cast_value(&inner.ty);
            } else {
                Err(EvalExprError::new(EvalExprErrorKind::SymbolNotFound(
                    EvalSymbol {
                        name: name.to_string(),
                        symbol_type: inner.ty,
                        span: Some(inner_span),
                        value: ExprValue::Unknown,
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

            inner.value = match op {
                UnaryOp::Neg => ExprValue::Int(-unary_expr.value.as_int().unwrap()),
                UnaryOp::BitNegation => ExprValue::Int(!unary_expr.value.as_int().unwrap()),
                UnaryOp::Not => ExprValue::Bool(!unary_expr.value.as_bool().unwrap()),
            };
        }
        ExprKind::Binary { op, left, right } => {
            eval_expr(left, ctx)?;
            eval_expr(right, ctx)?;

            // let left_span = left.span().clone();
            // let right_span = right.span().clone();

            let left = left.inner_mut();
            let right = right.inner_mut();

            inner.value = match op {
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
                | BinaryOp::BitOr => ExprValue::apply_int_binary_op(&left.value, &right.value, op),
                BinaryOp::And | BinaryOp::Or => {
                    ExprValue::apply_bool_binary_op(&left.value, &right.value, op)
                }
                BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                    ExprValue::apply_comparison_op(&left.value, &right.value, op)
                }
                BinaryOp::Eq | BinaryOp::Ne => {
                    ExprValue::apply_equality_op(&left.value, &right.value, op)
                }
            }
        }
    }

    // ensure value confines to the type
    inner.value = inner.value.clone().cast_value(&inner.ty);

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
pub enum EvalExprErrorKind {
    IdentityAlreadyTyped((AstSpan, Type)),
    SymbolNotFound(EvalSymbol),
    InvalidBinaryOpTypes((AstSpan, Type), (AstSpan, Type), BinaryOp),
    InvalidComparisonTypes((AstSpan, Type), (AstSpan, Type)),
    InvalidEqualityTypes((AstSpan, Type), (AstSpan, Type)),
    InvalidUnaryOpType((AstSpan, Type), UnaryOp),
}

impl EvalExprErrorKind {
    fn get_spans(&self) -> Vec<LabeledSpan> {
        match self {
            EvalExprErrorKind::IdentityAlreadyTyped((span, ty)) => {
                vec![LabeledSpan::new_with_span(
                    Some(format!("Identity of type \"{:?}\" defined here", ty)),
                    span,
                )]
            }
            EvalExprErrorKind::SymbolNotFound(symbol) => {
                if let Some(span) = &symbol.span {
                    vec![LabeledSpan::new_with_span(
                        Some("Symbol defined here".to_string()),
                        span,
                    )]
                } else {
                    vec![]
                }
            }
            EvalExprErrorKind::InvalidBinaryOpTypes((span1, ty1), (span2, ty2), _)
            | EvalExprErrorKind::InvalidEqualityTypes((span1, ty1), (span2, ty2))
            | EvalExprErrorKind::InvalidComparisonTypes((span1, ty1), (span2, ty2)) => {
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
            EvalExprErrorKind::InvalidUnaryOpType((span, ty), _) => {
                vec![LabeledSpan::new_with_span(
                    Some(format!("Defined with type \"{:?}\" here", ty)),
                    span,
                )]
            }
        }
    }

    fn get_source(&self) -> Option<NamedSource<Arc<str>>> {
        match self {
            EvalExprErrorKind::InvalidBinaryOpTypes((ast_span, _), _, _)
            | EvalExprErrorKind::InvalidComparisonTypes((ast_span, _), _)
            | EvalExprErrorKind::InvalidEqualityTypes((ast_span, _), _)
            | EvalExprErrorKind::InvalidUnaryOpType((ast_span, _), _)
            | EvalExprErrorKind::IdentityAlreadyTyped((ast_span, _)) => {
                Some(ast_span.to_miette_source_code())
            }
            EvalExprErrorKind::SymbolNotFound(symbol) => {
                symbol.span.as_ref().map(AstSpan::to_miette_source_code)
            }
        }
    }
}

impl Display for EvalExprErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalExprErrorKind::IdentityAlreadyTyped((_, ty)) => {
                write!(f, "Identity already has type \"{:?}\"", ty)
            }
            EvalExprErrorKind::SymbolNotFound(symbol) => {
                write!(f, "Symbol \"{}\" not found", symbol.name)
            }
            EvalExprErrorKind::InvalidBinaryOpTypes((_, ty1), (_, ty2), op) => {
                write!(
                    f,
                    "Cannot perform operation \"{:?}\" on types \"{:?}\" and \"{:?}\"",
                    op, ty1, ty2
                )
            }
            EvalExprErrorKind::InvalidComparisonTypes((_, ty1), (_, ty2)) => {
                write!(f, "Cannot compare types \"{:?}\" and \"{:?}\"", ty1, ty2)
            }
            EvalExprErrorKind::InvalidEqualityTypes((_, ty1), (_, ty2)) => {
                write!(
                    f,
                    "Cannot check types for equality \"{:?}\" and \"{:?}\"",
                    ty1, ty2
                )
            }
            EvalExprErrorKind::InvalidUnaryOpType((_, ty), op) => {
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
    kind: EvalExprErrorKind,

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
    fn new(kind: EvalExprErrorKind) -> Self {
        let spans = kind.get_spans();
        let source = kind.get_source();

        EvalExprError {
            spans,
            source,
            kind,
        }
    }
}
