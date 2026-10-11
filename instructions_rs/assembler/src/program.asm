start:
	; initialize stack
	lda SP, 0xffe0 ; lda sp, sp_start_addr

	; set up variables for fib routine

	; this is some more comments
	; these are next to each other
	; pusha push_return
	push low_byte(push_return)
	push high_byte(push_return)


	jmp fib, MAR     ; jump to func

	push_return:
	mv A, X
	mv B, Y

	halt


; x = x + y
; y = x + y
; ...
fib {
	; initial conditions
	mv X, 0
	mv Y, 1

	let num_iters = 128 % 5

	mv Z, num_iters

	lda MAR, loop

	loop {
		add X, Y
		add Y, X
		
		sub Z, 1

		jnz Z, MAR
	}

	; return routine
	pop MAR
	jmp MAR
}

fn low_byte(x: u16) {
	let low_byte: u8 = x & 0xff
	return low_byte
}

fn high_byte(x: u16) {
	let high_byte: u8 = (x >> 8) & 0xff
	return high_byte
}
