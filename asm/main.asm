#include "istr_defs.asm"

#ruledef {
	draw_character {curr_character: u8}, {video_x: u8}, {video_y: u8} => asm {
		; constants
		; character_base_addr = 0x3000 + 7 + curr_character * 8
		; initial_sp_position = (0x4001 - 64) + 64 * video_y + video_x

		; index of the loop
		move x, 7
		; keeps the current location in vram
		lda sp, (0x4001 - 64) + 64 * {video_y} + {video_x} ; initial_sp_position

		loop:
			; location in rom
			lda mar, 0x3000 + 7 + {curr_character} * 8 ; character_base_addr
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
			jnz loop ; flag updated from sub
	}
}

; macros
#ruledef {
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
	init_sp

	; do cool program
	call clear_vram
	; call copy_character
	draw_character "h" + 0xa - "a", 0, 0
	draw_character "e" + 0xa - "a", 1, 0
	draw_character "l" + 0xa - "a", 2, 0
	draw_character "l" + 0xa - "a", 3, 0
	draw_character "o" + 0xa - "a", 4, 0
	draw_character 0x26,            5, 0
	draw_character "w" + 0xa - "a", 6, 0
	draw_character "o" + 0xa - "a", 7, 0
	draw_character "r" + 0xa - "a", 8, 0
	draw_character "l" + 0xa - "a", 9, 0
	draw_character "d" + 0xa - "a", 10, 0
	draw_character 0x31,            11, 0 
	halt

	; 0x3A - start of capitals (starting with "A")
	; 0x24 - ,
	; 0x25 - .
	; 0x26 - space
	; 0x30 - )
	; 0x31 - !

clear_vram:
	save_sp_addr

	; terminal like colors
	fg_color = 0b1111 
	bg_color = 0b0000

	move z, (fg_color << 4) | bg_color

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

	restore_sp_addr

	return

; copy_character:
; 	move x, 7
;
; 	curr_character = 0xa
; 	video_x = 24
; 	video_y = 40
;
; 	character_base_addr = 0x3000 + 7 + curr_character * 8
; 	; keeps the current location in vram
; 	lda sp, (0x4001 - 64) + 64 * video_y + video_x
;
; 	.loop:
; 		; location in rom
; 		lda mar, character_base_addr
; 		; could be add if rom layout changes
; 		sub mar_lo, x
; 		load z ; read from rom into z
;
; 		; location in vram
; 		lda mar, sp ; load into mar to perform math
; 		add mar_lo, 64
; 		adc mar_hi, 0x00
; 		lda sp, mar ; save back to sp
;
; 		vram_write z
;
; 		sub x, 1
; 		jnz .loop ; flag updated from sub
; 	return



; character data
#addr 0x3000
#d $incbin("../vga_display_v1/charset.out")
