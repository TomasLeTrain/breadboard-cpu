use std::{collections::HashMap, error::Error, fmt::Display};

use crate::{
    ast::{AstNode, BinaryOp, Expr, Statement, TypedExpr, UnaryOp},
    parser::ParseError,
};
use miette::{IntoDiagnostic, Result, miette};
use pest::Span;

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
    pub fn unify(&self, other: &Type) -> Result<Type, String> {
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
            _ => Err(format!(
                "Type mismatch: expected {:?}, got {:?}",
                self, other
            )),
        }
    }

    // returns true if types are comparable to each other
    pub fn comparable(lhs: &Type, rhs: &Type) -> Result<bool, String> {
        match (lhs, rhs) {
            (Type::Label, Type::Int) | (Type::Int, Type::Label) => Ok(true),
            (Type::Int, Type::Int) => Ok(true),
            (Type::Bool, Type::Bool) => Ok(true),

            // Type mismatch
            _ => Err(format!(
                "Types not comparable - lhs: {:?}, rhs {:?}",
                lhs, rhs
            )),
        }
    }

    // returns true if both types are operable with arithmetic operations
    pub fn operable(lhs: &Type, rhs: &Type) -> Result<bool, String> {
        match (lhs, rhs) {
            (Type::Int, Type::Int) => Ok(true),

            // allow math between labels and int
            (Type::Label, Type::Int) | (Type::Int, Type::Label) => Ok(true),

            // allow math between characters and int
            (Type::Character, Type::Int) | (Type::Int, Type::Character) => Ok(true),

            // Type mismatch
            _ => Err(format!(
                "Can't operate arithmetic on types - lhs: {:?}, rhs: {:?}",
                lhs, rhs
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol<'a> {
    pub name: String,
    pub symbol_type: Type,
    pub definition: Option<Span<'a>>,
}

// keeps track of symbols by keeping track of their scope as well
// allows reusing one context struct through all operations
pub struct SymbolTypeContext<'a> {
    symbol_stack: Vec<Symbol<'a>>,
    symbols: HashMap<String, Type>,
}

#[derive(Debug)]
pub struct DuplicateSymbolError {
    name: String,
    type1: Type,
    type2: Type,
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

impl<'a> SymbolTypeContext<'a> {
    pub fn new() -> Self {
        SymbolTypeContext {
            symbol_stack: Vec::new(),
            symbols: HashMap::new(),
        }
    }

    pub fn push(&mut self, symbol: Symbol<'a>) -> Result<()> {
        self.symbol_stack.push(symbol.clone());
        let pushed_symbol = symbol.clone();

        let push_result = self
            .symbols
            .insert(pushed_symbol.name, pushed_symbol.symbol_type);

        if let Some(other) = push_result {
            Err(DuplicateSymbolError {
                name: symbol.name,
                type1: other,
                type2: symbol.symbol_type,
            })
            .into_diagnostic()
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

        let returned_type = self.symbols.remove(&popped_symbol.name).unwrap();

        Ok(Symbol {
            name: popped_symbol.name,
            symbol_type: returned_type,
            definition: None,
        })
    }

    fn get(&self, name: &String) -> Option<&Type> {
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
        if let Statement::Label { name } = statement.inner() {
            // TODO: ensure not duplicate
            // push into local scope
            let curr_symbol = Symbol {
                name: name.clone(),
                symbol_type: Type::Label,
                definition: None
            };
            symbols.push(curr_symbol.clone())?;
            labels.push(curr_symbol);
        }
    }

    for statement in statements.iter_mut() {
        // must match whole statement to get span without an additional mutable borrow
        match statement {
            AstNode {
                inner: Statement::BlockLabel { name, body },
                span,
            } => {
                // push into local scope
                if let Err(_err) = symbols.push(Symbol {
                    name: name.clone(),
                    symbol_type: Type::Label,
                    definition: Some(*span)
                }) {
                    Err(ParseError::from_span("Duplicate symbol in scope", *span))?
                }

                // fill labels for block
                typecheck(body, symbols)?;

                // remove from scope
                symbols.pop()?;
            }
            AstNode {
                inner: Statement::Instruction(instruction),
                span: _span,
            } => {
                for param in &mut instruction.params.iter_mut() {
                    typecheck_expr(param, symbols).unwrap();
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

fn typecheck_expr(
    typed_expr: &mut AstNode<TypedExpr>,
    symbols: &SymbolTypeContext,
) -> Result<(), String> {
    let inner = typed_expr.inner_mut();

    match &mut inner.expr {
        Expr::Int(_) => inner.ty = Type::Int,
        Expr::Bool(_) => inner.ty = Type::Bool,
        Expr::String(_) => inner.ty = Type::String,
        Expr::Char(_) => inner.ty = Type::Character,
        Expr::Identity(name) => {
            // try and find identity in symbols
            println!("querying for symbol: {name}");
            if symbols.contains(name) {
                if !matches!(inner.ty, Type::Unknown) {
                    return Err(format!("Identity already has type: {:#?}", inner.ty));
                }
                println!("found symbol: {name}");
                inner.ty = *symbols.get(name).unwrap();
            } else {
                return Err(format!("Symbol not found: {}", name));
            }
        }
        Expr::Unary { op, expr } => {
            typecheck_expr(expr, symbols)?;
            let expr = expr.inner_mut();

            // TODO: change to consider arithmetically operable and boolean operable
            match op {
                UnaryOp::Neg => {
                    if expr.ty != Type::Int {
                        return Err(format!("Cannot negate non-integer type: {:#?}", expr.ty));
                    }
                    expr.ty = Type::Int;
                }
                UnaryOp::Not => {
                    if expr.ty != Type::Bool {
                        return Err(format!("Cannot negate non-boolean type: {:#?}", expr.ty));
                    }
                    expr.ty = Type::Bool;
                }
                UnaryOp::BitNegation => {
                    if expr.ty != Type::Int {
                        return Err(format!("Cannot bit negate non-int type: {:#?}", expr.ty));
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
                        return Err(format!(
                            "Arithmetic operation requires int operands, got {:#?} and {:#?}",
                            left.ty, right.ty
                        ));
                    }
                    inner.ty = left.ty.unify(&right.ty)?;
                }
                BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                    if !Type::comparable(&left.ty, &right.ty)? {
                        return Err(format!(
                            "Comparison requires comparable operands, got {:#?} and {:#?}",
                            left.ty, right.ty
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
