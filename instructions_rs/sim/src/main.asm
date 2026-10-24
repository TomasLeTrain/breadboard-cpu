let terminal_start_jumptable = 0x0004
let snake_start_jumptable = 0x4000

; TODO: should move stack init before jumping, leave space for that as well
; jump to some program at first
#address 0x0 ; ensures start is first symbol executed
jmp terminal_start_jumptable, MAR

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
; 0x0000 -> jump instr to jump to jumptable for some program
; 0x0004 -> terminal jumptable
; 0x3000 - 0x3800 -> char data

; 0x4000 -> snake jumptable and code
; 0x7000 - 0x7800 -> snake char data

; RAM:
; 0x8000 - 0x8FFF -> 4k - reserved?
; 0x9000 - 0xAFFF -> 8k - program state
; 0xB000 - 0xFFFF -> 16k - ?

; 0xF000 - 0xFFFF -> 4k - reserved for stack and other system state
; 0xF000 - 0xFFE0 - stack
; 0xffe2 - stack_save_addr
