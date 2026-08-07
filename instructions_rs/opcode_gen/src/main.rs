use std::{fs::OpenOptions, io::Write};

use opcode_gen::{
    instructions::{self, OpcodeToOutput},
    opcode::addr_to_opcode,
};

fn write_contents_logisim(data: &(Vec<u8>, Vec<u8>)) -> std::io::Result<()> {
    let mut rom0_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("rom0_logisim.img")?;
    let mut rom1_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("rom1_logisim.img")?;

    rom0_file.write_all(b"v3.0 hex words plain\n")?;
    rom1_file.write_all(b"v3.0 hex words plain\n")?;

    for (i, curr) in data.0.iter().zip(data.1.iter()).enumerate() {
        write!(rom0_file, "{:02x}", curr.0)?;
        write!(rom1_file, "{:02x}", curr.1)?;

        if i % 16 == 15 {
            writeln!(rom0_file)?;
            writeln!(rom1_file)?;
        } else {
            write!(rom0_file, " ")?;
            write!(rom1_file, " ")?;
        }
    }
    Ok(())
}

fn write_contents_binary(data: &(Vec<u8>, Vec<u8>)) -> std::io::Result<()> {
    let mut rom0_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("rom0.bin")?;
    let mut rom1_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("rom1.bin")?;

    rom0_file.write_all(&data.0)?;
    rom1_file.write_all(&data.1)?;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let (_all_istrs, istr_set) = instructions::build_all_instructions();
    println!("{istr_set}");
    println!("constructed instruction set, generating rom data...");

    let rom_data: (Vec<_>, Vec<_>) = (0..(1 << 17))
        .into_iter()
        .map(|i| {
            let opcode = addr_to_opcode(i);
            let data = istr_set.opcode_to_output(opcode).get_output_data();
            (data as u8, (data >> 8) as u8)
        })
        .unzip();

    println!("generated rom data, writing to files...");

    write_contents_binary(&rom_data)?;
    write_contents_logisim(&rom_data)?;

    Ok(())
}
