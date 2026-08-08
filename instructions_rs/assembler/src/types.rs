use std::collections::HashMap;

use crate::ast::{BinaryOp, Expr, AstInstruction, Statement, TypedExpr, UnaryOp};

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

    Label,

    /// Unknown type
    Unknown,
}

impl Type {
    pub fn unify(&self, other: &Type) -> Result<Type, String> {
        match (self, other) {
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Bool, Type::Bool) => Ok(Type::Bool),

            // allow coercing labels into int (address)
            (Type::Label, Type::Int) | (Type::Int, Type::Label) => Ok(Type::Int),

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
pub struct Symbol {
    pub name: String,
    pub symbol_type: Type,
}

// keeps track of symbols by keeping track of their scope as well
// allows reusing one context struct through all operations
pub struct SymbolTypeContext {
    symbol_stack: Vec<Symbol>,
    symbols: HashMap<String, Type>,
}

impl SymbolTypeContext {
    pub fn new() -> Self {
        SymbolTypeContext {
            symbol_stack: Vec::new(),
            symbols: HashMap::new(),
        }
    }

    pub fn push(&mut self, symbol: Symbol) {
        // TODO: add errors
        self.symbol_stack.push(symbol.clone());
        self.symbols.insert(symbol.name, symbol.symbol_type);
    }

    fn pop(&mut self) {
        // TODO: turn into errors
        let popped_symbol = self.symbol_stack.pop().unwrap();
        self.symbols.remove(&popped_symbol.name).unwrap();
    }

    fn get(&self, name: &String) -> Option<&Type> {
        self.symbols.get(name)
    }

    fn contains(&self, name: &String) -> bool {
        self.symbols.contains_key(name)
    }
}

pub fn typecheck(statements: &mut [Statement], symbols: &mut SymbolTypeContext) {
    // TODO: can turn into label vec to verify poppped values are accurate (if pop returns val)
    let mut num_labels = 0;

    // first find all labels in the current scope (accessible from anywhere in scope)
    for statement in statements.iter() {
        if let Statement::Label { name } = statement {
            // TODO: ensure not duplicate
            // push into local scope
            symbols.push(Symbol {
                name: name.clone(),
                symbol_type: Type::Label,
            });
            num_labels += 1;
        }
    }

    for statement in statements.iter_mut() {
        match statement {
            Statement::BlockLabel { name, body } => {
                // TODO: ensure not duplicate
                // push into local scope
                symbols.push(Symbol {
                    name: name.clone(),
                    symbol_type: Type::Label,
                });

                // fill labels for block
                typecheck(body, symbols);

                // TODO: turn into error
                // remove from scope
                symbols.pop();
            }
            Statement::Instruction(instruction) => {
                for param in &mut instruction.params.iter_mut() {
                    typecheck_expr(param, symbols).unwrap();
                }
            }
            _ => (),
        }
    }

    for _ in 0..num_labels {
        symbols.pop();
    }
}

fn typecheck_expr(typed_expr: &mut TypedExpr, symbols: &SymbolTypeContext) -> Result<(), String> {
    match &mut typed_expr.expr {
        Expr::Int(_) => typed_expr.ty = Type::Int,
        Expr::Bool(_) => typed_expr.ty = Type::Bool,
        Expr::String(_) => typed_expr.ty = Type::String,
        Expr::Char(_) => typed_expr.ty = Type::Character,
        Expr::Identity(name) => {
            // try and find identity in symbols
            println!("querying for symbol: {name}");
            if symbols.contains(name) {
                if !matches!(typed_expr.ty, Type::Unknown) {
                    return Err(format!("Identity already has type: {:#?}", typed_expr.ty));
                }
                println!("found symbol: {name}");
                typed_expr.ty = *symbols.get(name).unwrap();
            } else {
                return Err(format!("Symbol not found: {}", name));
            }
        }
        Expr::Unary { op, expr } => {
            typecheck_expr(expr, symbols)?;

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
                    typed_expr.ty = left.ty.unify(&right.ty)?;
                }
                BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
                    if !Type::comparable(&left.ty, &right.ty)? {
                        return Err(format!(
                            "Comparison requires comparable operands, got {:#?} and {:#?}",
                            left.ty, right.ty
                        ));
                    }
                    typed_expr.ty = Type::Bool;
                }
                BinaryOp::Eq | BinaryOp::Ne => {
                    let _ = left.ty.unify(&right.ty)?;
                    typed_expr.ty = Type::Bool;
                }
            }
        }
    }
    Ok(())
}
