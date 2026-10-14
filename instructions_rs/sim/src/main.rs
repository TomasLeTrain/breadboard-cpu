use assembler::parse_file;
use sdl3::render::ScaleMode;
use sdl3::surface::Surface;
use sdl3::sys::keycode::SDL_Keymod;
use sim::{CpuState, DataRegister};

use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::{Color, Palette, PixelFormat};
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;

enum CpuMessage {
    Halt,
}

enum VideoMessage {
    KeyboardUpdate(u8),
    Halt,
}

fn main() {
    // init cpu state
    let mut state = CpuState::new();

    let video = Arc::clone(&state.video);

    state.load_opcode_roms("../opcode_gen/rom0.bin", "../opcode_gen/rom1.bin");

    let mut video_rom = Vec::new();

    File::open("../../vga_display_v1/vga_test_image.bin")
        .unwrap()
        .read_to_end(&mut video_rom)
        .unwrap();

    video.write().unwrap().load_image(video_rom);

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

    let (cpu_send, video_receieve) = mpsc::channel::<CpuMessage>();
    let (video_send, cpu_receive) = mpsc::channel::<VideoMessage>();

    let window_task = std::thread::spawn(move || {
        let sdl_context = sdl3::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            // .window("sim", 640 * 2, 480 * 2)
            .window("sim", 640, 480)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas();

        canvas.clear();
        canvas.present();

        let mut event_pump = sdl_context.event_pump().unwrap();

        'running: loop {
            canvas.clear();

            let mut some_keyboard = false;

            for scancode in event_pump.keyboard_state().pressed_scancodes() {
                let keycode = Keycode::from_scancode(scancode, SDL_Keymod::NONE, false).unwrap();
                if keycode.to_ll().0 <= 255 {
                    let code = keycode.to_ll().0 as u8;
                    let _ = video_send.send(VideoMessage::KeyboardUpdate(code));
                    some_keyboard = true;
                }
            }

            // nothing valid being held down, send 0 to signal
            if !some_keyboard {
                let _ = video_send.send(VideoMessage::KeyboardUpdate(0));
            }

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

            // draw image here
            let width = 320;
            let height = 240;

            let mut surface_data = video.read().unwrap().create_surface_data();
            let surface =
                Surface::from_data(&mut surface_data, width, height, width, PixelFormat::RGB332)
                    .unwrap();

            let texture_creator = canvas.texture_creator();

            let mut texture = texture_creator
                .create_texture_from_surface(surface)
                .unwrap();

            texture.set_scale_mode(ScaleMode::Nearest);

            canvas.copy(&texture, None, None).unwrap();

            canvas.present();
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
        // NOTE: cpu might have stopped before, so this can fail in expected cases
        let _ = video_send.send(VideoMessage::Halt);
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

        let mut video_halt = false;

        loop {
            match cpu_receive.try_recv() {
                Ok(msg) => match msg {
                    VideoMessage::KeyboardUpdate(code) => state.keyboard.set_code(code),
                    VideoMessage::Halt => {
                        video_halt = true;
                        break;
                    }
                },
                Err(err) => match err {
                    TryRecvError::Empty => break,
                    TryRecvError::Disconnected => {
                        video_halt = true;
                        break;
                    }
                },
            }
        }

        if state.is_halt() || video_halt {
            break;
        }
        i += 1;
    }

    // halt
    // NOTE: can fail if video stopped before cpu, so not catched
    let _ = cpu_send.send(CpuMessage::Halt);
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
