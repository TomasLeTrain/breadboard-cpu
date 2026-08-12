use crate::ast::{AddressedStatement, AstNode, Statement, StatementNode};
use miette::Result;

struct AllocationContext {
    current_addr: usize,
}

pub fn allocate_adresses(statements: &mut [StatementNode]) -> Result<()> {
    Ok(())
    // let mut res = Vec::new();
    //
    // // start at placing at addr 0
    // let mut address = 0;
    //
    // for statement in statements.into_iter() {
    //     let curr_addr = match statement.inner() {
    //         Statement::Label { name } => {
    //             // placed at byte right after current address
    //             let res = address;
    //             address += 1;
    //             res
    //         }
    //         Statement::BlockLabel { name, body } => {
    //             // placed at byte right after current address
    //             let res = address;
    //             // now
    //             address += 1;
    //             res
    //         }
    //         Statement::Instruction(ast_instruction) => todo!(),
    //     };
    //
    //     let span = statement.span().clone();
    //
    //     res.push(AstNode::new(
    //         AddressedStatement::new(statement.into_inner(), Some(curr_addr)),
    //         span,
    //     ));
    // }
    //
    // Ok(res)
}
