mod address_alloc;
mod ast;
mod error;
mod istr_resolver;
mod parser;
mod types;

use std::{fs, rc::Rc, sync::Arc};

use miette::{Context, Result};
use opcode_gen::{
    get_instruction_list,
    instructions::{AddressRegister, Instruction, Register},
};

use crate::{
    address_alloc::AllocationContext,
    ast::NamedSourceFile,
    istr_resolver::gen_instruction_lookup_table,
    types::{Symbol, Type},
};

// Structure:
// ast: parsing
// typechecked symbols: ast
//  - traverse ast and find all indentifiers and give them types (register, const, etc.)
//  - resolve all types in ast
// instruction signatures: typechecking
//  - resolve all instruction signatures in ast
// address allocation: instruction signatures
//  - allocate addresses based on instruction signatures and address directives
// labels: address allocation
//  - resolve values for all labels based on
// resolving expressions: labels
//  - eval all expressions and resolve values in ast
// emitting assembly: resolving expressions
//  - emit assembly from ast
fn main() -> Result<()> {
    let file_path_str = "src/program.asm";

    parse_file(file_path_str)?;

    Ok(())
}

// wrapper for parse_source that adds the source_code if errors occur
fn parse_file(file_path_str: &str) -> Result<()> {
    let file_path = file_path_str.to_string();
    let asm_file = fs::read_to_string(file_path.clone()).expect("cannot read file");

    parse_source(asm_file, file_path)
}

//
fn parse_source(file: String, file_path: String) -> Result<()> {
    let source = Arc::new(NamedSourceFile::new(file, file_path));

    let mut program = parser::parse_file(source).wrap_err("Parsing file failed.")?;

    let mut global_symbols = types::SymbolTypeContext::new();

    // add global symbols reserved for register names and the like

    for reg in Register::iterator() {
        global_symbols.push(Symbol {
            name: reg.name().to_string(),
            symbol_type: Type::Register,
            span: None,
        })?;
    }

    for reg in AddressRegister::iterator() {
        global_symbols.push(Symbol {
            name: reg.name().to_string(),
            symbol_type: Type::AddressRegister,
            span: None,
        })?;
    }

    types::typecheck(&mut program, &mut global_symbols).wrap_err("Typechecking failed.")?;

    let all_istrs: Vec<Rc<Instruction>> = get_instruction_list().into_iter().map(Rc::new).collect();
    let istr_lookup = gen_instruction_lookup_table(&all_istrs)
        .wrap_err("Failed generating instruction lookup table")?;

    istr_resolver::resolve_instructions(&mut program, &istr_lookup)
        .wrap_err("Failed to resolve instructions")?;

    address_alloc::allocate_adresses(&mut program, &mut AllocationContext::new())
        .wrap_err("Failed to resolve instructions")?;

    println!("{:#?}", program);

    Ok(())
}
