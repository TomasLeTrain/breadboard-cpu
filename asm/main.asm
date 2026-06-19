#include "istr_defs.asm"

; starts at address 0. sets up the system for running programs
start:
	; initialize stack
	lda	sp, 0xffff

	; do cool program
	jmp multiply3x4
	halt

multiply3x4:
    move x, 0
    move y, 4
	move z, 3

	; load addr into mar so jnz can be called with z argument
	lda mar, .loop
    
    .loop:
        add x, y
		sub z, 1
        jnz z
	
	; move x into a to see final result
	move a, x
	
	halt
