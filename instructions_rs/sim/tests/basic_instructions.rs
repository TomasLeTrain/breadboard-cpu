use assembler::parse_string;

use sim::*;

// TODO: add  fib test:
// ; fib {
// ; 	; initial conditions
// ; 	mv X, 0
// ; 	mv Y, 1
// ;
// ; 	mv Z, 255
// ;
// ; 	lda MAR, loop
// ;
// ; 	loop {
// ; 		add X, Y
// ; 		add Y, X
// ;
// ; 		sub Z, 1
// ;
// ; 		jnz Z, MAR
// ; 	}
// ;
// ;
// ; 	; return routine
// ; 	pop MAR
// ; 	jmp MAR
// ; }

fn run_program(source: String) -> Result<CpuState, String> {
    let mut state = CpuState::new();

    state.load_opcode_roms("../opcode_gen/rom0.bin", "../opcode_gen/rom1.bin");

    let asm_context = parse_string(source);

    if let Ok(asm_context) = asm_context {
        state.load_program(asm_context.assembly().clone());
    } else {
        eprintln!("Error parsing file:");
        eprintln!("{:?}", asm_context.unwrap_err());
        return Err("parsing error".to_string());
    }

    state.reset();

    let mut i = 1;
    loop {
        // perform full clock cycle
        state.step_half_clk();
        state.step_half_clk();

        // TODO: make i limit adjustable
        if state.is_halt() || i > 1_000_000 {
            break;
        }
        i += 1;
    }
    Ok(state)
}

#[test]
fn mv_istr() {
    let source = "mv A, 10\n";
    let final_state = run_program(source.into()).unwrap();
    assert_eq!(final_state.a.state(), Some(10));
}

#[test]
fn mv_istr_2() {
    let source = " mv A, 10
            mv B, A\n";
    let final_state = run_program(source.into()).unwrap();
    assert_eq!(final_state.a.state(), Some(10));
    assert_eq!(final_state.b.state(), Some(10));
}

#[test]
fn add_istr() {
    let source = " mv X, 5
            mv Y, 10
            add X, Y
            \n";
    let final_state = run_program(source.into()).unwrap();
    assert_eq!(final_state.x.state(), Some(15));
    assert_eq!(final_state.y.state(), Some(10));
}

#[test]
fn add_istr_overflow() {
    let source = " mv X, 255
            mv Y, 1
            add X, Y
            \n";
    let final_state = run_program(source.into()).unwrap();
    assert_eq!(final_state.x.state(), Some(0));
    assert_eq!(final_state.y.state(), Some(1));
    assert_eq!(final_state.get_flag(Flags::InvCarry), Some(false));
}

#[test]
fn zero_flag() {
    let source = "mv A, 0
            cmp A, 0
            \n";
    let final_state = run_program(source.into()).unwrap();
    assert_eq!(final_state.a.state(), Some(0));
    assert_eq!(final_state.get_flag(Flags::InvZero), Some(false));
}
