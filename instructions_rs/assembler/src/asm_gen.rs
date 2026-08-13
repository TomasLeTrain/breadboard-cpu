use crate::{
    ast::{StatementKind, StatementNode},
    eval::ExprValue,
    types::Address,
};
use miette::{Result, miette};
use opcode_gen::instructions::ArgumentValue;

/// simple vector of bools implemented as packed bytes
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

// TODO: keep track of used spaces to catch conflicts or perform that later?
pub struct AsmGenContext {
    assembly: Vec<u8>,
    addr_occupied: BoolVec,
}

impl AsmGenContext {
    pub fn new(max_addr_size: u16) -> Self {
        Self {
            assembly: vec![0; max_addr_size as usize],
            addr_occupied: BoolVec::new(max_addr_size as usize),
        }
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

    pub fn into_assembly(self) -> Vec<u8> {
        self.assembly
    }
}

pub fn generate_asm(statements: &[StatementNode], ctx: &mut AsmGenContext) -> Result<()> {
    for statement in statements.iter() {
        if let StatementKind::Instruction(ast_instruction) = statement.inner().inner() {
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

            ctx.place_bytes(statement.inner().address().unwrap(), &istr_bytes)?;
        };
    }

    Ok(())
}
