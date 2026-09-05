start:
	; initialize stack
	lda SP, 0xffe0 ; lda sp, sp_start_addr

	; set up variables for fib routine

	; this is some more comments
	; these are next to each other
	
	; TODO: this only works since we know the push_return will be smaller than 256 - need to implement addr placeholder 
	push push_return

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

	mv Z, 5

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

fn function_test(what: u8, addr: u16) {
}
