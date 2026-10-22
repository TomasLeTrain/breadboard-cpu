; clears 240 * 40 pixels with given value, at given addr
; NOTE: SP pointer gets used, must save before calling!
fn clear_range_vram(clear_val: u8, start_addr: Address, num_lines: u8){
	fn clear_2_pixels(){
		return {
			sw_vram Z, SP
			inc SP
			sw_vram Z, SP
			inc SP
		}
	}

	fn clear_10_pixels(){
		return {
			clear_2_pixels()
			clear_2_pixels()
			clear_2_pixels()
			clear_2_pixels()
			clear_2_pixels()
		}
	}

	return {
		; value that gets loaded as character everywhere
		mv Z, clear_val

		mv Y, num_lines

		lda SP, start_addr+1

		; addr = (x + 1) | (y << 6)

		y_loop {
			; clear line
			clear_10_pixels()
			clear_10_pixels()
			clear_10_pixels()
			clear_10_pixels()

			; must add offset to SP to get to next line
			add SpLo, ((1<<6) + 1) - (40+1)
			adc SpHi, 0

			sub Y, 1
			lda MAR, y_loop
			jnz MAR
		}
	}
}


; sets all colors equal to given value
; NOTE: SP pointer gets used, must save before calling!
fn clear_screen_colors(color: u8){
	return clear_range_vram(color, 0, 240)
}

; sets all character equal to given value
; NOTE: SP pointer gets used, must save before calling!
fn clear_screen_characters(character: u8){
	return clear_range_vram(character, 1 << 14, 240)
}
