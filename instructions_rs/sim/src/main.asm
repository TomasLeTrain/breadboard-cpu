#address 0x0 ; ensures start is first symbol executed
jmp start, MAR

; NOTE: includes some variable defs also useful here, better to include at the top
#include "src/utils.asm"

; terminal program

; Goals:
; prompt input from keyboard
; interpret commands and execute them (what format?)

; input buffer start
global INPUT_BUFFER_START = 0x9000

; input buffer end ptr
global INPUT_BUFFER_END_PTR = 0x9100

global background_color = 0b11000100
global main_color = 0b11000000

start {
	; initialize stack
	lda SP, STACK_START

	jsr(init_state)

	halt
}


init_state {
	; init any memory here
	clear_screen_colors(background_color)
	clear_screen_characters(0)

	pop MAR
	jmp MAR
}


; char data spans 0x3000 - 0x3800
global pCHAR_DATA = 0x3000
#address pCHAR_DATA
#data_file "../../vga_display_v1/charset.out"
