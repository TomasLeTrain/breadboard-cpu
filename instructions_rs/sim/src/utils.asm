global STACK_START: Address = 0xffe0
global STACK_SAVE_ADDR: Address = 0xffe2
global STACK_SAVE_ADDR_2: Address = 0xffe4

fn jsr(addr: Address) {
	return {
		pusha return_addr
		jmp addr, MAR
		return_addr:
	}
}

fn low_byte(x: u16) {
	let high_byte: u8 = x & 0xff
	return high_byte
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


; saves to tmp save addr
fn save_stack_ptr_2(){
	return {
		sw SpLo, STACK_SAVE_ADDR_2, MAR
		sw SpHi, STACK_SAVE_ADDR_2+1, MAR
	}
}

; loads from tmp save addr
fn load_stack_ptr_2(){
	return {
		lw SpLo, STACK_SAVE_ADDR_2, MAR
		lw SpHi, STACK_SAVE_ADDR_2+1, MAR
	}
}

fn ret(addr_reg: AddressRegister) {
	return {
		pop addr_reg
		jmp addr_reg
	}
}

; loads MAR with the address in memory corresponding to where the ptr points in the page
fn load_mar_page_ptr(page_start: Address, ptr: Address){
	return {
		; load buffer end pointer into MAR
		lw MarLo, ptr, MAR
		mv MarHi, high_byte(page_start)
	}
}

; loads SP with the address in memory corresponding to where the ptr points in the page
fn load_sp_page_ptr(page_start: Address, ptr: Address){
	return {
		; load buffer end pointer into MAR
		lw SpLo, ptr, SP
		mv SpHi, high_byte(page_start)
	}
}

; some other common utils
#include "src/video_utils.asm"
