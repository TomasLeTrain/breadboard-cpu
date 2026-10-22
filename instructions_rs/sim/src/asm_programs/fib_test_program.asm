; TODO: make into test
start:
	; initialize stack
	lda SP, 0xffe0 ; lda sp, sp_start_addr

	; guarantee a clear of carry flag
	mv A, 0
	add A, 0

	; push return addr fib before data
	jsr(fib_32)

	; retrieve result from stack into registers
	pop X
	pop Y
	pop Z
	pop A

	halt


fib_32 {
	fn inc_sw(){
		return {
			inc MAR
			sw A, MAR
		}
	}

	; 0x8040 - a
	; 0x8044 - b
	; 0x8048 - z - counter
	; lsb - msb

	; set initial condition for a
	sw_u8(1, 0x8040, A)

	; load 0 into all other numbers
	; a
	mv A, 0 
	inc_sw()
	inc_sw()
	inc_sw()
	; b - 0x8044
	inc_sw()
	inc_sw()
	inc_sw()
	inc_sw()
	; z - 0x8048
	inc_sw()
	inc_sw()
	inc_sw()
	inc_sw()

	; set counter - here its for 2^16 - 1th fib number
	sw_u8(255, 0x8048, A)
	sw_u8(255, 0x8048+1, A)

	fn push_u32_at_addr(addr: Address){
		return {
			lw A, addr+3, MAR
			push A
			lw A, addr+2, MAR
			push A
			lw A, addr+1, MAR
			push A
			lw A, addr, MAR
			push A
		}
	}

	; saves to a
	fn sum_routine(a_addr: Address, b_addr: Address, routine_addr: Address){
		return {
			pusha sum_return
			; function parameters
			push_u32_at_addr(a_addr)
			push_u32_at_addr(b_addr)

			jmp routine_addr, MAR
			sum_return:

			; now store result into a
			pop A
			sw A, a_addr + 3, MAR
			pop A
			sw A, a_addr + 2, MAR
			pop A
			sw A, a_addr + 1, MAR
			pop A
			sw A, a_addr, MAR
		}
	}

	loop {
		; a = a + b
		sum_routine(0x8040, 0x8044, sum_u32)
		; b = b + a
		sum_routine(0x8044, 0x8040, sum_u32)

		call3 {
			pusha sum_return
			; function parameters
			push_u32_at_addr(0x8048)

			; push -1
			push 0xff
			push 0xff
			push 0xff
			push 0xff

			jmp sum_u32, MAR
			sum_return:

			; now store result into counter
			pop A
			sw A, 0x8048 + 3, MAR
			pop A
			sw A, 0x8048 + 2, MAR
			pop A
			sw A, 0x8048 + 1, MAR
			pop A
			sw A, 0x8048, MAR
		}

		; at this point must check if counter is zero to see if we are finished
		; check if its zero
		mv X, 0
		lw B, 0x8048 + 3, MAR
		or X, B
		lw B, 0x8048 + 2, MAR
		or X, B
		lw B, 0x8048 + 1, MAR
		or X, B
		lw B, 0x8048, MAR
		or X, B

		lda MAR, loop
		cmp X, 0
		jnz MAR
	}

	; want to store result to stack, so must save return addr elsewhere
	; get return address and save it to memory, next step changes stack
	pop	A
	sw A, 0xfff0, MAR
	inc MAR
	pop	A
	sw A, MAR ; sw A, 0xfff1, MAR

	fn push_lw(){
		return {
			inc MAR
			lw A, MAR
			push A
		}
	}

	; return routine
	lw A, 0x8040, MAR
	push A

	push_lw()
	push_lw()
	push_lw()

	; get return address back
	lw A, 0xfff0, MAR
	; since next addr is right after can just increase MAR and load it into B
	; both approaches take the exact same # of clock cycles, but this approach is one less byte long
	inc MAR
	lw B, MAR ; lw B, 0xfff1, MAR

	; load into mar
	mv MarLo, A
	mv MarHi, B

	jmp MAR
}

; 32 bit sum of 32 bit nums in stack
; input is 2, 32bit numbers in the stack in the format:
; [return addr][B[3]][B[2]][B[1]][B[0]][A[3]][A[2]][A[1]][A[0]]
; stack after operation looks like:
; [C[0]][C[1]][C[2]][C[3]]

sum_u32 {
	; func defined only in this scope
	fn pop_sw(){
		return {
			inc MAR
			pop A
			sw A, MAR 
		}
	}
	
	; 0
	pop A
	sw A, 0x8080, MAR ; loads 0x8080 into MAR once 
	; 1
	pop_sw()
	; 2
	pop_sw()
	; 3
	pop_sw()
	; 4
	pop_sw()
	; 5
	pop_sw()
	; 6
	pop_sw()
	; 7
	pop_sw()

	; want to store result to stack, so must save return addr elsewhere
	; get return address and save it to memory, next step changes stack
	pop	A
	sw A, 0xfff0, MAR
	inc MAR
	pop	A
	sw A, MAR ; sw A, 0xfff1, MAR

	; get nums
	lw X, 0x8080, MAR
	lw Y, 0x8084, MAR
	; add nums
	; NOTE: no carry on first add
	add X, Y
	; push result to stack
	push X

	; get nums
	inc MAR
	lw Y, MAR
	lw X, 0x8081, MAR
	; add nums
	adc X, Y
	; push result to stack
	push X

	; get nums
	inc MAR
	lw X, MAR
	lw Y, 0x8086, MAR
	; add nums
	adc X, Y
	; push result to stack
	push X

	; get nums
	inc MAR
	lw Y, MAR
	lw X, 0x8083, MAR
	; add nums
	adc X, Y
	; push result to stack
	push X

	; get return address back
	lw A, 0xfff0, MAR
	; since next addr is right after can just increase MAR and load it into B
	; both approaches take the exact same # of clock cycles, but this approach is one less byte long
	inc MAR
	lw B, MAR ; lw B, 0xfff1, MAR

	; load into mar
	mv MarLo, A
	mv MarHi, B

	; return
	jmp MAR
}


fn low_byte(x: u16) {
	let low_byte: u8 = x & 0xff
	return low_byte
}

fn high_byte(x: u16) {
	let high_byte: u8 = (x >> 8) & 0xff
	return high_byte
}

fn jsr_return() {
	return {
		pop MAR
		jmp MAR
	}
}

fn jsr(addr: Address) {
	return {
		pusha return_addr
		jmp addr, MAR
		return_addr:
	}
}


fn jnz_end(reg: Register, addrReg: AddressRegister, addr: Address) {
	return {
		sub reg, 1
		lda addrReg, addr
		jnz reg, addrReg
	}
}

fn sw_u8(x: u8, addrReg: Address, scratch_reg: Register) {
	return {
		mv scratch_reg, x
		sw scratch_reg, addrReg, MAR
	}
}
