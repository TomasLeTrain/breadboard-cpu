use assembler::{
    asm_output::{AsmOutput, BinaryOutput, LogisimOutput},
    parse_file,
};
use miette::Result;

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

    let rom_size = 1 << 17;

    let asm = parse_file(file_path_str)?;

    LogisimOutput::new("asm_logisim.img")?.generate_output(asm.clone())?;
    BinaryOutput::new("asm_bin.bin", rom_size)?.generate_output(asm)?;

    Ok(())
}
