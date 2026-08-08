mod ast;
mod istr_resolver;
mod parser;
mod types;

use std::fs;

use opcode_gen::instructions::{AddressRegister, Register};

use crate::types::{Symbol, Type};

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

fn main() {
    let asm_file = fs::read_to_string("src/program.asm").expect("cannot read file");

    let statements = parser::parse(&asm_file);

    // pretty print the parse error
    if let Err(e) = statements {
        println!("{}", e);
        return;
    }

    let mut program = statements.unwrap();

    let mut global_symbols = types::SymbolTypeContext::new();

    // add global symbols reserved for register names and the like

    for reg in Register::iterator() {
        global_symbols.push(Symbol {
            name: reg.name().to_string(),
            symbol_type: Type::Register,
        });
    }

    for reg in AddressRegister::iterator() {
        global_symbols.push(Symbol {
            name: reg.name().to_string(),
            symbol_type: Type::AddressRegister,
        });
    }

    types::typecheck(&mut program, &mut global_symbols);

    istr_resolver::resolve_istr_signatures(&mut program);

    // TODO: find the actual instructions based on the signatures

    println!("{:#?}", program);
}
