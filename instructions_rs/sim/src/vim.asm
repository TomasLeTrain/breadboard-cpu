; super simple terminal editor
; goals:
; - vim movements
; - fast/responsive input

; ram mem layout:
; 0x9000 - scratch variables
; 0x9100 - input buffer
; 0xB000 - 0xEFFF -> text state (ascii and colors)

; jumptable address at 0x4000
#address start_jumptable
jmp start, MAR

; store individually of screen state?

; list of lines of fixed 40 width
; num lines grows as the file grows?

; 80 bytes (ascii + color per char) * # lines
; say some upper 128 # lines: 
; 80 * 128 = 10kb!
; however if padding gets used this becomes 16kb
; less compressed than buffer approach but way easier to render

; where text state gets stored
; TEXT_ASCII_STATE[(y << 6) | x] = ascii character at position
; TEXT_COLOR_STATE[(y << 6) | x] = color at position
; should be from 0xB000 - 0xCFFF
global TEXT_ASCII_STATE = 0xB000
; should be from 0xD000 - 0xEFFF
global TEXT_COLOR_STATE = 0xD000

global background_color = 0b00000000
global main_color = 0b11000000

; things to implement:
;
; fn put_ascii_at_position(){
; }
; fn put_color_at_position(){
; }

global SCREEN_TEXT_OFFSET = 0

; draws from the saved text state direct to screen
; used any time scrolling happens
; fn draw_screen_at_line_offset(){
; }

; other state
global TEXT_CURSOR_POSITION_X = 0x9000
global TEXT_CURSOR_POSITION_Y = TEXT_CURSOR_POSITION_X + 1
global SCREEN_CURSOR_POSITION_X = TEXT_CURSOR_POSITION_Y + 1
global SCREEN_CURSOR_POSITION_Y = SCREEN_CURSOR_POSITION_X + 1

global VIM_MODE = SCREEN_CURSOR_POSITION_Y + 1

global INPUT_BUFFER_READ_PTR = VIM_MODE + 1
global INPUT_BUFFER_WRITE_PTR = INPUT_BUFFER_READ_PTR + 1

global INPUT_BUFFER_PAGE = 0x9100

global NORMAL_MODE_VAL: u8 = 0
global INSERT_MODE_VAL: u8 = 1

fn addToVariable(variable_ptr: Address, increment_value: u8){
	lw A, variable_ptr, MAR
	add A, increment_value 
	sw A, variable_ptr, MAR
}

fn subtractToVariable(variable_ptr: Address, decrement_value: u8){
	lw A, variable_ptr, MAR
	sub A, decrement_value 
	sw A, variable_ptr, MAR
}



; actually places instructions at 0x4100
#address 0x4100
start {
	jmp main, MAR
}

main {
	jsr(init_state)

	main_loop {
		jsr(input_update)
		jsr(general_update)

		jmp main_loop, MAR
	}
	halt
}

init_state {
	; init any memory here
	save_stack_ptr()

	first_init:
	mv A, 0
	sw A, INPUT_BUFFER_READ_PTR, MAR
	mv A, 1
	sw A, INPUT_BUFFER_WRITE_PTR, MAR
	mv A, 0
	sw A, INPUT_BUFFER_PAGE, MAR

	mv A, 0
	sw A, SCREEN_TEXT_OFFSET, MAR

	; other state
	mv A, 0
	sw A, TEXT_CURSOR_POSITION_X, MAR
	sw A, TEXT_CURSOR_POSITION_Y, MAR
	sw A, SCREEN_CURSOR_POSITION_X, MAR
	sw A, SCREEN_CURSOR_POSITION_Y, MAR

	mv A, NORMAL_MODE_VAL
	sw A, VIM_MODE, MAR

	clear_screen_colors(background_color)

	let default_character = 0

	clear_screen_characters(default_character)

	fn clear_text_state_at_addr(addr: Address, fill_value: u8){
		return {
			; clear screen state
			lda SP, addr
			mv X, fill_value

			; 128 lines in total
			mv Z, 128

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
		}
	}

	clear_text_state_at_addr(TEXT_ASCII_STATE, default_character)
	clear_text_state_at_addr(TEXT_COLOR_STATE, background_color)

	load_stack_ptr()

	ret(MAR)
}

input_update {
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
	; store character
	sw X, MAR

	; increase write ptr
	mv A, MarLo
	add A, 1
	sw A, INPUT_BUFFER_WRITE_PTR, MAR

	eq_end:

	ret(MAR)
}

general_update {
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

		; proces based on mode
		lw A, INPUT_BUFFER_READ_PTR, MAR
		cmp A, NORMAL_MODE_VAL
		jeq normal_mode, MAR
		cmp A, INSERT_MODE_VAL
		jeq insert_mode, MAR
		jmp modes_switch_end, MAR

		normal_mode {
			push X
			push Z
			jsr(normal_mode_key_update)
			pop Z
			pop X

			jmp modes_switch_end, MAR
		}
		insert_mode {
			push X
			push Z
			jsr(insert_mode_key_update)
			pop Z
			pop X
			; jmp modes_switch_end, MAR
		}

		modes_switch_end:

		; read_ptr = read_ptr + 1
		addToVariable(INPUT_BUFFER_READ_PTR, 1)
		
		jmp loop, MAR
		loop_end:
	}

	ret(MAR)
}

normal_mode_key_update {
	cmp X, 13
	jeq enter_case, MAR
	; backspace key is just going left
	cmp X, 8
	jeq left_case, MAR
	cmp X, 'h'
	jeq left_case, MAR

	cmp X, 'j'
	jeq down_case, MAR

	cmp X, 'k'
	jeq up_case, MAR

	cmp X, 'l'
	jeq right_case, MAR

	; TODO: insert key case
	; cmp X, 'i'
	; jeq insert_case, MAR
	jmp key_else, MAR

	; before anything else, remove cursor at current position by redrawing character
	; then if cursor gets changed by cases it gets redrawn at correct position
	; TODO: impl

	; IF key == enter
	enter_case {
		; goes to beginning of new line

		; TODO: handle when going off screen
		; maybe changing coordinates should be handled by another function that checks bounds and upates screen shift

		; X = 0, Y++
		mv A, 0
		sw A, TEXT_CURSOR_POSITION_X, MAR
		addToVariable(TEXT_CURSOR_POSITION_Y, 1)

		mv A, 0
		sw A, SCREEN_CURSOR_POSITION_X, MAR
		addToVariable(SCREEN_CURSOR_POSITION_Y, 1)

		jmp key_cases_end, MAR
	}

		left_case {
		; TODO: do stuff
		jmp key_cases_end, MAR
	}

	; ELSE
	key_else {
		; TODO: do stuff
		; jmp key_cases_end, MAR
	}
	
	; in normal mode cursor is just flipping the color, done regardless of any key cases
	; TODO: impl thick cursor draw

	ret(MAR)
}

insert_mode_key_update {
	ret(MAR)
}


; TODO: 
; put_character(character, color) -> puts at text/color cursor position, draws to screen

; screen specific routines:
; redraw_at_cursor() -> redraws normally at cursor position - used before cursor changes to new position to restore screen state
; draw_thick_cursor() -> inverts color at position
; draw_thin_cursor() -> redraws character at position but with thin bar on the left
