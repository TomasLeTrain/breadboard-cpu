; fn global_test(){
; 	return thing
; }

; mv A, global_test()
mv A, thing

local thing : u8 = 1
; start:
; 	; initialize stack
; 	lda SP, 0xffe0 ; lda sp, sp_start_addr
;
; 	jsr(fib)
;
; 	mv A, X
; 	mv B, Y
;
; 	halt
;
;
; ; x = x + y
; ; y = x + y
; ; ...
; fib {
; 	; initial conditions
; 	mv X, 0
; 	mv Y, 1
;
; 	let num_iters = 128 % 5
;
; 	mv Z, num_iters
;
; 	; lda MAR, loop
;
; 	loop {
; 		add X, Y
; 		add Y, X
;
; 		; sub Z, 1
; 		; jnz Z, MAR
; 		jnz_end(Z, MAR, loop)
; 	}
;
; 	; return routine
; 	jsr_return()
; }
;
; fn low_byte(x: u16) {
; 	let low_byte: u8 = x & 0xff
; 	return low_byte
; }
;
; fn high_byte(x: u16) {
; 	let high_byte: u8 = (x >> 8) & 0xff
; 	return high_byte
; }
;
; fn jsr_return() {
; 	return {
; 		pop MAR
; 		jmp MAR
; 	}
; }
;
; fn jsr(addr: Address) {
; 	return {
; 		pusha return_addr
; 		jmp addr, MAR
; 		return_addr:
; 	}
; }
;
;
; fn jnz_end(reg: Register, addrReg: AddressRegister, addr: Address) {
; 	return {
; 		sub reg, 1
; 		lda addrReg, addr
; 		jnz reg, addrReg
; 	}
; }

; placing outside rom fails
; #address 0x8000
; placing outside rom fails
; #address 0x7400
; jmp MAR
;
; let thing = 1
; #address 0x6060
; #include "src/other_program.asm"

; #address 0x6070
; let embedded_data: u8 = 10
; #data embedded_data
;
; #data_file "src/test.data"
; data_file_end:
