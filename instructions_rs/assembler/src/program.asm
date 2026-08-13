start:
	; initialize stack
	lda SP, 0xffe0 ; lda sp, sp_start_addr

	; set up variables for fib routine
	
	; TODO: this only works since we know the push_return will be smaller than 256 - need to implement addr placeholder 
	push 0           ; push return addr (high byte)
	push push_return ; push return addr (low byte)
	jmp fib, MAR     ; jump to func

	push_return:
	halt


; x = x + y
; y = x + y
; ...
fib {
	; initial conditions
	mv X, 0
	mv Y, 1
	mv Z, 5

	loop {
		add X, Y
		add Y, X

		lda MAR, loop
		jnz Z, MAR
	}

	; return routine
	pop MAR
	jmp MAR
}
