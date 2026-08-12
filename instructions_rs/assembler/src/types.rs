use std::{collections::HashMap, error::Error, fmt::Display, sync::Arc};

use crate::ast::{AstNode, AstSpan, BinaryOp, Expr, Statement, TypedExpr, UnaryOp};
use miette::{Context, Diagnostic, IntoDiagnostic, NamedSource, Result, SourceSpan, miette};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Type {
    /// Integer type (64-bit signed)
    Int,
    /// Boolean type
    Bool,
    String,
    Character,

    Register,
    AddressRegister,

    Addr,
    Byte,

    Label,

    /// Unknown type
    Unknown,
}

impl Type {
    pub fn unify(&self, other: &Type) -> Result<Type> {
        match (self, other) {
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Bool, Type::Bool) => Ok(Type::Bool),

            // math on labels will mean an address
            (Type::Label, Type::Int) | (Type::Int, Type::Label) => Ok(Type::Addr),

            // allow coercing characters into int
            (Type::Character, Type::Int) | (Type::Int, Type::Character) => Ok(Type::Int),

            // Unknown can unify with anything
            (Type::Unknown, t) | (t, Type::Unknown) => Ok(*t),

            // Type mismatch
            _ => Err(miette!(
                "Type mismatch: expected {:?}, got {:?}",
                self,
                other
            )),
        }
    }

    // returns true if types are comparable to each other
    pub fn comparable(lhs: &Type, rhs: &Type) -> Result<bool> {
        match (lhs, rhs) {
            (Type::Label, Type::Int) | (Type::Int, Type::Label) => Ok(true),
            (Type::Int, Type::Int) => Ok(true),
            (Type::Bool, Type::Bool) => Ok(true),

            // Type mismatch
            _ => Err(miette!(
                "Types not comparable - lhs: {:?}, rhs {:?}",
                lhs,
                rhs
            )),
        }
    }

    // returns true if both types are operable with arithmetic operations
    pub fn operable(lhs: &Type, rhs: &Type) -> Result<bool> {
        match (lhs, rhs) {
            (Type::Int, Type::Int) => Ok(true),

            // allow math between labels and int
            (Type::Label, Type::Int) | (Type::Int, Type::Label) => Ok(true),

            // allow math between characters and int
            (Type::Character, Type::Int) | (Type::Int, Type::Character) => Ok(true),

            // Type mismatch
            _ => Err(miette!(
                "Can't operate arithmetic on types - lhs: {:?}, rhs: {:?}",
                lhs,
                rhs
            )),
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
            let source = symbol.span.as_ref().map(|e| e.to_miette_source_code());
            let current_symbol_span = symbol.span.as_ref().map(AstSpan::to_miette_span);
            let other_symbol_span = other.span.as_ref().map(AstSpan::to_miette_span);

            Err(DuplicateSymbolError {
                name: symbol.name,
                type1: other.symbol_type,
                type2: symbol.symbol_type,
                source,
                current_symbol_span,
                other_symbol_span,
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

pub fn typecheck(
    statements: &mut [AstNode<Statement>],
    symbols: &mut SymbolTypeContext,
) -> Result<()> {
    // TODO: can turn into label vec to verify poppped values are accurate (if pop returns val)
    let mut labels = Vec::new();

    // first find all labels in the current scope (accessible from anywhere in scope)
    for statement in statements.iter() {
        if let Statement::Label { name } | Statement::BlockLabel { name, .. } = statement.inner() {
            // TODO: ensure not duplicate
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
    }

    for statement in statements.iter_mut() {
        match statement.inner_mut() {
            Statement::BlockLabel { body, .. } => {
                typecheck(body, symbols)?;
            }
            Statement::Instruction(instruction) => {
                for param in instruction.params.iter_mut() {
                    typecheck_expr(param, symbols)?;
                }
            }
            _ => (),
        };
    }

    // checks that all returned symbols match what was pushed in
    // goes in reverse since pop starts from the last added element
    for label in labels.into_iter().rev() {
        let curr = symbols.pop()?;
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

fn typecheck_expr(typed_expr: &mut AstNode<TypedExpr>, symbols: &SymbolTypeContext) -> Result<()> {
    let inner = typed_expr.inner_mut();

    match &mut inner.expr {
        Expr::Int(_) => inner.ty = Type::Int,
        Expr::Bool(_) => inner.ty = Type::Bool,
        Expr::String(_) => inner.ty = Type::String,
        Expr::Char(_) => inner.ty = Type::Character,
        Expr::Identity(name) => {
            // try and find identity in symbols
            // println!("querying for symbol: {name}");
            if symbols.contains(name) {
                if !matches!(inner.ty, Type::Unknown) {
                    return Err(miette!("Identity already has type: {:#?}", inner.ty));
                }
                // println!("found symbol: {name}");
                inner.ty = symbols.get(name).unwrap().symbol_type;
            } else {
                // return Err(miette!("Symbol not found: {}", name));
                Err(TypecheckExprError::new(
                    TypecheckExprErrorKind::SymbolNotFound(Symbol {
                        name: name.to_string(),
                        symbol_type: inner.ty,
                        span: Some(typed_expr.span.clone()),
                    }),
                ))?;
            }
        }
        Expr::Unary { op, expr } => {
            typecheck_expr(expr, symbols)?;
            let expr = expr.inner_mut();

            // TODO: change to consider arithmetically operable and boolean operable
            match op {
                UnaryOp::Neg => {
                    if expr.ty != Type::Int {
                        return Err(miette!("Cannot negate non-integer type: {:#?}", expr.ty));
                    }
                    expr.ty = Type::Int;
                }
                UnaryOp::Not => {
                    if expr.ty != Type::Bool {
                        return Err(miette!("Cannot negate non-boolean type: {:#?}", expr.ty));
                    }
                    expr.ty = Type::Bool;
                }
                UnaryOp::BitNegation => {
                    if expr.ty != Type::Int {
                        return Err(miette!("Cannot bit negate non-int type: {:#?}", expr.ty));
                    }
                    expr.ty = Type::Bool;
                }
            }
        }
        Expr::Binary { op, left, right } => {
            typecheck_expr(left, symbols)?;
            typecheck_expr(right, symbols)?;
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
                | BinaryOp::BitOr
                | BinaryOp::And
                | BinaryOp::Or => {
                    if !Type::operable(&left.ty, &right.ty)? {
                        return Err(miette!(
                            "Arithmetic operation requires int operands, got {:#?} and {:#?}",
                            left.ty,
                            right.ty
                        ));
                    }
                    inner.ty = left.ty.unify(&right.ty)?;
                }
                BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                    if !Type::comparable(&left.ty, &right.ty)? {
                        return Err(miette!(
                            "Comparison requires comparable operands, got {:#?} and {:#?}",
                            left.ty,
                            right.ty
                        ));
                    }
                    inner.ty = Type::Bool;
                }
                BinaryOp::Eq | BinaryOp::Ne => {
                    let _ = left.ty.unify(&right.ty)?;
                    inner.ty = Type::Bool;
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
    SymbolNotFound(Symbol),
    InvalidArithmeticTypes((AstSpan, Type), (AstSpan, Type)),
    InvalidComparisonTypes((AstSpan, Type), (AstSpan, Type)),
    InvalidEqualityTypes((AstSpan, Type), (AstSpan, Type)),
    InvalidArithmeticType((AstSpan, Type)),
    InvalidComparisonType((AstSpan, Type)),
}

impl TypecheckExprErrorKind {}

impl Display for TypecheckExprErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypecheckExprErrorKind::IdentityAlreadyTyped((span, ty)) => write!(f, "Identity "),
            TypecheckExprErrorKind::SymbolNotFound(symbol) => {
                write!(f, "Symbol \"{}\" not found", symbol.name)
            }
            TypecheckExprErrorKind::InvalidArithmeticTypes(_, _) => write!(f, "what"),
            TypecheckExprErrorKind::InvalidComparisonTypes(_, _) => write!(f, "what"),
            TypecheckExprErrorKind::InvalidEqualityTypes(_, _) => write!(f, "what"),
            TypecheckExprErrorKind::InvalidArithmeticType(_) => write!(f, "what"),
            TypecheckExprErrorKind::InvalidComparisonType(_) => write!(f, "what"),
        }
    }
}

#[derive(Diagnostic, Debug)]
pub struct TypecheckExprError {
    #[source_code]
    source: Option<NamedSource<Arc<str>>>,
    kind: TypecheckExprErrorKind,

    #[label(primary,"Defined here")]
    symbol_span_1: Option<SourceSpan>,

    #[label("Second defined here")]
    symbol_span_2: Option<SourceSpan>,
}

impl Error for TypecheckExprError {}

impl Display for TypecheckExprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl TypecheckExprError {
    fn new(kind: TypecheckExprErrorKind) -> Self {
        let (source, span1) = match &kind {
            TypecheckExprErrorKind::InvalidArithmeticTypes((ast_span, _), _)
            | TypecheckExprErrorKind::InvalidComparisonTypes((ast_span, _), _)
            | TypecheckExprErrorKind::InvalidEqualityTypes((ast_span, _), _)
            | TypecheckExprErrorKind::InvalidArithmeticType((ast_span, _))
            | TypecheckExprErrorKind::InvalidComparisonType((ast_span, _))
            | TypecheckExprErrorKind::IdentityAlreadyTyped((ast_span, _)) => (
                Some(ast_span.to_miette_source_code()),
                Some(ast_span.into()),
            ),
            TypecheckExprErrorKind::SymbolNotFound(symbol) => (
                symbol.span.as_ref().map(AstSpan::to_miette_source_code),
                symbol.span.as_ref().map(AstSpan::to_miette_span),
            ),
        };

        let span2: Option<SourceSpan> = match &kind {
            TypecheckExprErrorKind::InvalidArithmeticTypes(_, (ast_span, _))
            | TypecheckExprErrorKind::InvalidComparisonTypes(_, (ast_span, _))
            | TypecheckExprErrorKind::InvalidEqualityTypes(_, (ast_span, _)) => {
                Some(ast_span.into())
            }
            TypecheckExprErrorKind::SymbolNotFound(_)
            | TypecheckExprErrorKind::InvalidArithmeticType(_)
            | TypecheckExprErrorKind::InvalidComparisonType(_)
            | TypecheckExprErrorKind::IdentityAlreadyTyped(_) => None,
        };

        TypecheckExprError {
            symbol_span_1: span1,
            symbol_span_2: span2,
            source,
            kind,
        }
    }
}
