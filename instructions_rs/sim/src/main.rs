use crate::sim::{CpuState, DataRegister};
use assembler::parse_file;

mod sim;

fn main() {
    let mut state = CpuState::new();

    state.load_opcode_roms("../opcode_gen/rom0.bin", "../opcode_gen/rom1.bin");

    let asm = parse_file("src/test_program.asm");

    if let Ok(asm) = asm {
        state.load_program(asm);
    } else {
        eprintln!("Error parsing file:");
        eprintln!("{:?}", asm.unwrap_err());
        return;
    }

    state.reset();

    use std::time::Instant;

    eprintln!("started");
    let start = Instant::now();

    let mut i = 1;
    loop {
        println!();
        println!("doing cycle {i}");

        // perform full clock cycle
        state.step_half_clk();
        state.step_half_clk();

        println!(
            "0x8070..0x8090: {:#x?}",
            &state.data_ram.state()[0x0070..0x0090]
        );
        let mut stack = Vec::new();
        state.data_ram.state()[(0xffa0 - 0x8000)..(0xffe0 - 0x8000)].clone_into(&mut stack);

        println!("0xffe0..0xffa0: {:#x?}", stack);

        if state.is_halt() {
            break;
        }
        i += 1;
    }
    eprintln!(
        "ended with {i} cycles in {:.2?} nanoseconds",
        start.elapsed().as_nanos()
    );
    let nano_per_clk = (start.elapsed().as_nanos() as f32) / (i as f32);
    eprintln!("nanoseconds per clock cycle {nano_per_clk}");
    let hz = 1_000.0 / nano_per_clk;
    eprintln!("MHz: {hz}");

    println!();
    println!("halted!");

    // print state of cpu
    println!("a: {:?}", state.a.state());
    println!("b: {:?}", state.b.state());
    println!("x: {:?}", state.x.state());
    println!("y: {:?}", state.y.state());
    println!("z: {:?}", state.z.state());

    println!("pc: 0x{:x?}", state.pc.state().unwrap());
    println!("mar: 0x{:x?}", state.mar.state().unwrap());
    println!("sp: 0x{:x?}", state.sp.state().unwrap());
}
