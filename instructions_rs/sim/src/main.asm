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
