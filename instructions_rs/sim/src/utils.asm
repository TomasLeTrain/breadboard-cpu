global STACK_START: Address = 0xffe0
global STACK_SAVE_ADDR: Address = 0xffe2

fn jsr(addr: Address) {
	return {
		pusha return_addr
		jmp addr, MAR
		return_addr:
	}
}

fn high_byte(x: u16) {
	let high_byte: u8 = (x >> 8) & 0xff
	return high_byte
}


; saves to tmp save addr
fn save_stack_ptr(){
	return {
		sw SpLo, STACK_SAVE_ADDR, MAR
		sw SpHi, STACK_SAVE_ADDR+1, MAR
	}
}

; loads from tmp save addr
fn load_stack_ptr(){
	return {
		lw SpLo, STACK_SAVE_ADDR, MAR
		lw SpHi, STACK_SAVE_ADDR+1, MAR
	}
}

; some other common utils
#include "src/video_utils.asm"
