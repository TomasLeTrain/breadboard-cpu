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
	halt

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

	halt
