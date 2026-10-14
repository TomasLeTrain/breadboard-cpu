; snake program

; Goals:
; - have circular queue of positions, can represent 16x16 in one byte
; - draw on screen
; - player input

; where the current x direction gets stored
let pDIR_X = 0x9001
; where the current y direction gets stored
let pDIR_Y = 0x9002

; game runs until == 0
let pHALT = 0x8080

; upper byte of queue
let pQUEUE_BASE = 0xA000

; read pointer - lower byte of whole addr
let pQUEUE_READ = 0x9020
; write pointer - lower byte of whole addr
let pQUEUE_WRITE = 0x9040

; TODO: add global symbols so they can be used in functions directly
fn get_STACK_START(){
	let STACK_START = 0xffe0
	return STACK_START
}
fn get_STACK_SAVE_ADDR(){
	let STACK_SAVE_ADDR = 0xffe2
	return  STACK_SAVE_ADDR
}



start {
	; initialize stack
	lda SP, get_STACK_START()

	; start the queue state
	mv A, 0
	sw A, pQUEUE_READ, MAR
	mv A, 0
	sw A, pQUEUE_WRITE, MAR

	; add initial position at (2, 2)
	; mv A, 0b0010 | (0b0010 << 4)
	mv A, 0b1111 | (0b0010 << 4)
	; 00100010
	sw A, pQUEUE_BASE, MAR

	mv A, 1
	sw A, pHALT, MAR

	mv A, 0
	sw A, pDIR_X, MAR
	sw A, pDIR_Y, MAR

	jsr(clear_screen)

	; main loop
	loop { 
		; delay since its too fast
		; NOTE: Z not used in update_input loop, must keep that way!
		mv Z, 128
		nop_loop {
			; keep updating input while waiting
			jsr(update_input)

			sub Z, 1
			lda MAR, nop_loop
			jnz Z, MAR
		}
	
		jsr(update_state)
		jsr(update_screen)


		lw A, pHALT, MAR
		cmp A, 0
		lda MAR, loop
		jnz MAR
	}

	halt
}

; saves to tmp save addr
fn save_stack_ptr(){
	return {
		sw SpLo, get_STACK_SAVE_ADDR(), MAR
		sw SpHi, get_STACK_SAVE_ADDR()+1, MAR
	}
}

; loads from tmp save addr
fn load_stack_ptr(){
	return {
		lw SpLo, get_STACK_SAVE_ADDR(), MAR
		lw SpHi, get_STACK_SAVE_ADDR()+1, MAR
	}
}

clear_screen {
	; sp ptr gets used as general pointer here, must save beforehand
	save_stack_ptr()

	fn clear_10_pixels(){
		return {
			sw_vram Z, SP
			inc SP
			sw_vram Z, SP
			inc SP
			sw_vram Z, SP
			inc SP
			sw_vram Z, SP
			inc SP
			sw_vram Z, SP
			inc SP
			sw_vram Z, SP
			inc SP
			sw_vram Z, SP
			inc SP
			sw_vram Z, SP
			inc SP
			sw_vram Z, SP
			inc SP
			sw_vram Z, SP
			inc SP
		}
	}

	; clears 240 * 40 pixels with given value, at given addr
	fn clear_range_vram(clear_val: u8, start_addr: Address){
		return {
			; value that gets loaded as character everywhere
			mv Z, clear_val

			mv Y, 240

			lda SP, start_addr+1

			; addr = (x + 1) | (y << 6)

			y_loop {
				; clear line
				clear_10_pixels()
				clear_10_pixels()
				clear_10_pixels()
				clear_10_pixels()

				; must add offset to SP to get to next line
				add SpLo, ((1<<6) + 1) - 41
				adc SpHi, 0

				sub Y, 1
				lda MAR, y_loop
				jnz MAR
			}
	}
	}

	; clear all colors
	clear_range_vram(0b11110001, 0)
	; clear all characters
	; clear_range_vram(0b11111111, 1 << 14)
	clear_range_vram(0b00000000, 1 << 14)

	; get stack pointer back
	load_stack_ptr()

	; return
	pop MAR
	jmp MAR
}

update_input {
	mv X, Keyb

	; W = 119
	; A =  97
	; S = 115
	; D = 100

	; if left -> X -= 1
	cmp X, 97
	jeq left_eq, MAR
	jmp left_ret, MAR
	left_eq:
	; TODO: fix negation parsing
	mv A, 255
	sw A, pDIR_X, MAR
	mv A, 0
	sw A, pDIR_Y, MAR
	left_ret:

	; if right -> X += 1
	cmp X, 100
	jeq right_eq, MAR
	jmp right_ret, MAR
	right_eq:
	mv A, 1
	sw A, pDIR_X, MAR
	mv A, 0
	sw A, pDIR_Y, MAR
	right_ret:

	; if down -> Y += 1
	cmp X, 115
	jeq down_eq, MAR
	jmp down_ret, MAR
	down_eq:
	mv A, 1
	sw A, pDIR_Y, MAR
	mv A, 0
	sw A, pDIR_X, MAR
	down_ret:

	; if up -> Y -= 1
	cmp X, 119
	jeq up_eq, MAR
	jmp up_ret, MAR
	up_eq:
	mv A, 255
	sw A, pDIR_Y, MAR
	mv A, 0
	sw A, pDIR_X, MAR
	up_ret:

	pop MAR
	jmp MAR
}

update_state {
	; get head of snake (end of queue)
	lw MarLo, pQUEUE_WRITE, MAR
	mv MarHi, high_byte(pQUEUE_BASE)

	lw X, MAR
	mv Y, X

	; mask first 4 bits
	and X, 0b00001111

	; mask last 4 bits
	and Y, 0b11110000
	shr Y
	shr Y
	shr Y
	shr Y

	; calculate new position
	lw Z, pDIR_X, MAR
	add X, Z
	lw Z, pDIR_Y, MAR
	add Y, Z

	; shift left again
	shl Y
	shl Y
	shl Y
	shl Y

	; make new byte
	or X, Y

	; increase write pointer, and write to it
	lw A, pQUEUE_WRITE, MAR
	add A, 1
	sw A, pQUEUE_WRITE, MAR

	; store to queue
	mv MarHi, high_byte(pQUEUE_BASE)
	mv MarLo, A
	sw X, MAR

	pop MAR
	jmp MAR
}

update_screen {
	; save stack ptr since it gets used here
	save_stack_ptr()

	; assumes X has position
	; leaves MAR with the desired position
	fn position_to_mar_addr(){
		return {
			; copy value to Y as well
			mv Y, X

			; mask first 4 bits
			and X, 0b00001111
			; mask last 4 bits
			and Y, 0b11110000

			; shift right 4 times, add z to it, then shift back up
			; only shifts y once to have it be y << 3 = y * 8, so the snake moves in uniform 8 by 8 squares
			shr Y
			; shr Y
			; shr Y
			; shr Y
			add Y, Z

			; TODO: in the future could implement 16-bit shifting?
			; y is already shifted by 4, but need shifted by 6
			; can do by:
			; y << 2 = (y + y) + (y + y)
			; final addr is
			; base_addr + (y << 2) + x
			lda SP, 0

			; Sp = y
			mv SpLo, Y
			mv SpHi, 0

			; Sp << 1
			add SpLo, SpLo
			adc SpHi, SpHi
			; Sp << 1
			add SpLo, SpLo
			adc SpHi, SpHi
			; Sp << 1
			add SpLo, SpLo
			adc SpHi, SpHi
			; Sp << 1
			add SpLo, SpLo
			adc SpHi, SpHi
			; Sp << 1
			add SpLo, SpLo
			adc SpHi, SpHi
			; Sp << 1
			add SpLo, SpLo
			adc SpHi, SpHi

			; must write to 8 blocks most likely, loop?
			; MAR = base_addr + 1
			lda MAR, (1 << 14) + 1
			
			; MAR += Sp
			add MarLo, SpLo
			adc MarHi, SpHi

			; MAR += x
			add MarLo, X
			adc MarHi, 0
		}
	}

	; loop index
	mv Z, 8

	loop {
		; draw new position
		lw MarLo, pQUEUE_WRITE, MAR
		mv MarHi, high_byte(pQUEUE_BASE)

		; read to x
		lw X, MAR

		position_to_mar_addr()
		; mv X, 0b01010101
		mv X, 0xff
		sw_vram X, MAR

		; now erase last one
		lw MarLo, pQUEUE_READ, MAR
		mv MarHi, high_byte(pQUEUE_BASE)

		; read to x
		lw X, MAR

		position_to_mar_addr()
		mv X, 0b00000000
		sw_vram X, MAR
	
		sub Z, 1
		lda MAR, loop
		jnz Z, MAR
	}

	; get stack pointer back
	load_stack_ptr()

	; pop previous after erasing it
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

fn high_byte(x: u16) {
	let high_byte: u8 = (x >> 8) & 0xff
	return high_byte
}
