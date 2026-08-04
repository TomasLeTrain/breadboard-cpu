/// generates assembly from the ast
/// At this point all symbols get found

use std::collections::HashMap;

use crate::ast::{Program, Statement};

// TODO: add labels
// TODO: add instructions
struct AsmGen {
    symbols: HashMap<String, Symbol>,
}

struct Symbol {
    name: String,
}

impl AsmGen {
    fn compile(&mut self, program: &Program) -> Result<(), String> {
        // can compile everything without regard for order since linking happens after
        for statement in program {
            self.compile_statement(statement)?;
        }

        Ok(())
    }

    fn compile_statement(&mut self, statement: &Statement) -> Result<(), String> {
        match statement {
            Statement::Label { name } => {
                todo!()
            }
            Statement::BlockLabel { body, .. } => self.compile(body)?,
            Statement::Instruction { name, params } => {
                todo!()
            }
        }
        Ok(())
    }

    fn symbols(&self) -> &Vec<String, Symbol> {
        &self.symbols
    }
}
