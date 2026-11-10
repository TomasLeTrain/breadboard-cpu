; super simple terminal editor
; goals:
; - vim movements
; - fast/responsive input

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

; things to implement:
;
fn put_ascii_at_position(){
}
fn put_at_position(){
}

global screen_text_offset = 0

; used any time scrolling happens
fn draw_screen_at_line_offset(){
}

; other state
global text_cursor_position_x = 0
global text_cursor_position_y = 0
global screen_cursor_position_x = 0
global screen_cursor_position_y = 0

global vim_mode = 0
; TODO: input buffer state


; actually places instructions at 0x4100
#address 0x4100
start {
	jmp main, MAR
}

main {
	; TODO: everything
	main_loop {
		jmp main_loop, MAR
	}
	halt
}
