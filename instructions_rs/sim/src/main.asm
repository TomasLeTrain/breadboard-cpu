#address 0x0 ; ensures start is first symbol executed
jmp start, MAR

#include "src/utils.asm"

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
let pHALT = 0x9003

; upper byte of queue
let pQUEUE_BASE = 0xA000

; read pointer - lower byte of whole addr
let pQUEUE_READ = 0x9100
; write pointer - lower byte of whole addr
let pQUEUE_WRITE = 0x9102
; apple pointer - where position of apple gets stored
let pAPPLE = 0x9104
; apple eaten pointer - is non-zero if apple was eaten, reset by display update
let pAPPLE_WAS_EATEN = 0x9106

; 8 bytes in size
let pHEAD_DOWN = 0x9200
let pHEAD_UP = pHEAD_DOWN + 8
let pHEAD_LEFT = pHEAD_UP + 8
let pHEAD_RIGHT = pHEAD_LEFT + 8

let pLONG_DOWN = pHEAD_RIGHT + 8
let pLONG_UP = pLONG_DOWN + 8
let pLONG_LEFT = pLONG_UP + 8
let pLONG_RIGHT = pLONG_LEFT + 8


; snake character address scheme (msb to lsb):
; [head/1bit][y_dir/2bits][x_dir/2bits][y position/3bits] -> 3*3*2 = 18 characters, 144 bytes
; TODO: not program 18 different combinations manually lol



let background_color = 0b11000100
let main_color = 0b11000000

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

create_pixel_data {
	fn save_value(addr: Address, offset: Address, value: u8){
		return {
			mv A, value
			sw A, addr + offset, MAR
		}
	}

	; head connections
	save_value(pHEAD_DOWN, 0, 0b00011000)
	save_value(pHEAD_DOWN, 1, 0b00011000)
	save_value(pHEAD_DOWN, 2, 0b00011000)
	save_value(pHEAD_DOWN, 3, 0b00011000)
	save_value(pHEAD_DOWN, 4, 0b00111100)
	save_value(pHEAD_DOWN, 5, 0b00111100)
	save_value(pHEAD_DOWN, 6, 0b00111100)
	save_value(pHEAD_DOWN, 7, 0b00011000)

	save_value(pHEAD_UP, 0, 0b00011000)
	save_value(pHEAD_UP, 1, 0b00111100)
	save_value(pHEAD_UP, 2, 0b00111100)
	save_value(pHEAD_UP, 3, 0b00111100)
	save_value(pHEAD_UP, 4, 0b00011000)
	save_value(pHEAD_UP, 5, 0b00011000)
	save_value(pHEAD_UP, 6, 0b00011000)
	save_value(pHEAD_UP, 7, 0b00011000)

	save_value(pHEAD_LEFT, 0, 0b00000000)
	save_value(pHEAD_LEFT, 1, 0b00000000)
	save_value(pHEAD_LEFT, 2, 0b01110000)
	save_value(pHEAD_LEFT, 3, 0b11111111)
	save_value(pHEAD_LEFT, 4, 0b11111111)
	save_value(pHEAD_LEFT, 5, 0b01110000)
	save_value(pHEAD_LEFT, 6, 0b00000000)
	save_value(pHEAD_LEFT, 7, 0b00000000)

	save_value(pHEAD_RIGHT, 0, 0b00000000)
	save_value(pHEAD_RIGHT, 1, 0b00000000)
	save_value(pHEAD_RIGHT, 2, 0b00001110)
	save_value(pHEAD_RIGHT, 3, 0b11111111)
	save_value(pHEAD_RIGHT, 4, 0b11111111)
	save_value(pHEAD_RIGHT, 5, 0b00001110)
	save_value(pHEAD_RIGHT, 6, 0b00000000)
	save_value(pHEAD_RIGHT, 7, 0b00000000)

	; long connections
	save_value(pLONG_DOWN, 0, 0b00011000)
	save_value(pLONG_DOWN, 1, 0b00011000)
	save_value(pLONG_DOWN, 2, 0b00011000)
	save_value(pLONG_DOWN, 3, 0b00011000)
	save_value(pLONG_DOWN, 4, 0b00011000)
	save_value(pLONG_DOWN, 5, 0b00011000)
	save_value(pLONG_DOWN, 6, 0b00011000)
	save_value(pLONG_DOWN, 7, 0b00011000)

	save_value(pLONG_UP, 0, 0b00011000)
	save_value(pLONG_UP, 1, 0b00011000)
	save_value(pLONG_UP, 2, 0b00011000)
	save_value(pLONG_UP, 3, 0b00011000)
	save_value(pLONG_UP, 4, 0b00011000)
	save_value(pLONG_UP, 5, 0b00011000)
	save_value(pLONG_UP, 6, 0b00011000)
	save_value(pLONG_UP, 7, 0b00011000)

	save_value(pLONG_LEFT, 0, 0b00000000)
	save_value(pLONG_LEFT, 1, 0b00000000)
	save_value(pLONG_LEFT, 2, 0b00000000)
	save_value(pLONG_LEFT, 3, 0b11111111)
	save_value(pLONG_LEFT, 4, 0b11111111)
	save_value(pLONG_LEFT, 5, 0b00000000)
	save_value(pLONG_LEFT, 6, 0b00000000)
	save_value(pLONG_LEFT, 7, 0b00000000)

	save_value(pLONG_RIGHT, 0, 0b00000000)
	save_value(pLONG_RIGHT, 1, 0b00000000)
	save_value(pLONG_RIGHT, 2, 0b00000000)
	save_value(pLONG_RIGHT, 3, 0b11111111)
	save_value(pLONG_RIGHT, 4, 0b11111111)
	save_value(pLONG_RIGHT, 5, 0b00000000)
	save_value(pLONG_RIGHT, 6, 0b00000000)
	save_value(pLONG_RIGHT, 7, 0b00000000)

	; return
	pop MAR
	jmp MAR
}


init_state {
	jsr(create_pixel_data)

	; start the queue state
	; start with two in queue
	mv A, 0
	sw A, pQUEUE_READ, MAR

	mv A, 1
	sw A, pQUEUE_WRITE, MAR

	; add second position at (2, 3)
	mv A, (2) | (3 << 4)
	sw A, pQUEUE_BASE, MAR

	; add head position at (3, 3)
	mv A, (3) | (3 << 4)
	sw A, pQUEUE_BASE+1, MAR


	mv A, 1
	sw A, pHALT, MAR

	mv A, 1
	sw A, pDIR_X, MAR
	mv A, 0
	sw A, pDIR_Y, MAR

	mv A, (8) | (8 << 4)
	sw A, pAPPLE, MAR
	mv A, 0
	sw A, pAPPLE_WAS_EATEN, MAR

	; clear screen
	jsr(clear_screen)

	pop MAR
	jmp MAR
}

start {
	; initialize stack
	lda SP, get_STACK_START()

	jsr(init_state)

	; main loop
	loop { 
		; let speed_dial = 1
		let speed_dial = 120

		; delay since its too fast
		; NOTE: Z not used in update_input loop, must keep that way!
		mv Z, speed_dial
		nop_loop {
			; keep updating input while waiting
			; some inlined to multiply impact of speed dial
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)
			jsr(update_input)

			sub Z, 1
			lda MAR, nop_loop
			jnz Z, MAR
		}
	
		lw A, pHALT, MAR
		cmp A, 0
		jeq skip_state_update, MAR
		; only update state if not halted
		jsr(update_state)
		jsr(update_screen)
		jmp skip_state_update_end, MAR
		skip_state_update:
		; if halted, restart state automatically
		jsr(init_state)
		skip_state_update_end:

		jmp loop, MAR
		; lw A, pHALT, MAR
		; cmp A, 0
		; lda MAR, loop
		; jnz MAR
	}

	; instead of halting just loop forever
	; forever:
	; jmp forever, MAR

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
	clear_range_vram(background_color, 0, 240)
	; clear all characters
	clear_range_vram(0b00000000, 1 << 14, 240)

	; make valid snake positions a different color
	clear_range_vram_16x(main_color, get_BASE_GAME_POSITION(), 8 * 16 - 1)

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
	let R_KEY = 114

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


	; if r key -> init state
	cmp X, R_KEY
	jeq r_eq, MAR
	jmp r_ret, MAR
	r_eq:
	jsr(init_state)
	r_ret:

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

	; now, check if this position is the same as apple 
	lw Y, pAPPLE, MAR
	cmp X, Y

	jeq apple_eaten, MAR
	jmp apple_ret, MAR
	apple_eaten:
	; store 1 that apple was eaten
	mv A, 1
	sw A, pAPPLE_WAS_EATEN, MAR

	; now, generate a new apple position
	; very rough pseudorandom generator
	lw X, pAPPLE, MAR
	mv Y, X
	xor X, 130
	shl X
	add X, 51
	; x acts as random position after current, otherwise x eventually ends up at stable position
	add Y, X
	sw Y, pAPPLE, MAR

	apple_ret:

	; now, check if current position intersects with any existing snake parts, and if so halt
	save_stack_ptr()

	; Z holds the end pointer position
	lw Z, pQUEUE_WRITE, MAR

	; X holds the current position
	mv MarLo, Z
	mv MarHi, high_byte(pQUEUE_BASE)
	lw X, MAR

	; start sp on read pointer
	lw SpLo, pQUEUE_READ, MAR
	mv SpHi, high_byte(pQUEUE_BASE)

	; TODO: read is getting deleted so should be fine?
	; inc SP
	
	collision_check_loop {
		cmp SpLo, Z
		jeq collision_check_loop_exit, MAR

		; Y holds the current position
		lw Y, SP

		; check if they are equal
		cmp Y, X
		; 
		jeq collision_eq, MAR
		jmp collision_end, MAR
		collision_eq:

		; collision, set halt!
		mv A, 0
		sw A, pHALT, MAR

		collision_end:

		; NOTE: don't increment sp! circular heap depends on wrap around from byte addition
		add SpLo, 1

		jmp collision_check_loop, MAR

		collision_check_loop_exit:
	}

	load_stack_ptr()

	; return
	pop MAR
	jmp MAR
}

update_screen {
	; save stack ptr since it gets used here
	save_stack_ptr()

	; draws pixel with given character pattern at position stored in X
	fn setup_vram_addr(){
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

			; can save this mar addr to sp, and reuse it for coloring
			mv SpLo, MarLo
			mv SpHi, MarHi
			; ; remove last bit for color instad of character
			and SpHi, 0b00111111
		}
	}

	fn line_write(Reg0: Register, Reg1: Register, addrReg: AddressRegister){
		return {
			; known offset to get to next line
			add Reg0, 1<<6
			adc Reg1, 0
			sw_vram X, addrReg
		}
	}

	fn draw_snake_pixel(color: u8, character_addr: Address){
		return {
			setup_vram_addr()
			; save SP to Y and Z, to save it only for the body below
			mv Y, SpLo
			mv Z, SpHi

			lw X, character_addr, SP
			sw_vram X, MAR
			inc SP
			lw X, SP
			line_write(MarLo,MarHi,MAR) ; 2
			inc SP
			lw X, SP
			line_write(MarLo,MarHi,MAR) ; 3
			inc SP
			lw X, SP
			line_write(MarLo,MarHi,MAR) ; 4
			inc SP
			lw X, SP
			line_write(MarLo,MarHi,MAR) ; 5
			inc SP
			lw X, SP
			line_write(MarLo,MarHi,MAR) ; 6
			inc SP
			lw X, SP
			line_write(MarLo,MarHi,MAR) ; 7
			inc SP
			lw X, SP
			line_write(MarLo,MarHi,MAR) ; 8

			; restore SP to Y and Z, to save it only for the body below
			mv SpLo, Y
			mv SpHi, Z

			mv X, color
			sw_vram X, SP
			line_write(SpLo,SpHi,SP) ; 2
			line_write(SpLo,SpHi,SP) ; 3
			line_write(SpLo,SpHi,SP) ; 4
			line_write(SpLo,SpHi,SP) ; 5
			line_write(SpLo,SpHi,SP) ; 6
			line_write(SpLo,SpHi,SP) ; 7
			line_write(SpLo,SpHi,SP) ; 8
		}
	}

	; draw new position
	lw MarLo, pQUEUE_WRITE, MAR
	mv MarHi, high_byte(pQUEUE_BASE)
	lw X, MAR
	; main color, filled block
	draw_snake_pixel(main_color, pHEAD_DOWN)

	; check if apple was eaten
	lw X, pAPPLE_WAS_EATEN, MAR
	cmp X, 0
	jnz apple_eaten, MAR
	apple_not_eaten:
	; now erase last one since no eat happened
	lw MarLo, pQUEUE_READ, MAR
	mv MarHi, high_byte(pQUEUE_BASE)
	lw X, MAR
	; main color, empty block
	draw_snake_pixel(main_color, pLONG_DOWN)

	; not eaten, pop previous after erasing it in display
	lw A, pQUEUE_READ, MAR
	add A, 1
	sw A, pQUEUE_READ, MAR

	apple_eaten:
	; eaten, don't pop queue
	apple_return:

	; no matter what, draw apple
	lw X, pAPPLE, MAR
	; draw design for apple
	setup_vram_addr()
	let apple0 = 0b000001000
	let apple1 = 0b000010000
	let apple2 = 0b000111000
	let apple3 = 0b001111100
	let apple4 = 0b001111100
	let apple5 = 0b001111100
	let apple6 = 0b000111000
	let apple7 = 0b000000000

	let apple_color_red = 0b10010000
	let apple_color_green = 0b01000000

	mv X, apple0
	sw_vram X, MAR
	mv X, apple1
	line_write(MarLo,MarHi,MAR) ; 2
	mv X, apple2
	line_write(MarLo,MarHi,MAR) ; 3
	mv X, apple3
	line_write(MarLo,MarHi,MAR) ; 4
	mv X, apple4
	line_write(MarLo,MarHi,MAR) ; 5
	mv X, apple5
	line_write(MarLo,MarHi,MAR) ; 6
	mv X, apple6
	line_write(MarLo,MarHi,MAR) ; 7
	mv X, apple7
	line_write(MarLo,MarHi,MAR) ; 8

	mv X, apple_color_green
	sw_vram X, SP
	line_write(SpLo,SpHi,SP) ; 2
	mv X, apple_color_red
	line_write(SpLo,SpHi,SP) ; 3
	line_write(SpLo,SpHi,SP) ; 4
	line_write(SpLo,SpHi,SP) ; 5
	line_write(SpLo,SpHi,SP) ; 6
	line_write(SpLo,SpHi,SP) ; 7
	line_write(SpLo,SpHi,SP) ; 8

	; reset apple being eaten
	mv A, 0
	sw A, pAPPLE_WAS_EATEN, MAR

	; get stack pointer back
	load_stack_ptr()

	pop MAR
	jmp MAR
}
