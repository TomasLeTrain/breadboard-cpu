start:
	; initialize stack
	lda SP, 0xffe0 ; lda sp, sp_start_addr

	; guarantee a clear of carry flag
	mv A, 0
	add A, 0

	; push return addr fib before data
	pusha push_return
	jmp fib_32, MAR     
	push_return:

	; retrieve result from stack into registers
	pop X
	pop Y
	pop Z
	pop A

	halt


fib_32 {
	; 0x8040 - a
	; 0x8044 - b
	; 0x8048 - z - counter
	; lsb - msb

	; set initial condition for a
	mv A, 1
	sw A, 0x8040, MAR

	; load 0 into all other numbers
	mv A, 0 
	inc MAR
	; a
	sw A, MAR
	inc MAR
	sw A, MAR
	inc MAR
	sw A, MAR
	; b - 0x8044
	inc MAR
	sw A, MAR
	inc MAR
	sw A, MAR
	inc MAR
	sw A, MAR
	inc MAR
	sw A, MAR
	; z - 0x8048
	inc MAR
	sw A, MAR 
	inc MAR
	sw A, MAR
	inc MAR
	sw A, MAR
	inc MAR
	sw A, MAR

	; set counter - here its for 2^16 - 1th fib number
	mv A, 255
	sw A, 0x8048, MAR
	mv A, 255 
	sw A, 0x8048+1, MAR

	loop {
		call1 {
			pusha sum_return
			; function parameters
			; push 
			lw A, 0x8044 + 3, MAR
			push A
			lw A, 0x8044 + 2, MAR
			push A
			lw A, 0x8044 + 1, MAR
			push A
			lw A, 0x8044, MAR
			push A

			lw A, 0x8040 + 3, MAR
			push A
			lw A, 0x8040 + 2, MAR
			push A
			lw A, 0x8040 + 1, MAR
			push A
			lw A, 0x8040, MAR
			push A

			jmp sum_u32, MAR
			sum_return:

			; now store result into a
			pop A
			sw A, 0x8040 + 3, MAR
			pop A
			sw A, 0x8040 + 2, MAR
			pop A
			sw A, 0x8040 + 1, MAR
			pop A
			sw A, 0x8040, MAR
		}

		call2 {
			pusha sum_return
			; function parameters
			; push 
			lw A, 0x8044 + 3, MAR
			push A
			lw A, 0x8044 + 2, MAR
			push A
			lw A, 0x8044 + 1, MAR
			push A
			lw A, 0x8044, MAR
			push A

			lw A, 0x8040 + 3, MAR
			push A
			lw A, 0x8040 + 2, MAR
			push A
			lw A, 0x8040 + 1, MAR
			push A
			lw A, 0x8040, MAR
			push A

			jmp sum_u32, MAR
			sum_return:

			; now store result into b
			pop A
			sw A, 0x8044 + 3, MAR
			pop A
			sw A, 0x8044 + 2, MAR
			pop A
			sw A, 0x8044 + 1, MAR
			pop A
			sw A, 0x8044, MAR
		}

		call3 {
			pusha sum_return
			; function parameters
			lw A, 0x8048 + 3, MAR
			push A
			lw A, 0x8048 + 2, MAR
			push A
			lw A, 0x8048 + 1, MAR
			push A
			lw A, 0x8048, MAR
			push A

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

	; return routine
	lw A, 0x8040, MAR
	push A
	inc MAR
	lw A, MAR
	push A
	inc MAR
	lw A, MAR
	push A
	inc MAR
	lw A, MAR
	push A

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
	; 0
	pop A
	sw A, 0x8080, MAR ; loads 0x8080 into MAR once 
	; 1
	inc MAR
	pop A
	sw A, MAR 
	; 2
	inc MAR
	pop A
	sw A, MAR 
	; 3
	inc MAR
	pop A
	sw A, MAR 
	; 4
	inc MAR
	pop A
	sw A, MAR 
	; 5
	inc MAR
	pop A
	sw A, MAR 
	; 6
	inc MAR
	pop A
	sw A, MAR 
	; 7
	inc MAR
	pop A
	sw A, MAR 

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
