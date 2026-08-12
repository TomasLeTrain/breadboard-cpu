use std::collections::HashMap;

use miette::{Context, Result, miette};
use opcode_gen::instructions::{AddressRegister, Register};

use crate::{
    ast::{AstNode, AstSpan, Expr, Statement, StatementNode, TypedExpr},
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
        if let Statement::Label { name } | Statement::BlockLabel { name, .. } =
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
            Statement::BlockLabel { body, .. } => {
                eval_program(body, ctx)?;
            }
            Statement::Instruction(instruction) => {
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

fn eval_expr(typed_expr: &mut AstNode<TypedExpr>, ctx: &mut EvalContext) -> Result<()> {
    let inner_span = typed_expr.span().clone();
    let inner = typed_expr.inner_mut();

    match &mut inner.expr {
        // literals already have their value filled in
        Expr::Literal => (),
        Expr::Identity(name) => {
            // try and find identity in symbols
            // if ctx.contains(name) {
            //     if !matches!(inner.ty, Type::Unknown) {
            //         Err(TypecheckExprError::new(
            //             TypecheckExprErrorKind::IdentityAlreadyTyped((inner_span, inner.ty)),
            //         ))?;
            //     }
            //
            //     inner.ty = ctx.get(name).unwrap().symbol_type;
            // } else {
            //     Err(TypecheckExprError::new(
            //         TypecheckExprErrorKind::SymbolNotFound(Symbol {
            //             name: name.to_string(),
            //             symbol_type: inner.ty,
            //             span: Some(typed_expr.span.clone()),
            //         }),
            //     ))?;
            // }
        }
        Expr::Unary {
            op,
            expr: unary_expr,
        } => {
            // typecheck_expr(unary_expr, symbols)?;
            // let span = unary_expr.span().clone();
            // let unary_expr = unary_expr.inner_mut();
            //
            // match op {
            //     UnaryOp::Neg | UnaryOp::BitNegation => {
            //         if !unary_expr.ty.int_operable() {
            //             Err(TypecheckExprError::new(
            //                 TypecheckExprErrorKind::InvalidUnaryOpType((span, unary_expr.ty), *op),
            //             ))?;
            //         }
            //         inner.ty = Type::Int;
            //     }
            //     UnaryOp::Not => {
            //         if !unary_expr.ty.bool_operable() {
            //             Err(TypecheckExprError::new(
            //                 TypecheckExprErrorKind::InvalidUnaryOpType((span, unary_expr.ty), *op),
            //             ))?;
            //         }
            //         inner.ty = Type::Bool;
            //     }
            // }
        }
        Expr::Binary { op, left, right } => {
            // typecheck_expr(left, symbols)?;
            // typecheck_expr(right, symbols)?;
            //
            // let left_span = left.span().clone();
            // let right_span = right.span().clone();
            //
            // let left = left.inner_mut();
            // let right = right.inner_mut();
            //
            // match op {
            //     BinaryOp::Add
            //     | BinaryOp::Sub
            //     | BinaryOp::Mul
            //     | BinaryOp::Div
            //     | BinaryOp::Mod
            //     | BinaryOp::Pow
            //     | BinaryOp::ShiftLeft
            //     | BinaryOp::ShiftRight
            //     | BinaryOp::BitAnd
            //     | BinaryOp::BitXor
            //     | BinaryOp::BitOr => {
            //         if !Type::int_binary_operable(&left.ty, &right.ty) {
            //             Err(TypecheckExprError::new(
            //                 TypecheckExprErrorKind::InvalidBinaryOpTypes(
            //                     (left_span, left.ty),
            //                     (right_span, right.ty),
            //                     *op,
            //                 ),
            //             ))?;
            //         }
            //         inner.ty = left.ty.unify(&right.ty).unwrap();
            //     }
            //     BinaryOp::And | BinaryOp::Or => {
            //         if !Type::bool_binary_operable(&left.ty, &right.ty) {
            //             Err(TypecheckExprError::new(
            //                 TypecheckExprErrorKind::InvalidBinaryOpTypes(
            //                     (left_span, left.ty),
            //                     (right_span, right.ty),
            //                     *op,
            //                 ),
            //             ))?;
            //         }
            //         inner.ty = left.ty.unify(&right.ty).unwrap();
            //     }
            //     BinaryOp::Lt | BinaryOp::Gt | BinaryOp::Le | BinaryOp::Ge => {
            //         if !Type::comparable(&left.ty, &right.ty) {
            //             Err(TypecheckExprError::new(
            //                 TypecheckExprErrorKind::InvalidComparisonTypes(
            //                     (left_span, left.ty),
            //                     (right_span, right.ty),
            //                 ),
            //             ))?;
            //         }
            //         inner.ty = Type::Bool;
            //     }
            //     BinaryOp::Eq | BinaryOp::Ne => {
            //         if left.ty.unify(&right.ty).is_none() {
            //             Err(TypecheckExprError::new(
            //                 TypecheckExprErrorKind::InvalidEqualityTypes(
            //                     (left_span, left.ty),
            //                     (right_span, right.ty),
            //                 ),
            //             ))?;
            //         }
            //         inner.ty = Type::Bool;
            //     }
            // }
        }
    }
    Ok(())
}
