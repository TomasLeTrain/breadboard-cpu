use crate::ast::{Address, Statement, StatementNode};
use miette::Result;

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
            Statement::Label { .. } => {
                // label does not advance address
            }
            Statement::BlockLabel { body, .. } => {
                // label does not advance address
                // allocate addresses inside body
                allocate_adresses(body, ctx)?;
            }
            Statement::Instruction(ast_instruction) => {
                // advance by however many bytes instruction takes up
                let istr_size = ast_instruction
                    .instruction
                    .as_ref()
                    .unwrap()
                    .get_byte_size();
                ctx.advance_address(istr_size as u16);
            }
        };
    }

    Ok(())
}
