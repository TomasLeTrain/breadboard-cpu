; start emitting code a bit after start to leave room for jumptables

#address start_jumptable
jmp start, MAR

; start emmitting code at 0x0500 to leave some bytes for jumptable/prelude code
#address 0x0500

; terminal program

; Goals:
; prompt input from keyboard
; interpret commands and execute them (what format?)

; where screen state gets stored
; SCREEN_STATE[(y << 6) | x] = character
; should be from 0x8000 - 0x87A8
global SCREEN_STATE = 0x8000

global SHOULD_INIT = 0x8800

global CMD_BUFFER_PAGE = 0x9100
; input buffer start
global INPUT_BUFFER_PAGE = 0x9200

; all var variables defined on 0x9000
; input buffer end ptr
; non-inclusive
global INPUT_BUFFER_WRITE_PTR = 0x9000

; pointer in buffer from last processed inputs from update_state
; inclusive
global INPUT_BUFFER_READ_PTR = INPUT_BUFFER_WRITE_PTR + 1

; pointer in the cmd buffer page - non-inclusive, one after where command ends
global CMD_BUFFER_COMMAND_PTR = INPUT_BUFFER_READ_PTR + 1

; variables defining the state
; global PROMPT_X_PTR = CMD_BUFFER_COMMAND_PTR + 1
; global PROMPT_Y_PTR = PROMPT_X_PTR + 1

; put outside of state that gets erased to work between other program runs
global PROMPT_X_PTR = 0xc000 + 1
global PROMPT_Y_PTR = PROMPT_X_PTR + 1

global CURR_CHAR_DRAW_PTR = PROMPT_Y_PTR + 1

; used to save ptr when drawing at arbitrary spots
global PROMPT_X_PTR_SAVE = CURR_CHAR_DRAW_PTR + 1
global PROMPT_Y_PTR_SAVE = PROMPT_X_PTR_SAVE + 1

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

; only gets run on first boot, sets up stack
; TODO: stack init should get moved to prelude code
start {
	; initialize stack
	lda SP, STACK_START

	mv A, 0
	sw A, SHOULD_INIT, MAR

	jmp main, MAR
}

main {
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

	; check if its first time or just restarting state
	lw A, SHOULD_INIT, MAR
	cmp A, 0
	jnz restart_init, MAR 

	first_init:
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

	let default_character = 0

	clear_screen_characters(default_character)

	; clear screen state
	lda SP, SCREEN_STATE
	mv X, default_character

	mv Z, 30

	loop {
		fn update_2_characters(){
			return {
				sw X, SP
				inc SP
				sw X, SP
				inc SP
			}
		}

		fn update_10_characters(){
			return {
				update_2_characters()
				update_2_characters()
				update_2_characters()
				update_2_characters()
				update_2_characters()
			}
		}

		update_10_characters()
		update_10_characters()
		update_10_characters()
		update_10_characters()

		; put Sp on next char
		add SpLo, (1 << 6) - 40
		adc SpHi, 0

		sub Z, 1
		lda MAR, loop
		jnz Z, MAR
	}

	load_stack_ptr()

	jmp init_end, MAR

	restart_init:
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

	; prompt X and Y should already be saved from last run

	; first reset all colors
	clear_screen_colors(background_color)

	; must draw all characters saved on screen state
	; save ptrs before overriding them
	lw A, PROMPT_X_PTR, MAR
	sw A, PROMPT_X_PTR_SAVE, MAR
	lw A, PROMPT_Y_PTR, MAR
	sw A, PROMPT_Y_PTR_SAVE, MAR

	; now, draw all chars
	mv X, 0
	mv Y, 0
	mv Z, 30

	; aout value Some(e109) from MAR
	; aout value Some(e109) from MAR
	; low to high clk
	; load 9 into low of PC
	; reset STEP
	; high to low clk
	; opcode_addr: 153a
	; load f1 into OPCODE_LATCH1
	; load 1 into OPCODE_LATCH2
	; aout value Some(e109) from PC
	; ram read None from 6109
	; low to high clk

	; store initial screen_state ptr
	mv A, low_byte(SCREEN_STATE)
	sw A, STACK_SAVE_ADDR_2, MAR
	mv A, high_byte(SCREEN_STATE)
	sw A, STACK_SAVE_ADDR_2+1, MAR

	; load to call routine
	load_stack_ptr()

	loop2 {
		; passthrough for function
		global draw_read_ptr_char_ = draw_read_ptr_char
		fn update_1_character(){
			return {
				; update X ptr
				sw X, PROMPT_X_PTR, MAR
				; get character and load into draw_ptr

				; get screen state position
				lw B, STACK_SAVE_ADDR_2, MAR
				lw MarHi, STACK_SAVE_ADDR_2+1, MAR
				mv MarLo, B
				or MarLo, X

				; get character at position in state and move to 
				lw A, MAR
				sw A, CURR_CHAR_DRAW_PTR, MAR

				; call subroutine
				push X
				push Y
				push Z

				jsr(draw_read_ptr_char_)

				pop Z
				pop Y
				pop X

				; increase X position
				add X, 1
			}
		}

		fn update_10_characters(){
			return {
				update_1_character()
				update_1_character()
				update_1_character()
				update_1_character()
				update_1_character()
				update_1_character()
				update_1_character()
				update_1_character()
				update_1_character()
				update_1_character()
			}
		}

		update_10_characters()
		update_10_characters()
		update_10_characters()
		update_10_characters()

		; put screen state ptr on next line
		lw B, STACK_SAVE_ADDR_2, MAR
		lw MarHi, STACK_SAVE_ADDR_2+1, MAR
		mv MarLo, B

		; add MarLo, (1 << 6) - 40
		add MarLo, 1 << 6
		adc MarHi, 0

		mv A, MarLo
		mv B, MarHi
		sw A, STACK_SAVE_ADDR_2, MAR
		sw B, STACK_SAVE_ADDR_2+1, MAR

		; reset X, update Y
		mv X, 0
		add Y, 1
		sw Y, PROMPT_Y_PTR, MAR

		; loop
		sub Z, 1
		lda MAR, loop2
		jnz Z, MAR
	}


	; restore back the ptrs
	lw A, PROMPT_X_PTR_SAVE, MAR
	sw A, PROMPT_X_PTR, MAR
	lw A, PROMPT_Y_PTR_SAVE, MAR
	sw A, PROMPT_Y_PTR, MAR
	
	init_end:

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
				; skip everything since no action happens
				jeq loop_increase, MAR

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

	lw X, PROMPT_X_PTR, MAR
	lw Y, PROMPT_Y_PTR, MAR
	lw Z, CURR_CHAR_DRAW_PTR, MAR

	; should be safe to shift 3 times
	shl Y
	shl Y
	shl Y

	; must then shift 3 more times
	mv SpLo, Y
	mv SpHi, 0

	add SpLo, SpLo
	adc SpHi, SpHi

	add SpLo, SpLo
	adc SpHi, SpHi

	add SpLo, SpLo
	adc SpHi, SpHi

	or SpLo, X
	; put in right page
	or SpHi, high_byte(SCREEN_STATE)

	; save to screen state
	sw Z, SP

	; get stack pointer back
	load_stack_ptr()

	ret(MAR)
}

; runs command in cmd_buffer[0..cmd_ptr)
run_command {
	lw Y, CMD_BUFFER_COMMAND_PTR, MAR
	cmp Y, 0
	; jump if no command
	jeq end, MAR
	; must interpret command by going through all characters!
	
	; super simple parser: look at first character and execute based on that
	
	mv MarLo, 0
	mv MarHi, high_byte(CMD_BUFFER_PAGE)
	lw A, MAR

	cmp A, 'r'
	jeq run_true, MAR
	jmp run_false, MAR

	run_true {
		; eat through characters until a space is found
	
		mv Z, 1

		loop {
			; if Z has reached write pointer then nothing more can be done, go to end since invalid command
			cmp Z, Y
			jeq end, MAR

			mv MarLo, Z
			mv MarHi, high_byte(CMD_BUFFER_PAGE)
			lw A, MAR
			cmp A, ' '
			jeq loop_end, MAR

			add Z, 1

			jmp loop, MAR
			loop_end:
		}
		
		; point after space
		add Z, 1

		; now, parse addr from input
		pusha parse_addr_return

		mv MarLo, Z
		mv MarHi, high_byte(CMD_BUFFER_PAGE)
		
		; NOTE: assumes there are at least 4 characters defined after run
		; TODO: make sure Z + 4 <= cmd_ptr before continuing
		lw A, MAR
		push A

		inc MAR
		lw A, MAR
		push A

		inc MAR
		lw A, MAR
		push A

		inc MAR
		lw A, MAR
		push A

		jmp parse_addr, MAR
		parse_addr_return:

		; here, result will be in stack
		pop MarLo
		pop MarHi
		
		; now, executes code at address
		pusha return_addr
		jmp MAR
		return_addr:
	
		; at this point all state might be corrupted, try and recover state from beginning
		mv A, 1
		sw A, SHOULD_INIT, MAR

		jmp main, MAR
	}

	run_false {
		mv MarLo, 0
		mv MarHi, high_byte(CMD_BUFFER_PAGE)
		lw A, MAR

		cmp A, 'e'
		jeq exit_true, MAR
		jmp exit_false, MAR

		exit_true {
			; jumps directly to beginning
			jmp 0x0000, MAR
		}
		exit_false {
			; TODO: other commands?

			mv MarLo, 0
			mv MarHi, high_byte(CMD_BUFFER_PAGE)
			lw A, MAR

			cmp A, 'h'
			jeq halt_true, MAR
			jmp halt_false, MAR

			halt_true {
				; just halts
				halt
			}
			halt_false {
				; TODO: other commands?
			}
			halt_end:
		}
		exit_end:
	}

	run_end:
	end:
	ret(MAR)
}


; gets characters from stack
; puts result addr into stack
parse_addr {
	; pop into X, keep track of result in SP
	let low = Y
	let high = Z
	mv low, 0
	mv high, 0

	fourth_char {
		pop A
		cmp A, '9'
		jc jc_true, MAR
		jmp jc_false, MAR
		jc_true:
		; A <= 9
		sub A, '0'
		jmp jc_end, MAR
		jc_false:
		; A > 9
		; A - 'a' + 10 = A - ('a' - 10)
		sub A, 'a' - 10
		jc_end:
		; can save value now - bottom half of first byte
		; TODO: or Y, A does not work currently
		mv B, A
		or Y, B
	}

	third_char {
		pop A
		cmp A, '9'
		jc jc_true, MAR
		jmp jc_false, MAR
		jc_true:
		; A <= 9
		sub A, '0'
		jmp jc_end, MAR
		jc_false:
		; A > 9
		; A - 'a' + 10 = A - ('a' - 10)
		sub A, 'a' - 10
		jc_end:
		; can save value now - top half of first byte
		mv X, A
		shl X
		shl X
		shl X
		shl X
		or low, X
	}

	second_char {
		pop A
		cmp A, '9'
		jc jc_true, MAR
		jmp jc_false, MAR
		jc_true:
		; A <= 9
		sub A, '0'
		jmp jc_end, MAR
		jc_false:
		; A > 9
		; A - 'a' + 10 = A - ('a' - 10)
		sub A, 'a' - 10
		jc_end:
		; can save value now - bottom half of second byte
		mv B, A
		or high, B
	}

	first_char {
		pop A
		cmp A, '9'
		jc jc_true, MAR
		jmp jc_false, MAR
		jc_true:
		; A <= 9
		sub A, '0'
		jmp jc_end, MAR
		jc_false:
		; A > 9
		; A - 'a' + 10 = A - ('a' - 10)
		sub A, 'a' - 10
		jc_end:
		; can save value now - bottom half of second byte
		mv X, A
		shl X
		shl X
		shl X
		shl X
		or high, X
	}

	; pop before pushing to stack
	pop MAR

	push high
	push low

	jmp MAR
}

; char data spans 0x3000 - 0x3800
global pCHAR_DATA = 0x3000
#address pCHAR_DATA
#data_file "../../vga_display_v1/ascii_chars.out"
