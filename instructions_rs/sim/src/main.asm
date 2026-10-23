#address 0x0 ; ensures start is first symbol executed
jmp start, MAR

; NOTE: includes some variable defs also useful here, better to include at the top
#include "src/utils.asm"

; terminal program

; Goals:
; prompt input from keyboard
; interpret commands and execute them (what format?)

; TODO: have backspace change input buffer write ptr
; TODO: keep track of command ptr


global CMD_BUFFER_PAGE = 0x8800
; input buffer start
global INPUT_BUFFER_PAGE = 0x9000

; input buffer end ptr
; non-inclusive
global INPUT_BUFFER_WRITE_PTR = 0x9100

; pointer in buffer from last processed inputs from update_state
; inclusive
global INPUT_BUFFER_READ_PTR = INPUT_BUFFER_WRITE_PTR + 1

; pointer in the cmd buffer page - non-inclusive, one after where command ends
global CMD_BUFFER_COMMAND_PTR = INPUT_BUFFER_READ_PTR + 1

; variables defining the state
global PROMPT_X_PTR = 0x9200
global PROMPT_Y_PTR = PROMPT_X_PTR+1

global CURR_CHAR_DRAW_PTR = PROMPT_Y_PTR + 1

; loads MAR with the address in memory corresponding to where the ptr points in the page
fn load_mar_page_ptr(page_start: Address, ptr: Address){
	return {
		; load buffer end pointer into MAR
		lw MarLo, ptr, MAR
		mv MarHi, high_byte(page_start)
	}
}

; loads SP with the address in memory corresponding to where the ptr points in the page
fn load_sp_page_ptr(page_start: Address, ptr: Address){
	return {
		; load buffer end pointer into MAR
		lw SpLo, ptr, SP
		mv SpHi, high_byte(page_start)
	}
}

; TODO: be able to use functions in global vars, would need defining function prototypes/return types?
; fn gen_BASE_CHAR_POSITION() {
; 	let base_char_position_x = 5
; 	let base_char_position_y = 5
; 	let base_char_position = (base_char_position_x) | (base_char_position_y << 6)
; 	return base_char_position
; }
; global BASE_CHAR_POSITION = gen_BASE_CHAR_POSITION()
global BASE_CHAR_POSITION = (5) | (5 << 6)

global background_color = 0b00000000
global main_color = 0b11000000

start {
	; initialize stack
	lda SP, STACK_START

	jsr(init_state)

	loop {
		; input adds to the input buffer, input looks at new inputs and also keeps track of where new commands start, also issues video commands to avoid adding more complexity
		jsr(update_input)
		jsr(update_input)
		jsr(update_input)

		jsr(update_state)

		jmp loop, MAR
	}

	halt
}

init_state {
	; init any memory here
	save_stack_ptr()

	mv A, 0
	sw A, INPUT_BUFFER_READ_PTR, MAR
	mv A, 1
	sw A, INPUT_BUFFER_WRITE_PTR, MAR
	mv A, 13
	sw A, INPUT_BUFFER_PAGE, MAR

	mv A, 0
	sw A, CMD_BUFFER_COMMAND_PTR, MAR
	mv A, 0
	sw A, CMD_BUFFER_PAGE, MAR

	mv A, 0
	sw A, PROMPT_X_PTR, MAR
	sw A, PROMPT_Y_PTR, MAR

	mv A, 0
	sw A, CURR_CHAR_DRAW_PTR, MAR

	clear_screen_colors(background_color)
	clear_screen_characters(0)

	load_stack_ptr()

	ret(MAR)
}

update_input {
	; load buffer end pointer into MAR
	load_mar_page_ptr(INPUT_BUFFER_PAGE, INPUT_BUFFER_WRITE_PTR)

	; get input and last input
	mv X, Keyb
	sub MarLo, 1
	lw Y, MAR

	; check if new input and relevant input
	cmp X, Y

	; equal, no new input
	jeq eq_end, MAR

	; new character, increase buffer
	load_mar_page_ptr(INPUT_BUFFER_PAGE, INPUT_BUFFER_WRITE_PTR)
	sw X, MAR

	; increase ptr
	mv A, MarLo
	add A, 1
	sw A, INPUT_BUFFER_WRITE_PTR, MAR

	eq_end:

	ret(MAR)
}


update_state {
	; Z = write_ptr
	lw Z, INPUT_BUFFER_WRITE_PTR, MAR

	loop {
		; check if read_ptr == write_ptr, in which case we are done
		lw A, INPUT_BUFFER_READ_PTR, MAR
		cmp A, Z
		jeq loop_end, MAR

		; MAR = &buffer[read_ptr]
		load_mar_page_ptr(INPUT_BUFFER_PAGE, INPUT_BUFFER_READ_PTR)

		; otherwise, process current input
		; X = buffer[read_ptr]
		lw X, MAR

		; zero is no character, just skip
		cmp X, 0
		jeq loop_increase, MAR

		cmp X, 13
		jeq enter_case_true, MAR
		jmp enter_case_false, MAR

		enter_case_true {
			; must erase previous cursor before going to next line
			; erase cursor at position after current
			; temporarily increase x
			lw A, PROMPT_X_PTR, MAR
			add A, 1
			sw A, PROMPT_X_PTR, MAR

			mv A, 8
			sw A, CURR_CHAR_DRAW_PTR, MAR

			; Z gets used in subroutine
			push Z
			; draw new character to screen
			jsr(draw_read_ptr_char)
			pop Z

			; increase line, reset x position
			lw A, PROMPT_Y_PTR, MAR
			add A, 1
			sw A, PROMPT_Y_PTR, MAR
			mv A, 0
			sw A, PROMPT_X_PTR, MAR

			; now, run command from buffer

			; Z only register that matters after calling
			push Z
			jsr(run_command)
			pop Z

			; reset command ptr back to zero
			mv A, 0
			sw A, CMD_BUFFER_COMMAND_PTR, MAR
			
			jmp all_end, MAR
		}
		enter_case_false {
			; check if backspace
			cmp X, 8
			jeq bs_case_true, MAR
			jmp bs_case_false, MAR

			bs_case_true {
				; only enable erase if not at beginning of line
				lw A, CMD_BUFFER_COMMAND_PTR, MAR
				cmp A, 0
				; jump to end if at start of line
				jeq all_end, MAR

				; erase cursor at position after current
				; temporarily increase x
				lw A, PROMPT_X_PTR, MAR
				add A, 1
				sw A, PROMPT_X_PTR, MAR

				mv A, 8
				sw A, CURR_CHAR_DRAW_PTR, MAR

				; Z gets used in subroutine
				push Z
				; draw new character to screen
				jsr(draw_read_ptr_char)
				pop Z

				; decrease X twice (one from temporary, one from backspace)
				lw A, PROMPT_X_PTR, MAR
				sub A, 2
				sw A, PROMPT_X_PTR, MAR

				; move back buffer ptr one as well
				lw A, CMD_BUFFER_COMMAND_PTR, MAR
				sub A, 1
				sw A, CMD_BUFFER_COMMAND_PTR, MAR

				jmp cursor_draw, MAR
			}
			bs_case_false {
				; increase X only
				lw A, PROMPT_X_PTR, MAR
				add A, 1
				sw A, PROMPT_X_PTR, MAR

				; also increase command buffer and copy character to it
				load_mar_page_ptr(CMD_BUFFER_PAGE, CMD_BUFFER_COMMAND_PTR)
				; copy character to cmd buffer
				; NOTE: assumes X has not been overriden in a subrountine call
				sw X, MAR

				; increase command buffer ptr
				lw A, CMD_BUFFER_COMMAND_PTR, MAR
				add A, 1
				sw A, CMD_BUFFER_COMMAND_PTR, MAR
			}
		}

		all_end:
		
		; setup char draw ptr
		load_mar_page_ptr(INPUT_BUFFER_PAGE, INPUT_BUFFER_READ_PTR)
		; y now holds the character at write_ptr
		lw A, MAR
		; store Y to the current char draw ptr used while drawing character
		sw A, CURR_CHAR_DRAW_PTR, MAR

		; Z gets used in subroutine
		push Z
		; draw new character to screen

		jsr(draw_read_ptr_char)
		pop Z
		
		; also, draw cursor at current position
		
		cursor_draw:
		; temporarily increase x
		lw A, PROMPT_X_PTR, MAR
		add A, 1
		sw A, PROMPT_X_PTR, MAR

		mv A, '_'
		sw A, CURR_CHAR_DRAW_PTR, MAR

		; Z gets used in subroutine
		push Z
		; draw new character to screen

		jsr(draw_read_ptr_char)
		pop Z

		; decrease X again
		lw A, PROMPT_X_PTR, MAR
		sub A, 1
		sw A, PROMPT_X_PTR, MAR

		loop_increase:
	
		; read_ptr = read_ptr + 1
		lw A, INPUT_BUFFER_READ_PTR, MAR
		add A, 1
		sw A, INPUT_BUFFER_READ_PTR, MAR
		
		jmp loop, MAR
		loop_end:
	}

	ret(MAR)
}

draw_read_ptr_char {
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
			mv SpHi, 0

			; shift left 3 times
			add SpLo, SpLo
			adc SpHi, SpHi

			add SpLo, SpLo
			adc SpHi, SpHi

			add SpLo, SpLo
			adc SpHi, SpHi

			; add char offset
			add SpHi, high_byte(character_addr_offset)

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

	; load coordinate of character
	lw X, PROMPT_X_PTR, MAR
	lw Y, PROMPT_Y_PTR, MAR

	; draw head, filled block
	draw_character(main_color, pCHAR_DATA, CURR_CHAR_DRAW_PTR)

	; get stack pointer back
	load_stack_ptr()

	ret(MAR)
}

; runs command in buffer[command_ptr..write_ptr]
run_command {
	lw Y, CMD_BUFFER_COMMAND_PTR, MAR
	cmp Y, 0
	; jump if no command
	jeq end, MAR
	; must interpret command by going through all characters!

	end:
	ret(MAR)
}

; char data spans 0x3000 - 0x3800
global pCHAR_DATA = 0x3000
#address pCHAR_DATA
#data_file "../../vga_display_v1/ascii_chars.out"
