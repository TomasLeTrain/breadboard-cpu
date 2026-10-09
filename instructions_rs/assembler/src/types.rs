use std::{collections::HashMap, error::Error, fmt::Display, hash::Hash, sync::Arc};

use crate::ast::{
    AstNode, AstSpan, BinaryOp, Expr, ExprKind, FunctionCall, ReturnKind, StatementKind,
    StatementNode, UnaryOp,
};
use miette::{Context, Diagnostic, IntoDiagnostic, LabeledSpan, NamedSource, Result, miette};

pub type Address = u16;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Int,
    Addr,
    Byte,

    Bool,
    String,

    Register,
    AddressRegister,

    Label,
    Function {
        name: String,
        params: Vec<Type>,
        return_ty: Box<Type>,
    },

    Block,

    Unknown,
}

impl Type {
    // collapses functions to their return type to allow operating on their output
    // pub fn to_simple(&self) -> Type {
    //     if let Type::Function { return_ty, .. } = self {
    //         return_ty.as_ref().clone()
    //     } else {
    //         self.clone()
    //     }
    // }

    pub fn as_simple(&self) -> &Type {
        if let Type::Function { return_ty, .. } = self {
            return_ty.as_ref()
        } else {
            self
        }
    }

    pub fn int_operable(&self) -> bool {
        matches!(
            self.as_simple(),
            Type::Int | Type::Label | Type::Byte | Type::Addr
        )
    }

    pub fn bool_operable(&self) -> bool {
        matches!(self.as_simple(), Type::Bool)
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
        let self_simple = self.as_simple();
        let other_simple = other.as_simple();

        if Self::int_binary_operable(self_simple, other_simple) {
            Some(Type::Int)
        } else if Self::bool_binary_operable(self_simple, other_simple) {
            Some(Type::Bool)
        } else if let (Type::Unknown, t) | (t, Type::Unknown) = (self_simple, other_simple) {
            Some(t.clone())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub name: String,
    pub span: Option<AstSpan>,
    pub ty: Type,
}

#[derive(Clone)]
pub struct SymbolContext {
    symbol_stack: Vec<Symbol>,
    symbols: HashMap<String, Symbol>,
    kind: ContextKind,
}

impl SymbolContext {
    pub fn new(kind: ContextKind) -> Self {
        Self {
            symbol_stack: Vec::new(),
            symbols: HashMap::new(),
            kind,
        }
    }

    pub fn push(&mut self, symbol: &Symbol) -> Option<Symbol> {
        self.symbol_stack.push(symbol.clone());
        self.symbols.insert(symbol.name.clone(), symbol.clone())
    }

    fn pop(&mut self) -> Result<Symbol> {
        let popped_symbol = self
            .symbol_stack
            .pop()
            .ok_or(EmptyStackError { kind: self.kind })
            .into_diagnostic()?;

        let map_symbol = self.symbols.remove(&popped_symbol.name).unwrap();

        Ok(map_symbol)
    }

    fn get(&self, key: &String) -> Option<&Symbol> {
        self.symbols.get(key)
    }

    fn contains(&self, key: &String) -> bool {
        self.symbols.contains_key(key)
    }
}

// keeps track of symbols by keeping track of their scope as well
// allows reusing one context struct through all operations

// local and global separated for constructing new contexts with only global symbols
// in this context local means local to only the current scope, global means accessible to all
// scopes below its definition
pub struct TypecheckContext {
    local_context: SymbolContext,
    global_context: SymbolContext,
}

#[derive(Clone, Copy, Debug)]
pub enum ContextKind {
    Local,
    Global,
}

impl Display for ContextKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ContextKind::Local => "local",
            ContextKind::Global => "global",
        };
        write!(f, "{name}")
    }
}

impl TypecheckContext {
    pub fn new() -> Self {
        Self {
            local_context: SymbolContext::new(ContextKind::Local),
            global_context: SymbolContext::new(ContextKind::Global),
        }
    }

    pub fn clone_global_ctx_only(&self) -> Self {
        Self {
            local_context: SymbolContext::new(ContextKind::Local),
            global_context: self.global_context.clone(),
        }
    }

    fn kind_to_ctx(&self, kind: ContextKind) -> &SymbolContext {
        match kind {
            ContextKind::Local => &self.local_context,
            ContextKind::Global => &self.global_context,
        }
    }

    fn kind_to_ctx_mut(&mut self, kind: ContextKind) -> &mut SymbolContext {
        match kind {
            ContextKind::Local => &mut self.local_context,
            ContextKind::Global => &mut self.global_context,
        }
    }

    fn ctx_push(&mut self, symbol: Symbol, kind: ContextKind) -> Result<()> {
        // must check in all context to ensure no conflicts across different context
        if self.contains(&symbol.name) {
            let other = self.get(&symbol.name).unwrap().clone();
            return DuplicateSymbolError::from_symbols(symbol, other);
        }

        self.kind_to_ctx_mut(kind)
            .push(&symbol)
            .map(|other| DuplicateSymbolError::from_symbols(symbol, other))
            .unwrap_or(Ok(()))
    }

    fn ctx_pop(&mut self, kind: ContextKind) -> Result<Symbol> {
        self.kind_to_ctx_mut(kind).pop()
    }

    fn ctx_get(&self, name: &String, kind: ContextKind) -> Option<&Symbol> {
        self.kind_to_ctx(kind).get(name)
    }

    pub fn push_local(&mut self, symbol: Symbol) -> Result<()> {
        self.ctx_push(symbol, ContextKind::Local)
    }

    pub fn push_global(&mut self, symbol: Symbol) -> Result<()> {
        self.ctx_push(symbol, ContextKind::Global)
    }

    fn pop_local(&mut self) -> Result<Symbol> {
        self.ctx_pop(ContextKind::Local)
    }

    fn pop_global(&mut self) -> Result<Symbol> {
        self.ctx_pop(ContextKind::Global)
    }

    fn get_local(&self, name: &String) -> Option<&Symbol> {
        self.ctx_get(name, ContextKind::Local)
    }

    fn get_global(&self, name: &String) -> Option<&Symbol> {
        self.ctx_get(name, ContextKind::Global)
    }

    fn get(&self, name: &String) -> Option<&Symbol> {
        self.get_local(name).or(self.get_global(name))
    }

    //
    // fn contains_local(&self, name: &String) -> bool {
    //     self.local_context.contains(name)
    // }
    //
    // fn contains_global(&self, name: &String) -> bool {
    //     self.global_context.contains(name)
    // }

    fn contains(&self, name: &String) -> bool {
        self.local_context.contains(name) || self.global_context.contains(name)
    }
}

pub fn typecheck(statements: &mut [StatementNode], ctx: &mut TypecheckContext) -> Result<()> {
    let mut pushed_locals = Vec::new();
    let mut pushed_globals = Vec::new();

    // must first typecheck functions to get their return type if not specified
    // only then can its symbol be constructed
    for statement in statements.iter_mut() {
        if let StatementKind::Function(function) = statement.inner_mut().inner_mut() {
            // only copies function context - no access to outer labels
            let mut function_ctx = ctx.clone_global_ctx_only();

            for param in &function.params {
                // push into local scope
                let inner = param.inner();

                let curr_symbol = Symbol {
                    name: inner.name.clone(),
                    ty: inner.ty.clone(),
                    span: Some(param.span().clone()),
                };

                function_ctx
                    .push_local(curr_symbol.clone())
                    .wrap_err("Pushing function symbol failed.")?;
            }

            typecheck(&mut function.body, &mut function_ctx).context(format!(
                "Parsing function body of \"{}\" failed.",
                function.name
            ))?;

            // now pop all the symbols that were just added
            for param in function.params.iter().rev() {
                let inner = param.inner();

                let curr_symbol = Symbol {
                    name: inner.name.clone(),
                    ty: inner.ty.clone(),
                    span: Some(param.span().clone()),
                };

                let pop_val = function_ctx.pop_local()?;

                if pop_val != curr_symbol {
                    // TODO: make specific type for error with more details
                    return Err(miette!(
                        "Popped parameter symbol does not match - original: {:?}, got: {:?}",
                        curr_symbol,
                        pop_val,
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
                        ReturnKind::Expr(expr) => expr.inner().ty.clone(),
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
                    ty: Type::Label,
                };

                ctx.push_local(curr_symbol.clone())
                    .wrap_err("Pushing local label symbol failed.")?;

                pushed_locals.push(curr_symbol);
            }
            StatementKind::Function(function) => {
                // push into local scope
                let curr_symbol = Symbol {
                    name: function.name.clone(),
                    span: Some(statement.span().clone()),
                    ty: Type::Function {
                        name: function.name.clone(),
                        params: function.as_signature().params,
                        return_ty: Box::new(function.return_type.clone()),
                    },
                };

                ctx.push_global(curr_symbol.clone())
                    .wrap_err("Pushing function symbol failed.")?;

                pushed_globals.push(curr_symbol);
            }
            _ => (),
        }
    }

    for statement in statements.iter_mut() {
        // NOTE: functions were already typechecked
        match statement.inner_mut().inner_mut() {
            StatementKind::BlockLabel { body, .. } | StatementKind::Block { body } => {
                typecheck(body, ctx)?;
            }
            StatementKind::Return(kind) => match kind {
                ReturnKind::Expr(expr) => typecheck_expr(expr, ctx)?,
                ReturnKind::Block(block) => typecheck(block, ctx)?,
            },
            StatementKind::Instruction(instruction) => {
                for param in instruction.params.iter_mut() {
                    typecheck_expr(param, ctx)?;
                }
            }
            StatementKind::FunctionCall(FunctionCall { name, params }) => {
                println!("whar {name}");

                // typecheck all params first
                for param in params.iter_mut() {
                    typecheck_expr(param, ctx)?;
                }

                // try and find identity in symbols
                if ctx.contains(name) {
                    let found_symbol = ctx.get(name).unwrap();

                    // make sure function signature of call and found symbol match
                    if let Type::Function {
                        params: found_params,
                        return_ty,
                        ..
                    } = &found_symbol.ty
                    {
                        let matching_elements = params
                            .iter()
                            .map(|e| e.inner.ty.clone())
                            .zip(found_params.iter())
                            .filter(|(a, b)| a.as_simple() == b.as_simple())
                            .count();

                        if matching_elements != params.len()
                            || matching_elements != found_params.len()
                        {
                            // parameters dont match
                            // TODO: make detailed error
                            Err(miette!(
                                "expected function of signature, found function with different signature"
                            ))?;
                        }

                        if !matches!(return_ty.as_ref(), Type::Block) {
                            // TODO: make detailed error
                            Err(miette!("expected function to return block, instead ..."))?;
                        }
                    } else {
                        // TODO: make detailed error
                        Err(miette!("expected function, got different type:"))?;
                    }
                } else {
                    // TODO: make detailed error
                    Err(miette!("function of name \"{}\" not found", name))?;
                }
            }
            _ => (),
        };
    }

    // Checks that all returned symbols match what was pushed in.
    // Goes in reverse since pop starts from the last added element
    for symbol in pushed_locals.into_iter().rev() {
        let curr = ctx.pop_local()?;
        if symbol != curr {
            // TODO: make specific type for error with more details
            return Err(miette!(
                "Popped symbol does not match - original: {:?}, got: {:?}",
                symbol,
                curr,
            ));
        }
    }

    // Checks that all returned symbols match what was pushed in.
    // Goes in reverse since pop starts from the last added element
    for symbol in pushed_globals.into_iter().rev() {
        let curr = ctx.pop_global()?;
        if symbol != curr {
            // TODO: make specific type for error with more details
            return Err(miette!(
                "Popped symbol does not match - original: {:?}, got: {:?}",
                symbol,
                curr,
            ));
        }
    }

    Ok(())
}

fn typecheck_expr(typed_expr: &mut AstNode<Expr>, symbols: &TypecheckContext) -> Result<()> {
    let inner_span = typed_expr.span().clone();
    let inner = typed_expr.inner_mut();

    match &mut inner.kind {
        // literals already have their typed filled in
        ExprKind::Literal => (),
        ExprKind::FunctionCall(FunctionCall { name, params }) => {
            println!("whar {name}");

            // typecheck all params first
            for param in params.iter_mut() {
                typecheck_expr(param, symbols)?;
            }

            // try and find identity in symbols
            if symbols.contains(name) {
                if !matches!(inner.ty, Type::Unknown) {
                    Err(TypecheckExprError::new(
                        TypecheckExprErrorKind::IdentityAlreadyTyped((
                            inner_span,
                            inner.ty.clone(),
                        )),
                    ))?;
                }

                let found_symbol = symbols.get(name).unwrap();

                // make sure function signature of call and found symbol match
                if let Type::Function {
                    params: found_params,
                    ..
                } = &found_symbol.ty
                {
                    let matching_elements = params
                        .iter()
                        .map(|e| e.inner.ty.clone())
                        .zip(found_params.iter())
                        .filter(|(a, b)| a.as_simple() == b.as_simple())
                        .count();

                    if matching_elements == params.len() && matching_elements == found_params.len()
                    {
                        // signature matches, function is valid
                        inner.ty = found_symbol.ty.clone();
                    } else {
                        // TODO: make detailed error
                        Err(miette!(
                            "expected function of signature, found function with different signature"
                        ))?;
                    }
                } else {
                    // TODO: make detailed error
                    Err(miette!("expected function, got different type:"))?;
                }
            } else {
                Err(TypecheckExprError::new(
                    TypecheckExprErrorKind::SymbolNotFound(Symbol {
                        name: name.to_string(),
                        ty: inner.ty.clone(),
                        span: Some(typed_expr.span.clone()),
                    }),
                ))?;
            }
        }
        ExprKind::Identity(name) => {
            // try and find identity in symbols
            if symbols.contains(name) {
                if !matches!(inner.ty, Type::Unknown) {
                    Err(TypecheckExprError::new(
                        TypecheckExprErrorKind::IdentityAlreadyTyped((
                            inner_span,
                            inner.ty.clone(),
                        )),
                    ))?;
                }

                // TODO: ensure type is not invalid (example function)
                inner.ty = symbols.get(name).unwrap().ty.clone();
            } else {
                Err(TypecheckExprError::new(
                    TypecheckExprErrorKind::SymbolNotFound(Symbol {
                        name: name.to_string(),
                        ty: inner.ty.clone(),
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
                            TypecheckExprErrorKind::InvalidUnaryOpType(
                                (span, unary_expr.ty.clone()),
                                *op,
                            ),
                        ))?;
                    }
                    inner.ty = Type::Int;
                }
                UnaryOp::Not => {
                    if !unary_expr.ty.bool_operable() {
                        Err(TypecheckExprError::new(
                            TypecheckExprErrorKind::InvalidUnaryOpType(
                                (span, unary_expr.ty.clone()),
                                *op,
                            ),
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
                                (left_span, left.ty.clone()),
                                (right_span, right.ty.clone()),
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
                                (left_span, left.ty.clone()),
                                (right_span, right.ty.clone()),
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
                                (left_span, left.ty.clone()),
                                (right_span, right.ty.clone()),
                            ),
                        ))?;
                    }
                    inner.ty = Type::Bool;
                }
                BinaryOp::Eq | BinaryOp::Ne => {
                    if left.ty.unify(&right.ty).is_none() {
                        Err(TypecheckExprError::new(
                            TypecheckExprErrorKind::InvalidEqualityTypes(
                                (left_span, left.ty.clone()),
                                (right_span, right.ty.clone()),
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
    fn from_symbols(symbol: Symbol, other: Symbol) -> Result<()> {
        let mut spans = Vec::new();

        let source = symbol.span.as_ref().map(|e| e.to_miette_source_code());

        if let Some(ast_span) = symbol.span.as_ref() {
            spans.push(LabeledSpan::new_with_span(
                Some(format!("Symbol of type \"{:?}\" defined here", symbol.ty)),
                ast_span.to_miette_span(),
            ));
        };

        if let Some(ast_span) = other.span.as_ref() {
            spans.push(LabeledSpan::new_with_span(
                Some(format!("Symbol of type \"{:?}\" defined here", symbol.ty)),
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
pub struct EmptyStackError {
    kind: ContextKind,
}

impl Error for EmptyStackError {}

impl Display for EmptyStackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No symbols in {} stack.", self.kind)
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
