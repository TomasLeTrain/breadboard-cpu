mod ast;
mod parser;
mod types;

use std::fs;

fn main() {
    let asm_file = fs::read_to_string("src/program.asm").expect("cannot read file");

    let program = parser::parse(&asm_file);

    // pretty print the parse error
    if let Err(e) = program {
        println!("{}", e);
    }else{
        println!("{:#?}", program);
    }
}
