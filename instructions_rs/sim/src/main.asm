; leaves 0x400-ish bytes for prelude
let terminal_start_jumptable = 0x0400
let snake_start_jumptable = 0x4000

#address 0x0 ; ensures start is first symbol executed
prelude{
	; initialize stack
	lda SP, STACK_START

	; jump to program
	jmp terminal_start_jumptable, MAR
}

; NOTE: includes some variable defs also useful here, better to include at the top
#include "src/utils.asm"

terminal {
	let start_jumptable = terminal_start_jumptable
	#include "src/terminal.asm"
}

snake {
	let start_jumptable = snake_start_jumptable
	#include "src/snake.asm"
}

; current memory layout:
; ROM:
; 0x0000-0x03ff -> prelude
; 0x0400 -> terminal jumptable
; 0x0500 -> terminal code
; 0x3000 - 0x3800 -> char data

; 0x4000 -> snake jumptable
; 0x4100 -> snake code
; 0x7000 - 0x7800 -> snake char data

; RAM:
; 0x8000 - 0x8FFF -> 4k - reserved?
; 0x9000 - 0xAFFF -> 8k - program state
; 0xB000 - 0xFFFF -> 16k - ?

; 0xF000 - 0xFFFF -> 4k - reserved for stack and other system state
; 0xF000 - 0xFFE0 - stack
; 0xffe2 - stack_save_addr
