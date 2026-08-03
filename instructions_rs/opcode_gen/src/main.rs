mod action;
mod instructions;
mod opcode;
mod output;
mod step_template;

use crate::instructions::OpcodeToOutput;
use std::{fs::File, fs::OpenOptions, io::Write};

fn main() -> std::io::Result<()> {
    let istr_set = instructions::build_all_instructions();
    println!("{istr_set}");

    let mut rom0_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("rom0.bin")
        .unwrap();
    let mut rom1_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("rom1.bin")
        .unwrap();

    let rom_data: (Vec<_>, Vec<_>) = (0..(1 << 17))
        .into_iter()
        .map(|i| {
            let opcode = opcode::addr_to_opcode(i);
            let data = istr_set.to_output(opcode).get_output_data();
            (data as u8, (data >> 8) as u8)
        })
        .unzip();

    // FIXME: deal with errors
    rom0_file.write_all(&rom_data.0)?;
    rom1_file.write_all(&rom_data.1)?;

    Ok(())
}
