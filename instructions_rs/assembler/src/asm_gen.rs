use std::{cmp::Ordering, fmt::Display};

use crate::{
    ast::{AstInstruction, AstNode, StatementKind, StatementNode},
    eval::ExprValue,
    types::Address,
};
use miette::{Result, miette};
use opcode_gen::instructions::ArgumentValue;

/// simple vector of bools implemented as packed bytes
#[derive(Debug)]
struct BoolVec {
    vec: Vec<u8>,
}

impl BoolVec {
    fn new(capacity: usize) -> Self {
        Self {
            vec: vec![0; Self::get_vec_idx(capacity)],
        }
    }

    fn get_vec_idx(i: usize) -> usize {
        i >> 3
    }

    fn get_byte_idx(i: usize) -> u8 {
        1 << (i & 0b111)
    }

    fn set(&mut self, i: usize) -> Result<()> {
        assert!(!self.get(i)?);

        let value = self
            .vec
            .get_mut(Self::get_vec_idx(i))
            .ok_or(miette!("couldn't get index"))?;

        *value |= Self::get_byte_idx(i);

        Ok(())
    }

    fn get(&self, i: usize) -> Result<bool> {
        let value = self
            .vec
            .get(Self::get_vec_idx(i))
            .ok_or(miette!("couldn't get index"))?;

        let res = (value & Self::get_byte_idx(i)) != 0;

        Ok(res)
    }
}

#[derive(Debug, Clone)]
struct AsmSpan {
    statement: StatementNode,
    span: (u16, u16),
}

// TODO: keep track of used spaces to catch conflicts or perform that later?
#[derive(Debug)]
pub struct AsmGenContext {
    assembly: Vec<u8>,
    addr_occupied: BoolVec,
    istr_slices: Vec<AsmSpan>,
}

impl AsmGenContext {
    pub fn new(max_addr_size: u16) -> Self {
        Self {
            assembly: vec![0; max_addr_size as usize],
            addr_occupied: BoolVec::new(max_addr_size as usize),
            istr_slices: Vec::new(),
        }
    }

    fn get_byte(&self, addr: Address) -> Result<u8> {
        Ok(*self
            .assembly
            .get(addr as usize)
            .ok_or(miette!("couldn't get byte"))?)
    }

    fn place_byte(&mut self, addr: Address, byte: u8) -> Result<()> {
        let asm_byte = self
            .assembly
            .get_mut(addr as usize)
            .ok_or(miette!("couldnt place at addr"))?;

        *asm_byte = byte;

        self.addr_occupied.set(addr as usize)?;
        Ok(())
    }

    fn place_bytes(&mut self, addr: Address, bytes: &[u8]) -> Result<()> {
        let end_addr = addr + bytes.len() as u16;

        for (&byte, i) in bytes.iter().zip(addr..end_addr) {
            self.place_byte(i, byte)?;
        }

        Ok(())
    }

    fn place_statement(
        &mut self,
        statement: StatementNode,
        addr: Address,
        bytes: &[u8],
    ) -> Result<()> {
        if !bytes.is_empty() {
            self.place_bytes(addr, bytes)?;
        }
        let end_addr = addr + (bytes.len() as u16);
        self.istr_slices.push(AsmSpan {
            statement,
            span: (addr, end_addr),
        });
        Ok(())
    }

    pub fn into_assembly(self) -> Vec<u8> {
        self.assembly
    }

    fn get_sorted_spans(&self) -> Vec<AsmSpan> {
        let mut res = self.istr_slices.clone();
        res.sort_by_key(|a| a.span);
        res
    }

    fn get_bytes_from_span(&self, span: &AsmSpan) -> &[u8] {
        &self.assembly[(span.span.0 as usize)..(span.span.1 as usize)]
    }

    pub fn format_pretty(&self) -> String {
        let mut result = String::new();

        let spans = self.get_sorted_spans();

        result.push_str(&format!("Addr | {:^10} | Text Assembly\n", "Bytes"));
        result.push('\n');


        for span in spans.iter() {
            // addr  | bytes | program
            result.push_str(&format!("{:>4} | ", format!("{:X}", span.span.0)));

            let thing: Vec<_> = self
                .get_bytes_from_span(span)
                .iter()
                .map(|byte| format!("{byte:02X}"))
                .collect();

            let formatted_bytes = thing.join(" ");

            result.push_str(&format!("{:<10} | ", formatted_bytes));

            result.push_str(span.statement.span.get_line_str());
            result.push('\n');
        }

        result
    }
}

pub fn generate_asm(statements: &[StatementNode], ctx: &mut AsmGenContext) -> Result<()> {
    for statement in statements.iter() {
        match statement.inner().inner() {
            StatementKind::BlockLabel { body, .. } => {
                ctx.place_statement(statement.clone(), statement.inner().address().unwrap(), &[])?;
                generate_asm(body, ctx)?;
            }
            StatementKind::Instruction(ast_instruction) => {
                let arg_values: Vec<ArgumentValue> = ast_instruction
                    .params
                    .iter()
                    .map(|e| e.inner().value.as_istr_arg_value())
                    .collect();

                let istr_bytes = ast_instruction
                    .instruction
                    .as_ref()
                    .unwrap()
                    .get_asm_bytes(arg_values);

                ctx.place_statement(
                    statement.clone(),
                    statement.inner().address().unwrap(),
                    &istr_bytes,
                )?;
            }
            _ => (),
        };
    }

    Ok(())
}
