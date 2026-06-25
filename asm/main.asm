#include "istr_defs.asm"

#ruledef macros {
	vram_write_1 => asm {
		vram_write z
		inc mar
	}
	vram_write_2 => asm {
		vram_write_1
		vram_write_1
	}
	vram_write_10 => asm {
		vram_write_2
		vram_write_2
		vram_write_2
		vram_write_2
		vram_write_2
	}
	vram_write_41 => asm {
		vram_write_10
		vram_write_10
		vram_write_10
		vram_write_10
		vram_write_1
	}
}

; starts at address 0. sets up the system for running programs
start:
	; initialize stack
	lda	sp, 0xffff

	; do cool program
	jmp write_to_vram

write_to_vram:
    ; cool zig zag pattern
	move z, 0xaa

	lda sp, 0x4000

	.loop:
		; retrieve address that had been saved to sp
		lda mar, sp

		; write to vram 41 times (first character should not matter)
		vram_write_41
		; sp holds the start address for the line, can use that to jump to start of next line
		; need to load to mar to perform math with
		lda mar, sp

		; add (1 << 6)/7th to low address (increasing y by one)
		add mar_lo, 0x40
		; updates mar_hi if overflow
		adc mar_hi, 0x00

		; save this address for after
		lda sp, mar

		; check if done - (nCarry will be off when greater, causing no jump)
		cmp mar_hi, 0x7c ; (1 << 6) | ((240 << 6) >> 8)
		jnc .loop

	jmp copy_character

copy_character:
	move x, 0x7

	curr_character = 0
	character_base_addr = 0x3000 + curr_character + 0x7
	; keeps the current location in vram
	lda sp, 0x4000 + curr_character - 64

	.loop:
		; location in rom
		lda mar, character_base_addr
		; could be add if rom layout changes
		sub mar_lo, x
		load z ; read from rom into z
 
		; location in vram
		lda mar, sp ; load into mar to perform math
		add mar_lo, 64
		adc mar_hi, 0x00
		lda sp, mar ; save back to sp

		vram_write z

		sub x, 1
		jnz .loop ; flag updated from sub
	halt
