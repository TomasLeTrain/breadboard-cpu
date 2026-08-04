mod ast;
mod parser;
mod types;

mod comp;
mod link;
mod eval;

use std::fs;

// Structure:
// - parsing
//    - parse ast from file
// - compilation
//    - find and keep track of local/global symbols(vars and labels)
//    - generate instructions with expressions to yet be resolved
//    - typecheck expressions
// - linking
//    - give a value to all symbols
// - (const) evaluation - must happen after linking to make sure all symbols have values
//    - eval all expressions and symbols
//    - ensure types are correct

fn main() {
    let asm_file = fs::read_to_string("src/program.asm").expect("cannot read file");

    let program = parser::parse(&asm_file);

    // pretty print the parse error
    if let Err(e) = program {
        println!("{}", e);
    } else {
        println!("{:#?}", program);
    }
}
