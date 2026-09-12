mod address_alloc;
mod asm_gen;
pub mod asm_output;
mod ast;
mod error;
mod eval;
mod istr_resolver;
mod parser;
mod types;

use std::{fs, rc::Rc, sync::Arc};

use miette::{Context, IntoDiagnostic, Result};
use opcode_gen::{
    get_instruction_list,
    instructions::{AddressRegister, Instruction, Register},
};

use crate::{
    address_alloc::AllocationContext,
    asm_gen::AsmGenContext,
    asm_output::{AsmOutput, BinaryOutput, LogisimOutput},
    ast::NamedSourceFile,
    eval::{EvalContext, EvalSymbol, ExprValue},
    istr_resolver::gen_instruction_lookup_table,
    types::{Symbol, Type},
};

pub fn parse_file(file_path_str: &str) -> Result<Vec<u8>> {
    let file_path = file_path_str.to_string();
    let file = fs::read_to_string(file_path.clone())
        .into_diagnostic()
        .wrap_err("Failed reading file to parse")?;

    let source = Arc::new(NamedSourceFile::new(file, file_path));

    let mut program = parser::parse_file(source).wrap_err("Parsing file failed.")?;

    // println!("initial: {:#?}", program);

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

    // println!("initial: {:#?}", program);
    types::typecheck(&mut program, &mut global_symbols).wrap_err("Typechecking failed.")?;
    // println!("after typecheck: {:#?}", program);

    let all_istrs: Vec<Rc<Instruction>> = get_instruction_list().into_iter().map(Rc::new).collect();
    let istr_lookup = gen_instruction_lookup_table(&all_istrs)
        .wrap_err("Failed generating instruction lookup table")?;

    istr_resolver::resolve_instructions(&mut program, &istr_lookup)
        .wrap_err("Failed to resolve instructions")?;
    // println!("after instructions: {:#?}", program);

    address_alloc::allocate_adresses(&mut program, &mut AllocationContext::new())
        .wrap_err("Failed to allocate addresses")?;
    // println!("after addresses: {:#?}", program);

    let mut valued_symbols = EvalContext::new();

    for reg in Register::iterator() {
        valued_symbols.push(EvalSymbol {
            name: reg.name().to_string(),
            symbol_type: Type::Register,
            value: ExprValue::Register(*reg),
            span: None,
        })?;
    }

    for reg in AddressRegister::iterator() {
        valued_symbols.push(EvalSymbol {
            name: reg.name().to_string(),
            symbol_type: Type::AddressRegister,
            value: ExprValue::AddressRegister(*reg),
            span: None,
        })?;
    }
    // println!("{:#?}", program);

    eval::eval_program(&mut program, &mut valued_symbols).wrap_err("Failed to evaluate program")?;
    // println!("after eval: {:#?}", program);

    // println!("{:#?}", program);

    let max_addr_size = 1 << 15;

    let mut asm_context = AsmGenContext::new(max_addr_size);

    asm_gen::generate_asm(&program, &mut asm_context)?;

    // println!("{:#?}", asm_context);

    println!("Annotated rogram:");
    println!("{}", asm_context.format_pretty());

    let asm = asm_context.into_assembly();

    Ok(asm)
}
