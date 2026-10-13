; snake program

; Goals:
; - have circular queue of positions, can represent 16x16 in one byte

; where the current snake direction gets stored
let pDIR = 0
; game runs until == 0
let pHALT = 0

; upper byte of queue
let pQUEUE_BASE = 0
; read pointer - lower byte of whole addr
let pQUEUE_READ = 0
; write pointer - lower byte of whole addr
let pQUEUE_WRITE = 0


start {
	; initialize stack
	lda SP, 0xffe0 ; lda sp, sp_start_addr

	; guarantee a clear of carry flag
	mv A, 0
	add A, 0

	; main loop
	loop { 
		jsr(update_input)
		jsr(update_state)
		jsr(update_screen)

		lw A, pHALT, MAR
		jnz A, MAR
	}

	halt
}

update_input {
	; TODO: actually get input
	mw A, 1
	sw A, pDIR, MAR

	pop MAR
	jmp MAR
}

update_state {
	; get head of snake (end of queue)
	lw A, pQUEUE_WRITE, MAR
	lw B, pQUEUE_BASE, MAR

	mv MarHi, B
	mv MarLo, A

	lw X, MAR
	; mask last 4 bits
	and X, 0b00001111

	lw Y, MAR
	; mask first 4 bits
	and Y, 0b11110000
	shr Y, 4

	lw Z, pDIR, MAR

	cmp Z, 0
	jeq left_ret
	; if left -> X -= 1
	sub X, 1
	left_ret:

	; if right -> X += 1
	cmp Z, 1
	jeq right_ret
	add X, 1
	right_ret:
	; if down -> Y -= 1
	cmp Z, 2
	jeq down_ret
	sub Y, 1
	down_ret:
	; if up -> X += 1
	cmp Z, 3
	jeq up_ret
	add Y, 1
	up_ret:

	; increase write pointer, and write to it
	lw A, pQUEUE_WRITE, MAR
	add A, 1
	sw A, pQUEUE_WRITE, MAR

	lw B, pQUEUE_BASE, MAR
	mv MarLo, A

	; shift left again
	shl Y, 4

	; make new byte
	or X, Y

	; store to queue
	sw X, pQUEUE_BASE, MAR

	; NOTE: first in queue gets popped when updating screen

	pop MAR
	jmp MAR
}

update_screen {
	lw A, pQUEUE_READ, MAR
	lw B, pQUEUE_BASE, MAR

	mv MarHi, B
	mv MarLo, A

	; pop previous
	lw A, pQUEUE_READ, MAR
	add A, 1
	sw A, pQUEUE_READ, MAR

	pop MAR
	jmp MAR
}


fn jsr(addr: Address) {
	return {
		pusha return_addr
		jmp addr, MAR
		return_addr:
	}
}
