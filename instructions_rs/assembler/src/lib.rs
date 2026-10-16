mod address_alloc;
mod asm_gen;
pub mod asm_output;
mod ast;
mod directive_checker;
mod error;
mod eval;
mod istr_resolver;
mod parser;
mod types;

use std::{collections::HashSet, fs, sync::Arc};

use miette::{Context, IntoDiagnostic, Result};
use opcode_gen::{
    get_instruction_list,
    instructions::{AddressRegister, Instruction, Register},
};

use crate::{
    address_alloc::AllocationContext,
    asm_gen::AsmGenContext,
    ast::{MultiFileSource, NamedSourceFile},
    eval::{EvalContext, EvalSymbol, ExprValue},
    istr_resolver::gen_instruction_lookup_table,
    types::{Symbol, Type},
};

pub fn parse_string(string_source: String) -> Result<AsmGenContext> {
    let file_path = "string".to_string();
    let mut source =
        MultiFileSource::new(vec![NamedSourceFile::new(string_source.clone(), file_path)]);

    // TODO: change for source index to avoid copies
    parse_source(&string_source, &mut source)
}

pub fn parse_file(file_path_str: &str) -> Result<AsmGenContext> {
    let file_path = file_path_str.to_string();
    let file = fs::read_to_string(file_path.clone())
        .into_diagnostic()
        .wrap_err("Failed reading file to parse")?;

    let mut source = MultiFileSource::new(vec![NamedSourceFile::new(file.clone(), file_path)]);
    parse_source(&file, &mut source)
}

fn parse_source(source_str: &str, source: &mut MultiFileSource) -> Result<AsmGenContext> {
    let mut program = parser::parse_source(source_str, source).wrap_err("Parsing file failed.")?;

    // println!("initial: {:#?}", program);

    let mut global_symbols = types::TypecheckContext::new();

    // add global symbols reserved for register names and the like

    for reg in Register::iterator() {
        global_symbols.push_global(Symbol {
            name: reg.name().to_string(),
            ty: Type::Register,
            span: None,
        })?;
    }

    for reg in AddressRegister::iterator() {
        global_symbols.push_global(Symbol {
            name: reg.name().to_string(),
            ty: Type::AddressRegister,
            span: None,
        })?;
    }

    // println!("initial: {:#?}", program);
    types::typecheck(&mut program, &mut global_symbols).wrap_err("Typechecking failed.")?;

    let mut directives = HashSet::new();
    directives.insert("address".to_string());

    directive_checker::check_directives(&mut program, &directives)?;

    // first eval stage, parts of the program will be unknown since labels don't have values yet
    let mut valued_symbols = EvalContext::new(true);

    for reg in Register::iterator() {
        valued_symbols.push_global(EvalSymbol {
            name: reg.name().to_string(),
            ty: Type::Register,
            value: ExprValue::Register(*reg),
            span: None,
        })?;
    }

    for reg in AddressRegister::iterator() {
        valued_symbols.push_global(EvalSymbol {
            name: reg.name().to_string(),
            ty: Type::AddressRegister,
            value: ExprValue::AddressRegister(*reg),
            span: None,
        })?;
    }

    // println!("before macro eval: {:#?}", program);
    eval::eval_program(&mut program, &mut valued_symbols).wrap_err("Failed to evaluate program")?;
    // println!("after macro eval: {:#?}", program);

    let all_istrs: Vec<Arc<Instruction>> =
        get_instruction_list().into_iter().map(Arc::new).collect();
    let istr_lookup = gen_instruction_lookup_table(&all_istrs)
        .wrap_err("Failed generating instruction lookup table")?;

    istr_resolver::resolve_instructions(&mut program, &istr_lookup)
        .wrap_err("Failed to resolve instructions")?;
    // println!("after instructions: {:#?}", program);

    address_alloc::allocate_adresses(&mut program, &mut AllocationContext::new())
        .wrap_err("Failed to allocate addresses")?;
    // println!("after addresses: {:#?}", program);

    // second eval, all values should be determined now
    let mut valued_symbols = EvalContext::new(false);

    for reg in Register::iterator() {
        valued_symbols.push_global(EvalSymbol {
            name: reg.name().to_string(),
            ty: Type::Register,
            value: ExprValue::Register(*reg),
            span: None,
        })?;
    }

    for reg in AddressRegister::iterator() {
        valued_symbols.push_global(EvalSymbol {
            name: reg.name().to_string(),
            ty: Type::AddressRegister,
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

    // println!("Annotated rogram:");
    // println!("{}", asm_context.format_pretty());

    Ok(asm_context)
}
