use std::{collections::HashMap, error::Error, fmt::Display, hash::Hash, sync::Arc};

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
pub struct Symbol<T> {
    pub name: String,
    pub span: Option<AstSpan>,
    pub data: T,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LabelSymbolData {
    pub ty: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionSymbolData {
    pub signature: FunctionSignature,
}

trait SymbolAsKey {
    type Key: Eq + Hash + Clone;
    fn as_key(&self) -> &Self::Key;
}

impl SymbolAsKey for Symbol<LabelSymbolData> {
    type Key = String;

    fn as_key(&self) -> &String {
        &self.name
    }
}

impl SymbolAsKey for Symbol<FunctionSymbolData> {
    type Key = FunctionSignature;

    fn as_key(&self) -> &FunctionSignature {
        &self.data.signature
    }
}

type SymbolKey<T> = <Symbol<T> as SymbolAsKey>::Key;

pub struct SymbolContext<T>
where
    Symbol<T>: SymbolAsKey,
{
    symbol_stack: Vec<Symbol<T>>,
    symbols: HashMap<SymbolKey<T>, Symbol<T>>,
}

impl<T> SymbolContext<T>
where
    T: Clone,
    Symbol<T>: SymbolAsKey,
{
    pub fn new() -> Self {
        Self {
            symbol_stack: Vec::new(),
            symbols: HashMap::new(),
        }
    }

    pub fn push(&mut self, symbol: &Symbol<T>) -> Option<Symbol<T>> {
        let key = symbol.as_key();

        self.symbol_stack.push(symbol.clone());
        self.symbols.insert(key.clone(), symbol.clone())
    }

    fn pop(&mut self) -> Result<Symbol<T>> {
        let popped_symbol = self
            .symbol_stack
            .pop()
            .ok_or(EmptyStackError {})
            .into_diagnostic()?;

        let map_symbol = self.symbols.remove(popped_symbol.as_key()).unwrap();

        Ok(map_symbol)
    }

    fn get(&self, key: &SymbolKey<T>) -> Option<&Symbol<T>> {
        self.symbols.get(key)
    }

    fn contains(&self, key: &SymbolKey<T>) -> bool {
        self.symbols.contains_key(key)
    }
}

// keeps track of symbols by keeping track of their scope as well
// allows reusing one context struct through all operations
pub struct SymbolTypeContext {
    label_context: SymbolContext<LabelSymbolData>,
    function_context: SymbolContext<FunctionSymbolData>,
}

impl SymbolTypeContext {
    pub fn new() -> Self {
        Self {
            label_context: SymbolContext::new(),
            function_context: SymbolContext::new(),
        }
    }

    pub fn push_label(&mut self, symbol: Symbol<LabelSymbolData>) -> Result<()> {
        let push_result = self.label_context.push(&symbol);

        if let Some(other) = push_result {
            DuplicateSymbolError::from_label_duplicates(symbol, other)
        } else {
            Ok(())
        }
    }

    fn pop_label(&mut self) -> Result<Symbol<LabelSymbolData>> {
        self.label_context.pop()
    }

    fn get_label(&self, name: &String) -> Option<&Symbol<LabelSymbolData>> {
        self.label_context.get(name)
    }

    fn contains_label(&self, name: &String) -> bool {
        self.label_context.contains(name)
    }

    pub fn push_function(&mut self, symbol: Symbol<FunctionSymbolData>) -> Result<()> {
        let push_result = self.function_context.push(&symbol);

        if let Some(other) = push_result {
            DuplicateSymbolError::from_function_duplicates(symbol, other)
        } else {
            Ok(())
        }
    }

    fn pop_function(&mut self) -> Result<Symbol<FunctionSymbolData>> {
        self.function_context.pop()
    }

    fn get_function(&self, name: &FunctionSignature) -> Option<&Symbol<FunctionSymbolData>> {
        self.function_context.get(name)
    }

    fn contains_function(&self, name: &FunctionSignature) -> bool {
        self.function_context.contains(name)
    }
}

pub fn typecheck(statements: &mut [StatementNode], symbols: &mut SymbolTypeContext) -> Result<()> {
    let mut labels = Vec::new();

    // must first typecheck functions to get their return type if not specified
    // only then can its symbol be constructed
    for statement in statements.iter_mut() {
        if let StatementKind::Function(function) = statement.inner_mut().inner_mut() {
            // first typecheck body
            // TODO: the typecheck context should exclude labels
            // the body context also includes the parameters

            for param in &function.params {
                // push into local scope
                let inner = param.inner();
                let curr_symbol = LabelSymbol {
                    name: inner.name.clone(),
                    symbol_type: inner.ty,
                    span: Some(param.span().clone()),
                };

                symbols
                    .push_label(curr_symbol.clone())
                    .wrap_err("Pushing local label symbol failed.")?;
            }

            typecheck(&mut function.body, symbols)?;

            // now pop all the symbols that were just added
            for param in function.params.iter().rev() {
                // push into local scope
                let inner = param.inner();
                let curr_symbol = LabelSymbol {
                    name: inner.name.clone(),
                    symbol_type: inner.ty,
                    span: Some(param.span().clone()),
                };

                let poppped = symbols.pop_label()?;
                if poppped != curr_symbol {
                    // TODO: make specific type for error with more details
                    return Err(miette!(
                        "Popped symbol does not match - original: {:?}, got: {:?}",
                        curr_symbol,
                        poppped,
                    ));
                }
            }

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
                    span: Some(statement.span().clone()),
                    data: LabelSymbolData { ty: Type::Label },
                };

                symbols
                    .push_label(curr_symbol.clone())
                    .wrap_err("Pushing local label symbol failed.")?;

                labels.push(curr_symbol);
            }
            StatementKind::Function(function) => {
                // push into local scope
                let curr_symbol = Symbol {
                    name: function.name.clone(),
                    span: Some(statement.span().clone()),
                    data: FunctionSymbolData {
                        signature: function.params
                    },
                };

                symbols
                    .push_label(curr_symbol.clone())
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
        let curr = symbols.pop_label()?;
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
            if symbols.contains_label(name) {
                if !matches!(inner.ty, Type::Unknown) {
                    Err(TypecheckExprError::new(
                        TypecheckExprErrorKind::IdentityAlreadyTyped((inner_span, inner.ty)),
                    ))?;
                }

                inner.ty = symbols.get_label(name).unwrap().symbol_type;
            } else {
                Err(TypecheckExprError::new(
                    TypecheckExprErrorKind::SymbolNotFound(LabelSymbol {
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

impl DuplicateSymbolError {
    fn from_function_duplicates(
        symbol: Symbol<FunctionSymbolData>,
        other: Symbol<FunctionSymbolData>,
    ) -> Result<()> {
        let mut spans = Vec::new();

        let source = symbol.span.as_ref().map(|e| e.to_miette_source_code());

        if let Some(ast_span) = symbol.span.as_ref() {
            spans.push(LabeledSpan::new_with_span(
                Some(format!(
                    "Symbol of signature \"{:?}\" defined here",
                    symbol.data.signature
                )),
                ast_span.to_miette_span(),
            ));
        };

        if let Some(ast_span) = other.span.as_ref() {
            spans.push(LabeledSpan::new_with_span(
                Some(format!(
                    "Symbol of signature \"{:?}\" defined here",
                    other.data.signature
                )),
                ast_span.to_miette_span(),
            ));
        };

        Err(DuplicateSymbolError {
            name: symbol.name,
            source,
            spans,
        })?
    }

    fn from_label_duplicates(
        symbol: Symbol<LabelSymbolData>,
        other: Symbol<LabelSymbolData>,
    ) -> Result<()> {
        let mut spans = Vec::new();

        let source = symbol.span.as_ref().map(|e| e.to_miette_source_code());

        if let Some(ast_span) = symbol.span.as_ref() {
            spans.push(LabeledSpan::new_with_span(
                Some(format!(
                    "Symbol of type \"{:?}\" defined here",
                    symbol.data.ty
                )),
                ast_span.to_miette_span(),
            ));
        };

        if let Some(ast_span) = other.span.as_ref() {
            spans.push(LabeledSpan::new_with_span(
                Some(format!(
                    "Symbol of type \"{:?}\" defined here",
                    other.data.ty
                )),
                ast_span.to_miette_span(),
            ));
        };

        Err(DuplicateSymbolError {
            name: symbol.name,
            source,
            spans,
        })?
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
    SymbolNotFound(LabelSymbol),
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
