//! Errors if directive is unkown. Directives can apply at various stages, so any such errors are
//! catched here.

use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use miette::{Result, miette};
use opcode_gen::instructions::{
    AddressRegister, ArgumentType, Instruction, InstructionSignature, Register,
};

use crate::{
    ast::{AstNode, Directive, Expr, InlinedFunctionCall, StatementKind, StatementNode},
    error::ParseError,
    eval::ExprValue,
    types::Type,
};

// TODO: ensure directive has correct types here?

pub fn check_directives(
    statements: &mut [StatementNode],
    known_directives: &HashSet<String>,
) -> Result<()> {
    for statement in statements {
        match statement.inner_mut().inner_mut() {
            StatementKind::BlockLabel { body, .. }
            | StatementKind::Block { body }
            | StatementKind::InlinedFunctionCall(InlinedFunctionCall { body, .. }) => {
                check_directives(body, known_directives)?;
            }
            StatementKind::Directive(Directive { name, .. })
                if !known_directives.contains(name) =>
            {
                // TODO: detailed error
                Err(miette!("Unkown directive \"{name}\""))?;
            }
            _ => (),
        }
    }
    Ok(())
}
