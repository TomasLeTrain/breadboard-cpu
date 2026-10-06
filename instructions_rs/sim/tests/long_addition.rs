use assembler::parse_string;

use sim::*;

fn run_program(source: String) -> Result<CpuState, String> {
    let mut state = CpuState::new();

    state.load_opcode_roms("../opcode_gen/rom0.bin", "../opcode_gen/rom1.bin");

    let asm_context = parse_string(source);

    if let Ok(asm_context) = asm_context {
        println!("{}",asm_context.format_pretty());
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

fn test_addition_source(lhs: u32, rhs: u32, func: String) {
    let lhs0 = lhs & 0xff;
    let lhs1 = (lhs >> 8) & 0xff;
    let lhs2 = (lhs >> 16) & 0xff;
    let lhs3 = (lhs >> 24) & 0xff;

    let rhs0 = rhs & 0xff;
    let rhs1 = (rhs >> 8) & 0xff;
    let rhs2 = (rhs >> 16) & 0xff;
    let rhs3 = (rhs >> 24) & 0xff;

    let mut source = std::format!(
        "
start:
	; initialize stack
	lda SP, 0xffe0 ; lda sp, sp_start_addr

	; guarantee a clear of carry flag
	mv A, 0
	add A, 0

	; push return addr fib before data
	pusha push_return

	; push the left and right nums
	; right first
	push {rhs3}
	push {rhs2}
	push {rhs1}
	push {rhs0}

	; push the left and right nums
	; left second
	push {lhs3}
	push {lhs2}
	push {lhs1}
	push {lhs0}

	jmp func, MAR     
	push_return:

	; retrieve result from stack into registers
	pop X
	pop Y
	pop Z
	pop A

	halt\n"
    );

    let expected_result = (lhs as u64) + (rhs as u64);
    let expected0 = expected_result & 0xff;
    let expected1 = (expected_result >> 8) & 0xff;
    let expected2 = (expected_result >> 16) & 0xff;
    let expected3 = (expected_result >> 24) & 0xff;

    source.push_str(func.as_str());

    let final_state = run_program(source).unwrap();
    assert_eq!(final_state.x.state(), Some(expected3 as u8));
    assert_eq!(final_state.y.state(), Some(expected2 as u8));
    assert_eq!(final_state.z.state(), Some(expected1 as u8));
    assert_eq!(final_state.a.state(), Some(expected0 as u8));
}

#[test]
fn unrolled_long_addition() {
    let func = "
; unrolled and optimized 32-bit sum, with inputs given from stack
func {
	; 0
	pop A
	sw A, 0x8080, MAR ; loads 0x8080 into MAR once 
	; 1
	inc MAR
	pop A
	sw A, MAR 
	; 2
	inc MAR
	pop A
	sw A, MAR 
	; 3
	inc MAR
	pop A
	sw A, MAR 
	; 4
	inc MAR
	pop A
	sw A, MAR 
	; 5
	inc MAR
	pop A
	sw A, MAR 
	; 6
	inc MAR
	pop A
	sw A, MAR 
	; 7
	inc MAR
	pop A
	sw A, MAR 

	; want to store result to stack, so must save return addr elsewhere
	; get return address and save it to memory, next step changes stack
	pop	A
	sw A, 0xfff0, MAR
	inc MAR
	pop	A
	sw A, MAR ; sw A, 0xfff1, MAR

	; get nums
	lw X, 0x8080, MAR
	lw Y, 0x8084, MAR
	; add nums
	; NOTE: no carry on first add
	add X, Y
	; push result to stack
	push X

	; get nums
	inc MAR
	lw Y, MAR
	lw X, 0x8081, MAR
	; add nums
	adc X, Y
	; push result to stack
	push X

	; get nums
	inc MAR
	lw X, MAR
	lw Y, 0x8086, MAR
	; add nums
	adc X, Y
	; push result to stack
	push X

	; get nums
	inc MAR
	lw Y, MAR
	lw X, 0x8083, MAR
	; add nums
	adc X, Y
	; push result to stack
	push X

	; get return address back
	lw A, 0xfff0, MAR
	; since next addr is right after can just increase MAR and load it into B
	; both approaches take the exact same # of clock cycles, but this approach is one less byte long
	inc MAR
	lw B, MAR ; lw B, 0xfff1, MAR

	; load into mar
	mv MarLo, A
	mv MarHi, B

	; return
	jmp MAR
}\n";

    // some edge cases
    test_addition_source(0, 0, func.to_string());

    test_addition_source(u32::MAX, 0, func.to_string());
    test_addition_source(0, u32::MAX, func.to_string());

    test_addition_source(u32::MAX, 1, func.to_string());
    test_addition_source(1, u32::MAX, func.to_string());

    // some random tests
    for i in (1..u32::MAX).step_by(1234567) {
        for j in (1..u32::MAX).step_by(1234567) {
            test_addition_source(i, j, func.to_string());
        }
    }
}
