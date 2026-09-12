use assembler::{
    asm_output::{AsmOutput, BinaryOutput, LogisimOutput},
    parse_file,
};
use miette::Result;


// if function is macro:
// inline function block inside call - should only happen if is emitting assembly, cannot call inside isntruction param / other expr (what aobut storing some asm in a variable?)
// inside block only consider context from params 
// else:
// eval during eval stage
//
// with addr:
// label
// istr
// blocked label
// everything else is not

fn main() -> Result<()> {
    let file_path_str = "src/program.asm";

    let rom_size = 1 << 17;

    let asm = parse_file(file_path_str)?;

    LogisimOutput::new("asm_logisim.img")?.generate_output(asm.clone())?;
    BinaryOutput::new("asm_bin.bin", rom_size)?.generate_output(asm)?;

    Ok(())
}
