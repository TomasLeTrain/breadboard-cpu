; places all instructions at 0x4000, also easy to remember
#address 0x4000 ; ensures start is first symbol executed
jmp start, MAR

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

; dir encoded with:
; 0 -> 0
; 1 -> 1
; 2 -> -1
; encoded as [0][x/2bits][y/2bits][000]
global pDIR_VECTOR = 0x9004

; dir vector that can directly address the char table
global pDIR_CHAR_IDX_LAST = 0x9005

; dir vector that can directly address the char table
global pDIR_CHAR_IDX_CURR = 0x9006

; gets set to 0 on draw_update, reset back to 1 input_update
let pCHAR_DRAW = 0x9007

; pHALT but for real
let pHALT_ACTUAL = 0x9008

; upper byte of queue
let pQUEUE_BASE = 0xA000

; read pointer - lower byte of whole addr
let pQUEUE_READ = 0x9100
; write pointer - lower byte of whole addr
let pQUEUE_WRITE = 0x9102
; apple pointer - where position of apple gets stored
let pAPPLE = 0x9104
; apple eaten pointer - is non-zero if apple was eaten, reset by display update
let pAPPLE_WAS_EATEN = 0x9105

let background_color = 0b11000100
let main_color = 0b11000000

fn get_BASE_GAME_POSITION(){
	let base_game_position_x = 12
	let base_game_position_y = 56
	let base_game_position = (base_game_position_x) | (base_game_position_y << 6)
	return base_game_position
}

init_state {
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
	sw A, pHALT_ACTUAL, MAR

	mv A, 1
	sw A, pDIR_X, MAR
	mv A, 0
	sw A, pDIR_Y, MAR

	mv A, (8) | (8 << 4)
	sw A, pAPPLE, MAR
	mv A, 0
	sw A, pAPPLE_WAS_EATEN, MAR

	mv A, (0b1010) << 3
	sw A, pDIR_CHAR_IDX_LAST, MAR
	mv A, (0b1010) << 3
	sw A, pDIR_CHAR_IDX_CURR, MAR
	mv A, 1
	sw A, pDIR_VECTOR, MAR

	mv A, 0
	sw A, pCHAR_DRAW, MAR

	; clear screen
	jsr(clear_screen)

	pop MAR
	jmp MAR
}

start {
	; initialize stack
	; TODO: should get moved to prelude since now can get called from other program
	; lda SP, STACK_START

	jsr(init_state)

	; main loop
	loop { 
		; let speed_dial = 1
		let speed_dial = 150

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

		; jmp loop, MAR
		lw A, pHALT_ACTUAL, MAR
		cmp A, 0
		jnz loop, MAR
	}

	; instead of halting just loop forever forever:
	; jmp forever, MAR

	; halt
	; return in case was called from other program
	; TODO: add condition to check if SP has any elements or not, to know if should halt or to continue (halting problem reference??)
	ret(MAR)
}

clear_screen {
	; sp ptr gets used as general pointer here, must save beforehand
	save_stack_ptr()

	fn clear_range_vram_16x(clear_val: u8, start_addr: Address, num_lines: u8){
		return {
			fn clear_2_pixels(){
				return {
					sw_vram Z, SP
					inc SP
					sw_vram Z, SP
					inc SP
				}
			}

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
	; clear_range_vram(background_color, 0, 240)
	clear_screen_colors(background_color)

	; clear all characters
	clear_screen_characters(0b00000000)

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

	fn save_dir_vector(data: u8){
		return {
			mv A, (data << 3)
			; mv A, y_dir << 5
			; mv A, (x_dir << 3)
			sw A, pDIR_VECTOR, MAR
		}
	}

	; save prev vector
	lw Y, pDIR_VECTOR, MAR

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
	save_dir_vector(0) ; (-1, 0)
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
	save_dir_vector(1) ; (1, 0)
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
	save_dir_vector(2) ; (0, -1)
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
	save_dir_vector(3) ; (0, -1)
	up_ret:

	; if r key -> init state
	cmp X, R_KEY
	jeq r_eq, MAR
	jmp r_ret, MAR
	r_eq:
	; exit game
	mv A, 0
	sw A, pHALT_ACTUAL, MAR
	; 
	; jsr(init_state)
	r_ret:

	; if direction changes in the same axis the game halts, so that case can be ignored
	; so, or both dir vectors to get the character to draw
	lw X, pDIR_VECTOR, MAR
	shl X
	shl X
	lw A, pDIR_VECTOR, MAR
	or A, X
	; no matter what, save to curr
	sw A, pDIR_CHAR_IDX_CURR, MAR

	lw X, pDIR_VECTOR, MAR
	cmp Y, X
	jeq input_change_eq, MAR
	doing_update:
	; update if input changed
	shl Y
	shl Y
	or X, Y
	sw X, pDIR_CHAR_IDX_LAST, MAR
	jmp update_end, MAR
	input_change_eq:

	; also update if char_draw changed
	lw A, pCHAR_DRAW, MAR
	cmp A, 0
	jeq doing_update, MAR
	update_end:

	mv A, 1
	sw A, pCHAR_DRAW, MAR

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

	fn draw_snake_pixel(color: u8, character_addr: Address, char_low_addr_ptr: Address, low_addr_mask: u8){
		return {
			setup_vram_addr()
			; save SP to Y and Z, to save it only for the body below
			mv Y, SpLo
			mv Z, SpHi

			; here, sp used to index character_addr
			; NOTE: must use SP as scratch since mar has important value
			lw SpLo, char_low_addr_ptr, SP
			mv SpHi, high_byte(character_addr)

			; in case its head
			or SpLo, low_addr_mask

			lw X, SP
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

	fn draw_fixed_pixel(color: u8, character: u8){
		return {
			setup_vram_addr()
			mv X, character
			sw_vram X, MAR
			line_write(MarLo,MarHi,MAR) ; 2
			line_write(MarLo,MarHi,MAR) ; 3
			line_write(MarLo,MarHi,MAR) ; 4
			line_write(MarLo,MarHi,MAR) ; 5
			line_write(MarLo,MarHi,MAR) ; 6
			line_write(MarLo,MarHi,MAR) ; 7
			line_write(MarLo,MarHi,MAR) ; 8

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

	; draw head, filled block
	draw_snake_pixel(main_color, pCHAR_DATA, pDIR_CHAR_IDX_CURR,0b10000000)

	; re-draw last, filled block
	lw SpLo, pQUEUE_WRITE, SP
	mv SpHi, high_byte(pQUEUE_BASE)
	sub SpLo, 1
	lw X, SP
	draw_snake_pixel(main_color, pCHAR_DATA, pDIR_CHAR_IDX_LAST,0b00000000)

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
	draw_fixed_pixel(main_color, 0)

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

	; reset char draw
	mv A, 0
	sw A, pCHAR_DRAW, MAR

	; get stack pointer back
	load_stack_ptr()

	pop MAR
	jmp MAR
}

; spans 0x7000 - 0x7800
global pCHAR_DATA = 0x7000
#address pCHAR_DATA
#data_file "../../vga_display_v1/snake_charsv2.out"
