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

global background_color = 0b11110000
global main_color = 0b11000000

; other state
; X position can be reused for both text and screen
global CURSOR_POSITION_X = 0x9000
global CURSOR_POSITION_Y = CURSOR_POSITION_X + 1

; when on, colors at cursor position flip on screen
global CURSOR_VISUAL_FLIPPED = CURSOR_POSITION_Y + 1

global VIM_MODE = CURSOR_VISUAL_FLIPPED + 1

global INPUT_BUFFER_READ_PTR = VIM_MODE + 1
global INPUT_BUFFER_WRITE_PTR = INPUT_BUFFER_READ_PTR + 1

global SCREEN_TEXT_OFFSET = INPUT_BUFFER_WRITE_PTR + 1

global INPUT_BUFFER_PAGE = 0x9100

global NORMAL_MODE_VAL: u8 = 0
global INSERT_MODE_VAL: u8 = 1

fn setVariable(variable_ptr: Address, value: u8){
	return {
		mv A, value
		sw A, variable_ptr, MAR
	}
}

fn addToVariable(variable_ptr: Address, increment_value: u8){
	return {
		lw A, variable_ptr, MAR
		add A, increment_value 
		sw A, variable_ptr, MAR
	}
}

fn subtractToVariable(variable_ptr: Address, decrement_value: u8){
	return {
		lw A, variable_ptr, MAR
		sub A, decrement_value 
		sw A, variable_ptr, MAR
	}
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
	mv A, 3
	sw A, CURSOR_POSITION_X, MAR
	sw A, CURSOR_POSITION_Y, MAR

	mv A, NORMAL_MODE_VAL
	sw A, VIM_MODE, MAR

	clear_screen_colors(background_color)

	let default_character = 0
	; let default_character = 0b0011000

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
		jeq loop_break, MAR

		; MAR = &buffer[read_ptr]
		load_mar_page_ptr(INPUT_BUFFER_PAGE, INPUT_BUFFER_READ_PTR)

		; otherwise, process current input
		; X = buffer[read_ptr]
		lw X, MAR

		; zero is no character, just skip
		cmp X, 0
		jeq loop_increase, MAR

		; proces based on mode
		lw A, VIM_MODE, MAR
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

		loop_increase:
		; read_ptr = read_ptr + 1
		addToVariable(INPUT_BUFFER_READ_PTR, 1)

		jmp loop, MAR
		loop_break:
	}

	ret(MAR)
}

normal_mode_key_update {
	; before anything else, remove cursor at current position by redrawing character
	; then if cursor gets changed by cases it gets redrawn at correct position

	; save X before calling, it holds the holden
	push X
	setVariable(CURSOR_VISUAL_FLIPPED, 0)
	jsr(flip_colors_at_cursor)
	pop X

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
	cmp X, 'i'
	jeq insert_case, MAR
	jmp key_else, MAR

	; IF key == enter
	enter_case {
		; same as down case but resets X
		; can safely set X = 0 without worrying about bounds
		mv A, 0
		sw A, CURSOR_POSITION_X, MAR

		jmp down_case, MAR
	}

	left_case {
		; X--
		mv X, 255
		mv Y, 0
		jmp call_move_cursor, MAR
	}
	down_case {
		; Y++
		mv X, 0
		mv Y, 1
		jmp call_move_cursor, MAR
	}
	up_case {
		; Y--
		mv X, 0
		mv Y, 255
		jmp call_move_cursor, MAR
	}
	right_case {
		; X++
		mv X, 1
		mv Y, 0
		jmp call_move_cursor, MAR
	}

	call_move_cursor:
	jsr(move_cursor)
	jmp key_cases_end, MAR

	insert_case {
		; switch to insert mode
		setVariable(VIM_MODE, INSERT_MODE_VAL)
		; jmp key_cases_end, MAR
	}

	; ELSE
	key_else {
		; ignore all other characters while in normal mode
		; jmp key_cases_end, MAR
	}

	key_cases_end:
	
	; in normal mode cursor is just flipping the color, done regardless of any key cases
	setVariable(CURSOR_VISUAL_FLIPPED, 1)
	jsr(flip_colors_at_cursor)

	ret(MAR)
}

; moves cursor by X, Y (registers) amount, keeping it in bounds and updating the screen text offset if neccesary
move_cursor {
	x_update {
		; skip if delta is zero
		cmp X, 0
		jeq text_y_update, MAR

		lw Z, CURSOR_POSITION_X, MAR

		; move by offset
		add Z, X

		; check for left bound
		cmp 128, Z
		jc left_bound_hit, MAR

		; check for right bound
		cmp Z, 39
		; jc is equivalent to A <= B, so jumps if Z <= 39 (in bounds)
		jc no_bounds_hit, MAR
		; if didn't jump then right bound got hit
		right_bound_hit:
		; cap x at 39
		mv X, 39

		jmp bounds_end, MAR
		left_bound_hit:
		; cap x at 0
		mv X, 0
		jmp bounds_end, MAR

		no_bounds_hit:
		; halt
		; no bounds hit, can update safely to new value
		mv X, Z
		; jmp bounds_end, MAR
		bounds_end:

		; save back resulting X
		sw X, CURSOR_POSITION_X, MAR
	}

	text_y_update {
		; skip both y and offset update if zero
		cmp Y, 0
		jeq cursor_update_end, MAR

		lw Z, CURSOR_POSITION_Y, MAR

		; move by offset
		add Z, Y

		; check for left bound
		cmp 128, Z
		jc up_bound_hit, MAR

		; check for right bound
		cmp Z, 29
		; jc is equivalent to A <= B, so jumps if Z <= 39 (in bounds)
		jc no_bounds_hit, MAR
		; if didn't jump then right bound got hit
		down_bound_hit:
		; cap y at 29
		mv X, 29

		; taking out 29 gives the # of lines that must be scrolled
		sub Z, 29
		mv Y, Z

		jmp bounds_end, MAR
		up_bound_hit:
		; cap y at 0
		mv X, 0
		; amount to scroll by is the addition of Y + CURSOR_POSITION_Y, which is already in Z
		mv Y, Z

		jmp bounds_end, MAR
		no_bounds_hit:
		; no scrolling occurs, update text y with new value and set screen offset to zero
		mv X, Z
		mv Y, 0
		; jmp bounds_end, MAR
		bounds_end:

		; save back resulting Y offset
		sw X, CURSOR_POSITION_Y, MAR
	}

	; check if scrolling is neccesary
	cmp Y, 0
	jeq cursor_update_end, MAR
	
	; here scrolling must occur, apply same logic for checking bounds
	scrolling_update {
		lw Z, SCREEN_TEXT_OFFSET, MAR

		; move by delta
		add Z, Y

		; do check bounds:
		mv A, Z
		; check for left bound
		and A, 0b10000000
		jnz up_bound_hit, MAR

		; check for right bound
		; offset is from 0 to the first line, so 88 + 39 = 127 is location of last line
		cmp Z, 88
		; jc is equivalent to A <= B, so jumps if Z <= 88 (in bounds)
		jc no_bounds_hit, MAR
		; if didn't jump then right bound got hit
		down_bound_hit:
		; cap offset at 88
		mv X, 88

		jmp bounds_end, MAR
		up_bound_hit:
		; cap offset at 0
		mv X, 0

		jmp bounds_end, MAR
		no_bounds_hit:
		; no bounds get hit, change offset without issues
		mv X, Z

		; jmp bounds_end, MAR
		bounds_end:

		; save back resulting offset
		sw X, SCREEN_TEXT_OFFSET, MAR
	}

	cursor_update_end:

	ret(MAR)
}

insert_mode_key_update {
	ret(MAR)
}

; turns a position into an addr without the page
; leaves result in stack
position_to_addr {
	; should be safe to shift 3 times
	shl Y
	shl Y
	shl Y

	; must then shift 3 more times
	mv MarLo, Y
	mv MarHi, 0

	add MarLo, MarLo
	adc MarHi, MarHi

	add MarLo, MarLo
	adc MarHi, MarHi

	add MarLo, MarLo
	adc MarHi, MarHi

	or MarLo, X

	mv X, MarLo
	mv Y, MarHi

	ret(MAR)
}

; sets X equal to color at cursor position
get_cursor_color {
	;get color at position
	lw X, CURSOR_POSITION_X, MAR
	lw Y, CURSOR_POSITION_Y, MAR
	
	jsr(position_to_addr)

	mv MarLo, X
	mv MarHi, Y

	; put in correct page
	or MarHi, high_byte(TEXT_COLOR_STATE)

	; fetch current color
	lw X, MAR
	
	ret(MAR)
}

flip_colors_at_cursor {
	jsr(get_cursor_color)
	; now x has current color
	; check if must flip or not
	
	lw Y, CURSOR_VISUAL_FLIPPED, MAR
	lda MAR, flipped
	jnz Y, MAR
	not_flipped:
	; no change to the color

	jmp flipped_end, MAR
	flipped:
	; flip position of 4 lsbs and 4msbs

	; Y becomes the new upper bits, X the lower bits
	mv Y, X
	shr X
	shr X
	shr X
	shr X
	shl Y
	shl Y
	shl Y
	shl Y
	or X, Y
	; mv X, 0b00001111

	; jmp flipped_end, MAR
	flipped_end:

	; now, redraw at cursor position with given color
	mv Z, X
	;get color at position
	lw X, CURSOR_POSITION_X, MAR
	lw Y, CURSOR_POSITION_Y, MAR
	jsr(draw_color)

	ret(MAR)
}

; arguments:
; X - x position of character
; Y - y position of character
; Z - color to draw
draw_color {
	; sets up address for pixel at character position (X, Y)
	fn setup_vram_addr(){
		return {
			; must shift in total 9 times to the left (3 for character line, 6 for position in addr)
			; equal to setting high byte = y and shifting left once 
			shl Y

			; must write to 8 blocks most likely, loop?
			; MAR = base_addr + 1
			; lda MAR, BASE_CHAR_POSITION + 1
			lda MAR, 1

			; MAR += x
			add MarLo, X
			; MAR += y<<8, really should be OR but the carry from last add must be propagated, can compress into one instruction 
			adc MarHi, Y
		}
	}

	fn line_write(Reg0: Register, Reg1: Register, addrReg: AddressRegister){
		return {
			; known offset to get to next line
			add Reg0, 1<<6
			adc Reg1, 0
			sw_vram Z, addrReg
		}
	}

	setup_vram_addr()
	sw_vram Z, MAR
	line_write(MarLo, MarHi, MAR) ; 2
	line_write(MarLo, MarHi, MAR) ; 3
	line_write(MarLo, MarHi, MAR) ; 4
	line_write(MarLo, MarHi, MAR) ; 5
	line_write(MarLo, MarHi, MAR) ; 6
	line_write(MarLo, MarHi, MAR) ; 7
	line_write(MarLo, MarHi, MAR) ; 8

	ret(MAR)
}

; TODO: 
; put_character(character, color) -> puts at text/color cursor position, draws to screen

; screen specific routines:
; redraw_at_cursor() -> redraws normally at cursor position - used before cursor changes to new position to restore screen state
; draw_thin_cursor() -> redraws character at position but with thin bar on the left


; char data spans 0x3000 - 0x3800
global pCHAR_DATA = 0x7000
#address pCHAR_DATA
#data_file "../../vga_display_v1/ascii_chars.out"
