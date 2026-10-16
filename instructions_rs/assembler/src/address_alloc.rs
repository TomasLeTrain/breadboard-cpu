use std::fs;

use crate::{
    ast::{Directive, InlinedFunctionCall, StatementKind, StatementNode},
    eval::ExprValue,
    types::Address,
};
use miette::{Context, IntoDiagnostic, Result, miette};

// TODO: keep track of used spaces to catch conflicts or perform that later?
pub struct AllocationContext {
    current_addr: Address,
}

impl AllocationContext {
    pub fn new() -> Self {
        Self { current_addr: 0 }
    }

    pub fn address(&self) -> u16 {
        self.current_addr
    }

    pub fn advance_address(&mut self, n: u16) {
        self.current_addr += n;
    }

    pub fn set_address(&mut self, n: u16) {
        self.current_addr = n;
    }
}

pub fn allocate_adresses(
    statements: &mut [StatementNode],
    ctx: &mut AllocationContext,
) -> Result<()> {
    for statement in statements.iter_mut() {
        statement.inner_mut().set_address(Some(ctx.address()));

        match statement.inner_mut().inner_mut() {
            StatementKind::Label { .. } => {
                // label does not advance address
            }
            StatementKind::Directive(Directive { name, params }) => {
                if name == "address" {
                    // TODO: add errors, add checks that directive is of proper form
                    let addr = params.first().unwrap().inner.value.as_int()?;
                    ctx.set_address(addr as u16);
                }

                // TODO: duplicate code for address alloc and asm_gen
                if name == "data" || name == "data_file" {
                    let expr_value = &params.first().unwrap().inner().value;

                    let data = if name == "data_file" {
                        // get data from file
                        match &expr_value {
                            ExprValue::String(file_path) => {
                                let file = fs::read_to_string(file_path.clone())
                                    .into_diagnostic()
                                    .wrap_err("Failed reading file for data")?;
                                Vec::from(file.as_bytes())
                            }
                            _ => Err(miette!("data_file directive does not have path"))?,
                        }
                    } else {
                        // data stays as data, but must convert to byte representation
                        expr_value.as_bytes()?
                    };

                    // reserve space for this data
                    ctx.advance_address(data.len() as u16);
                }
            }

            StatementKind::BlockLabel { body, .. }
            | StatementKind::Block { body }
            | StatementKind::InlinedFunctionCall(InlinedFunctionCall { body, .. }) => {
                // label does not advance address
                // allocate addresses inside body
                allocate_adresses(body, ctx)?;
            }
            StatementKind::Instruction(ast_instruction) => {
                // advance by however many bytes instruction takes up
                let istr_size = ast_instruction
                    .instruction
                    .as_ref()
                    .unwrap()
                    .get_byte_size();
                ctx.advance_address(istr_size as u16);
            }
            // nothing else requires addr alloc
            _ => (),
        };
    }

    Ok(())
}
