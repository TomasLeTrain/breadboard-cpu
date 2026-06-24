#include "istr_defs.asm"

; starts at address 0. sets up the system for running programs
start:
	; initialize stack
	lda	sp, 0xffff

	; do cool program
	jmp write_to_vram
	halt

write_to_vram:
    ; cool zig zag pattern
	move z, 0xaa
	; start at character ram portion
	lda sp, 0x4000 - 1

	.loop:
		; write contents of z into vram at mar
		inc sp
		lda mar, sp
		vram_write z
		; check if mar is done
		move a, mar_hi
		; isolate 2nd msb from mar_hi
		and a, 0x40
		; loops back if 2nd msb becomes zero (end of vram)
		jnz .loop

	halt

; multiply3x4:
;     move x, 0
;     move y, 4
; 	move z, 3
;
; 	; load addr into mar so jnz can be called with z argument
; 	lda mar, .loop
;
;     .loop:
;         add x, y
; 		sub z, 1
;         jnz z
;
; 	; move x into a to see final result
; 	move a, x
;
; 	halt
