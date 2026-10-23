#address 0x0 ; ensures start is first symbol executed
jmp start, MAR

; NOTE: includes some variable defs also useful here, better to include at the top
#include "src/utils.asm"

; terminal program

; Goals:
; prompt input from keyboard
; interpret commands and execute them (what format?)

; TODO: have input only add to input circular buffer, have update state process the new input into (x,y) positions on screen and interpreting commands

; input buffer start
global INPUT_BUFFER_START = 0x9000

; input buffer end ptr
global INPUT_BUFFER_END_PTR = 0x9100

global PROMPT_LINE_PTR = 0x9200
global CURR_CHAR_DRAW_PTR = 0x9201

; fn gen_BASE_CHAR_POSITION() {
; 	let base_char_position_x = 5
; 	let base_char_position_y = 5
; 	let base_char_position = (base_char_position_x) | (base_char_position_y << 6)
; 	return base_char_position
; }
; global BASE_CHAR_POSITION = gen_BASE_CHAR_POSITION()
global BASE_CHAR_POSITION = (5) | (5 << 6)

global background_color = 0b11000100
global main_color = 0b11000000

start {
	; initialize stack
	lda SP, STACK_START

	jsr(init_state)

	loop {
		jsr(update_input)
		jsr(update_state)
		jsr(update_screen)

		jmp loop, MAR
	}

	halt
}

init_state {
	; init any memory here
	save_stack_ptr()

	mv A, 0
	sw A, INPUT_BUFFER_END_PTR, MAR
	mv A, 0
	sw A, INPUT_BUFFER_START, MAR

	mv A, 0
	sw A, PROMPT_LINE_PTR, MAR

	mv A, 0
	sw A, CURR_CHAR_DRAW_PTR, MAR


	clear_screen_colors(background_color)
	clear_screen_characters(0)

	load_stack_ptr()

	pop MAR
	jmp MAR
}

update_input {
	; load buffer end pointer into MAR
	lw MarLo, INPUT_BUFFER_END_PTR, MAR
	mv MarHi, high_byte(INPUT_BUFFER_START)

	; get input and last input
	mv X, Keyb
	lw Y, MAR

	; check if new input and relevant input
	cmp X, Y

	; equal, no new input
	jeq eq_end, MAR
	
	; else, check if not actual input but blank input
	cmp X, 0
	jeq eq_end, MAR

	; new character, increase buffer
	lw MarLo, INPUT_BUFFER_END_PTR, MAR
	mv MarHi, high_byte(INPUT_BUFFER_START)

	add MarLo, 1
	; Store the character
	sw X, MAR

	mv A, MarLo
	sw A, INPUT_BUFFER_END_PTR, MAR

	cmp X, 13
	jeq enter_case, MAR
	jmp eq_end, MAR

	enter_case:
	; ; restore the buffer pointer
	; mv A, 0
	; sw A, INPUT_BUFFER_END_PTR, MAR

	; increase line by one
	lw A, PROMPT_LINE_PTR, MAR
	add A, 1
	sw A, PROMPT_LINE_PTR, MAR

	eq_end:

	pop MAR
	jmp MAR
}


update_state {

	pop MAR
	jmp MAR
}

update_screen {
	; save stack ptr since it gets used here
	save_stack_ptr()

	; sets up address for pixel at character position (X, Y)
	fn setup_vram_addr(){
		return {
			; must shift in total 9 times to the left (3 for character line, 6 for position in addr)
			; equal to setting high byte = y and shifting left once 
			shl Y

			; must write to 8 blocks most likely, loop?
			; MAR = base_addr + 1
			lda MAR, BASE_CHAR_POSITION + 1 + (1 << 14)

			; MAR += x
			add MarLo, X
			; MAR += y<<8, really should be OR but the carry from last add must be propagated, can compress into one instruction 
			adc MarHi, Y

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

	fn draw_character(color: u8, character_addr_offset: Address, char_low_addr_ptr: Address){
		return {
			setup_vram_addr()
			; save SP to Y and Z, to save it only for the body below
			mv Y, SpLo
			mv Z, SpHi

			; here, sp used to index character_addr
			; NOTE: must use SP as scratch since mar has important value
			lw SpLo, char_low_addr_ptr, SP
			mv SpHi, high_byte(character_addr_offset)

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

	lw MarLo, INPUT_BUFFER_END_PTR, MAR
	mv MarHi, high_byte(INPUT_BUFFER_START)
	lw Y, MAR

	shl Y
	shl Y
	shl Y

	mv X, MarLo
	sw Y, CURR_CHAR_DRAW_PTR, MAR

	lw Y, PROMPT_LINE_PTR, MAR

	; draw head, filled block
	draw_character(main_color, pCHAR_DATA, CURR_CHAR_DRAW_PTR)

	; get stack pointer back
	load_stack_ptr()

	pop MAR
	jmp MAR
}

; char data spans 0x3000 - 0x3800
global pCHAR_DATA = 0x3000
#address pCHAR_DATA
#data_file "../../vga_display_v1/charset.out"
