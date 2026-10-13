use assembler::parse_file;
use sim::{CpuState, DataRegister};

use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;

fn main() {
    // init cpu state
    let mut state = CpuState::new();

    state.load_opcode_roms("../opcode_gen/rom0.bin", "../opcode_gen/rom1.bin");

    let asm_context = parse_file("src/test_program.asm");

    if let Ok(asm_context) = asm_context {
        println!("{}", asm_context.format_pretty());
        state.load_program(asm_context.assembly().clone());
    } else {
        eprintln!("Error parsing file:");
        eprintln!("{:?}", asm_context.unwrap_err());
        return;
    }

    state.reset();

    let (cpu_send, video_receieve) = mpsc::channel::<()>();
    let (video_send, cpu_receive) = mpsc::channel::<()>();

    let window_task = std::thread::spawn(move || {
        let sdl_context = sdl3::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("sim", 640, 480)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas();

        canvas.set_draw_color(Color::RGB(0, 255, 255));
        canvas.clear();
        canvas.present();
        let mut event_pump = sdl_context.event_pump().unwrap();
        let mut i = 0;
        'running: loop {
            i = (i + 1) % 255;
            canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
            canvas.clear();
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    _ => {}
                }
            }
            // The rest of the game loop goes here...
            match video_receieve.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    println!("Terminating.");
                    break 'running;
                }
                Err(TryRecvError::Empty) => {}
            }

            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
        // NOTE: cpu might have stopped before, so this can fail in expected cases
        let _ = video_send.send(());
    });
    use std::time::Instant;

    eprintln!("started");
    let start = Instant::now();

    let mut i = 1;
    loop {
        // println!();
        // println!("doing cycle {i}");

        // perform full clock cycle
        state.step_half_clk();
        state.step_half_clk();

        // println!(
        //     "0x8070..0x8090: {:#x?}",
        //     &state.data_ram.state()[0x0070..0x0090]
        // );
        //
        // let mut stack = Vec::new();
        // state.data_ram.state()[(0xffa0 - 0x8000)..(0xffe0 - 0x8000)].clone_into(&mut stack);
        //
        // println!("stack - 0xffe0..0xffa0: {:#x?}", stack);

        let video_halt = matches!(
            cpu_receive.try_recv(),
            Ok(_) | Err(TryRecvError::Disconnected)
        );

        if state.is_halt() || video_halt {
            break;
        }
        i += 1;
    }

    // halt
    // NOTE: can fail if video stopped before cpu, so not catched
    let _ = cpu_send.send(());
    let _ = window_task.join();

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
    println!("flags: {:?}", state.flags.state());

    println!("pc: 0x{:x?}", state.pc.state().unwrap());
    println!("mar: 0x{:x?}", state.mar.state().unwrap());
    println!("sp: 0x{:x?}", state.sp.state().unwrap());
}
