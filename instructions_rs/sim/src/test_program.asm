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

fn get_BASE_GAME_POSITION(){
	let base_game_position_x = 12
	let base_game_position_y = 56
	let base_game_position = (base_game_position_x) | (base_game_position_y << 6)
	return base_game_position
}


start {
	; initialize stack
	lda SP, get_STACK_START()

	; start the queue state
	; start with two in queue
	mv A, 0
	sw A, pQUEUE_READ, MAR
	; mv A, 0
	; sw A, pQUEUE_WRITE, MAR
	; add second position at (2, 3)
	; mv A, (2) | (3 << 4)
	; sw A, pQUEUE_BASE, MAR

	mv A, 1
	sw A, pQUEUE_WRITE, MAR

	; add head position at (3, 3)
	mv A, (3) | (3 << 4)
	sw A, pQUEUE_BASE+1, MAR

	; add second position at (2, 3)
	mv A, (2) | (3 << 4)
	sw A, pQUEUE_BASE, MAR

	mv A, 1
	sw A, pHALT, MAR

	mv A, 0
	sw A, pDIR_X, MAR
	sw A, pDIR_Y, MAR

	jsr(clear_screen)

	; main loop
	loop { 
		let speed_dial = 64
		; let speed_dial = 255

		; delay since its too fast
		; NOTE: Z not used in update_input loop, must keep that way!
		mv Z, speed_dial
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

	fn clear_2_pixels(){
		return {
			sw_vram Z, SP
			inc SP
			sw_vram Z, SP
			inc SP
		}
	}

	fn clear_10_pixels(){
		return {
			clear_2_pixels()
			clear_2_pixels()
			clear_2_pixels()
			clear_2_pixels()
			clear_2_pixels()
		}
	}

	; clears 240 * 40 pixels with given value, at given addr
	fn clear_range_vram(clear_val: u8, start_addr: Address, num_lines: u8){
		return {
			; value that gets loaded as character everywhere
			mv Z, clear_val

			mv Y, num_lines

			lda SP, start_addr+1

			; addr = (x + 1) | (y << 6)

			y_loop {
				; clear line
				; TODO: make the number of cleared pixels adjustable
				clear_10_pixels()
				clear_10_pixels()
				clear_10_pixels()
				clear_10_pixels()

				; must add offset to SP to get to next line
				add SpLo, ((1<<6) + 1) - (40+1)
				adc SpHi, 0

				sub Y, 1
				lda MAR, y_loop
				jnz MAR
			}
		}
	}

	fn clear_range_vram_16x(clear_val: u8, start_addr: Address, num_lines: u8){
		return {
			; value that gets loaded as character everywhere
			mv Z, clear_val

			mv Y, num_lines

			lda SP, start_addr+1

			y_loop {
				; clear line
				; TODO: make the number of cleared pixels adjustable
				clear_2_pixels()
				clear_2_pixels()
				clear_2_pixels()
				clear_2_pixels()
				clear_2_pixels()
				clear_2_pixels()
				clear_2_pixels()
				clear_2_pixels()

				; must add offset to SP to get to next line
				add SpLo, ((1<<6) + 1) - (16+1)
				adc SpHi, 0

				sub Y, 1
				lda MAR, y_loop
				jnz MAR
			}
		}
	}

	; clear all colors
	clear_range_vram(0b11000100, 0, 240)
	; clear all characters
	clear_range_vram(0b00000000, 1 << 14, 240)

	; make valid snake positions a different color
	clear_range_vram_16x(0b11000000, get_BASE_GAME_POSITION(), 8 * 16 - 1)

	; get stack pointer back
	load_stack_ptr()

	; return
	pop MAR
	jmp MAR
}

update_input {
	mv X, Keyb

	let W_KEY = 119
	let A_KEY =  97
	let S_KEY = 115
	let D_KEY = 100

	; if left -> X -= 1
	cmp X, A_KEY
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
	cmp X, D_KEY
	jeq right_eq, MAR
	jmp right_ret, MAR
	right_eq:
	mv A, 1
	sw A, pDIR_X, MAR
	mv A, 0
	sw A, pDIR_Y, MAR
	right_ret:

	; if down -> Y += 1
	cmp X, S_KEY
	jeq down_eq, MAR
	jmp down_ret, MAR
	down_eq:
	mv A, 1
	sw A, pDIR_Y, MAR
	mv A, 0
	sw A, pDIR_X, MAR
	down_ret:

	; if up -> Y -= 1
	cmp X, W_KEY
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
	; mods the position
	and X, 0b1111

	lw Z, pDIR_Y, MAR
	add Y, Z
	; mods the position
	and Y, 0b1111

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

			; TODO: in the future could implement 16-bit shifting?
			; shifted 5 times, in theory is 6 but y is implicitly shifted by 4 already (want y to be by three originally to move in multiples of 8)
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

			; must write to 8 blocks most likely, loop?
			; MAR = base_addr + 1
			lda MAR, get_BASE_GAME_POSITION() + 1 + (1 << 14)
			
			; MAR += Sp
			add MarLo, SpLo
			adc MarHi, SpHi

			; MAR += x
			add MarLo, X
			adc MarHi, 0
		}
	}

	; draw new position
	lw MarLo, pQUEUE_WRITE, MAR
	mv MarHi, high_byte(pQUEUE_BASE)

	; read to x
	lw X, MAR

	position_to_mar_addr()

	fn line_write(){
		return {
			; known offset to get to next line
			add MarLo, 1<<6
			adc MarHi, 0
			sw_vram X, MAR
		}
	}

	mv X, 0xff
	sw_vram X, MAR
	line_write() ; 2
	line_write() ; 3
	line_write() ; 4
	line_write() ; 5
	line_write() ; 6
	line_write() ; 7
	line_write() ; 8

	; now erase last one
	lw MarLo, pQUEUE_READ, MAR
	mv MarHi, high_byte(pQUEUE_BASE)

	; read to x
	lw X, MAR

	position_to_mar_addr()
	mv X, 0b00000000
	sw_vram X, MAR
	line_write() ; 2
	line_write() ; 3
	line_write() ; 4
	line_write() ; 5
	line_write() ; 6
	line_write() ; 7
	line_write() ; 8

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
