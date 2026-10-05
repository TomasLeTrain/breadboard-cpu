start:
	; initialize stack
	lda SP, 0xffe0 ; lda sp, sp_start_addr

	; guarantee a clear of carry flag
	mv A, 0
	add A, 0

	; push return addr fib before data
	pusha push_return

	; push the left and right nums
	; right first
	push 0
	push 0
	push 0
	push 0x32

	; push the left and right nums
	; left second
	push 0
	push 0
	push 0
	push 0x14

	; jmp fib, MAR     ; jump to func

	; 618 cycles (total program)
	; 0x6A = 106 bytes total program
	; jmp sum, MAR     

	; 455 cycles - 163 clock cycles faster than sum
	; 0x6A = 106 bytes total program - same as sum
	; jmp sum_unrolled, MAR     

	; 225 cycles - 393 clock cycles faster than sum
	; 0x70 = 112 bytes total program - 6 bytes more than sum
	jmp sum_unrolled_twice, MAR     
	push_return:


	; retrieve result from stack into registers
	; expected result is 
	pop X ; 00
	pop Y ; 00
	pop Z ; 00
	pop A ; 70

	halt



; 32 bit sum of 32 bit nums in stack
; manually unrolled loop version (only for sum operation)
sum_unrolled_twice {
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
	lw X, 0x8081, MAR
	lw Y, 0x8085, MAR
	; add nums
	adc X, Y
	; push result to stack
	push X

	; get nums
	lw X, 0x8082, MAR
	lw Y, 0x8086, MAR
	; add nums
	adc X, Y
	; push result to stack
	push X

	; get nums
	lw X, 0x8083, MAR
	lw Y, 0x8087, MAR
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


; 32 bit sum of 32 bit nums in stack
; manually unrolled loop version (only for sum operation)
; sum_unrolled {
; 	; where numbers get stored
; 	pusha 0x8080
;
; 	; read 8 bytes from the stack directly into 0x8080 (reads both 4 byte nums)
; 	mv Z, 8
;
; 	loop_1 {
; 		; get MAR value
; 		pop MAR
;
; 		; retrieve current byte
; 		pop A
; 		sw A, MAR 
; 		inc MAR
;
; 		; save MAR again
; 		push MAR
;
; 		; loop logic
; 		sub Z, 1
; 		lda MAR, loop_1
; 		jnz Z, MAR
; 	}
;
; 	; halt
;
; 	; want to pop the mar addr stored in the stack
; 	; can increment SP twice instead of popping useless value
; 	; pop MAR
; 	inc SP
; 	inc SP
;
; 	; want to store result to stack, so must save return addr elsewhere
; 	; get return address and save it to memory, next step changes stack
; 	pop	A
; 	sw A, 0xfff0, MAR
; 	inc MAR
; 	pop	A
; 	sw A, MAR ; sw A, 0xfff1, MAR
;
; 	; get nums
; 	lw X, 0x8080, MAR
; 	lw Y, 0x8084, MAR
; 	; add nums
; 	; NOTE: no carry on first add
; 	add X, Y
; 	; push result to stack
; 	push X
;
; 	; get nums
; 	lw X, 0x8081, MAR
; 	lw Y, 0x8085, MAR
; 	; add nums
; 	adc X, Y
; 	; push result to stack
; 	push X
;
; 	; get nums
; 	lw X, 0x8082, MAR
; 	lw Y, 0x8086, MAR
; 	; add nums
; 	adc X, Y
; 	; push result to stack
; 	push X
;
; 	; get nums
; 	lw X, 0x8083, MAR
; 	lw Y, 0x8087, MAR
; 	; add nums
; 	adc X, Y
; 	; push result to stack
; 	push X
;
; 	; get return address back
; 	lw A, 0xfff0, MAR
; 	; since next addr is right after can just increase MAR and load it into B
; 	; both approaches take the exact same # of clock cycles, but this approach is one less byte long
; 	inc MAR
; 	lw B, MAR ; lw B, 0xfff1, MAR
;
; 	; load into mar
; 	mv MarLo, A
; 	mv MarHi, B
;
; 	; return
; 	jmp MAR
; }
;
; ; 32 bit sum of 32 bit nums in stack
; sum {
; 	; where numbers get stored
; 	pusha 0x8080
;
; 	; read 8 bytes from the stack directly into 0x8080 (reads both 4 byte nums)
; 	mv Z, 8
;
; 	loop_1 {
; 		; get MAR value
; 		pop MAR
;
; 		; retrieve current byte
; 		pop A
; 		sw A, MAR 
; 		inc MAR
;
; 		; save MAR again
; 		push MAR
;
; 		; loop logic
; 		sub Z, 1
; 		lda MAR, loop_1
; 		jnz Z, MAR
; 	}
;
; 	; halt
;
; 	; want to pop the mar addr stored in the stack
; 	; can increment SP twice instead of popping useless value
; 	; pop MAR
; 	inc SP
; 	inc SP
;
; 	; num iterations (4 bytes)
; 	mv Z, 4
;
; 	; guarantee a clear of carry flag and push said flags
; 	mv A, 0
; 	cmp A, 0
; 	sw Flags, 0x9000, MAR
;
; 	; want to store result to stack, so must save return addr elsewhere
; 	; get return address and save it to memory, next step changes stack
; 	pop	A
; 	sw A, 0xfff0, MAR
; 	inc MAR
; 	pop	A
; 	sw A, MAR ; sw A, 0xfff1, MAR
;
; 	; now perform byte by byte ops
; 	loop_2 {
; 		; x = left num
; 		lda MAR, 0x8084
; 		; subtract current offset
; 		sub MarLo, Z
; 		lw X, MAR
;
; 		; y = right num
; 		lda MAR, 0x8088
; 		; subtract current offset
; 		sub MarLo, Z
; 		lw Y, MAR
;
;
; 		; get previous flags, add with carry, and save flags
; 		lw Flags, 0x9000, MAR
; 		adc X, Y
; 		; 0x9000 addr already stored at MAR here, no need to load it again
; 		sw Flags, MAR
;
; 		; push result to stack
; 		push X
;
; 		; loop logic
; 		sub Z, 1
; 		lda MAR, loop_2
; 		jnz Z, MAR
; 	}
;
; 	; get return address back
; 	lw A, 0xfff0, MAR
; 	; since next addr is right after can just increase MAR and load it into B
; 	; both approaches take the exact same # of clock cycles, but this approach is one less byte long
; 	inc MAR
; 	lw B, MAR ; lw B, 0xfff1, MAR
;
; 	; 63 | 49 FF F0   | lw A, 0xfff0, MAR
; 	; 66 | 4A FF F1   | lw B, 0xfff1, MAR
; 	; compared to
; 	; 63 | 49 FF F0   | lw A, 0xfff0, MAR
; 	; 66 | D5         | inc MAR
; 	; 67 | 2D         | lw B, MAR ; lw B, 0xfff1, MAR
;
;
; 	; load into mar
; 	mv MarLo, A
; 	mv MarHi, B
;
; 	; return
; 	jmp MAR
; }

; fib {
; 	; initial conditions
; 	mv X, 0
; 	mv Y, 1
;
; 	mv Z, 255
;
; 	lda MAR, loop
;
; 	loop {
; 		add X, Y
; 		add Y, X
;
; 		sub Z, 1
;
; 		jnz Z, MAR
; 	}
;
;
; 	; return routine
; 	pop MAR
; 	jmp MAR
; }
