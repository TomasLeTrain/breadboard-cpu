; snake program

; Goals:
; - have circular queue of positions, can represent 16x16 in one byte
; - draw on screen
; - player input

; where the current snake direction gets stored
let pDIR = 0x9000
; game runs until == 0
let pHALT = 0x8080

; upper byte of queue
let pQUEUE_BASE = 0xA000

; read pointer - lower byte of whole addr
let pQUEUE_READ = 0x9020
; write pointer - lower byte of whole addr
let pQUEUE_WRITE = 0x9040


start {
	; initialize stack
	lda SP, 0xffe0 ; lda sp, sp_start_addr

	; guarantee a clear of carry flag
	mv A, 0
	add A, 0

	; start the queue state
	mv A, 0
	sw A, pQUEUE_READ, MAR
	mv A, 0
	sw A, pQUEUE_WRITE, MAR

	; add initial position at (2, 2)
	mv A, 0b0010 | (0b0010 << 4)
	sw A, pQUEUE_BASE, MAR

	mv A, 1
	sw A, pHALT, MAR

	; main loop
	loop { 
		; jsr(update_input)
		; jsr(update_state)
		; jsr(update_screen)

		lw A, pHALT, MAR
		cmp A, 0
		lda MAR, loop
		jnz MAR
	}

	halt
}

; clear_screen {
; }

update_input {
	; TODO: actually get input
	mv A, 1
	sw A, pDIR, MAR

	pop MAR
	jmp MAR
}

update_state {
	; get head of snake (end of queue)
	lw MarLo, pQUEUE_WRITE, MAR
	mv MarHi, high_byte(pQUEUE_BASE)

	lw X, MAR
	mv Y, X

	; mask last 4 bits
	and X, 0b00001111

	; mask first 4 bits
	and Y, 0b11110000
	shr Y
	shr Y
	shr Y
	shr Y

	lw Z, pDIR, MAR

	; W = 119
	; A =  97
	; S = 115
	; D = 100

	; if left -> X -= 1
	cmp Z, 97
	jeq left_eq, MAR
	jmp left_ret, MAR
	left_eq:
	sub X, 1
	left_ret:

	; if right -> X += 1
	cmp Z, 100
	jeq right_eq, MAR
	jmp right_ret, MAR
	right_eq:
	add X, 1
	right_ret:

	; if down -> Y -= 1
	cmp Z, 115
	jeq down_eq, MAR
	jmp down_ret, MAR
	down_eq:
	sub Y, 1
	down_ret:

	; if up -> Y += 1
	cmp Z, 119
	jeq up_eq, MAR
	jmp up_ret, MAR
	up_eq:
	sub Y, 1
	up_ret:

	; increase write pointer, and write to it
	lw A, pQUEUE_WRITE, MAR
	add A, 1
	sw A, pQUEUE_WRITE, MAR

	mv MarHi, high_byte(pQUEUE_BASE)
	mv MarLo, A

	; shift left again
	shl Y
	shl Y
	shl Y
	shl Y

	; make new byte
	or X, Y

	; store to queue
	sw X, MAR

	; pop previous
	lw A, pQUEUE_READ, MAR
	add A, 1
	sw A, pQUEUE_READ, MAR

	pop MAR
	jmp MAR
}

update_screen {
	lw MarLo, pQUEUE_READ, MAR
	mv MarHi, high_byte(pQUEUE_BASE)

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

fn high_byte(x: u16) {
	let high_byte: u8 = (x >> 8) & 0xff
	return high_byte
}
