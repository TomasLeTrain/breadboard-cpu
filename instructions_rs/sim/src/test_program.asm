start:
	; initialize stack
	lda SP, 0xffe0 ; lda sp, sp_start_addr

	; push the left and right nums
	; right first
	push 0
	push 0
	push 0
	push 50

	; push the left and right nums
	; left second
	push 0
	push 0
	push 0
	push 20

	; call fib
	pusha push_return
	; jmp fib, MAR     ; jump to func
	jmp sum, MAR     ; jump to func
	push_return:

	; retrieve result from stack into registers
	pop X
	pop Y
	pop Z
	pop A

	halt



; 32 bit sum of 32 bit nums in stack
sum {
	; where numbers get stored
	pusha 0x8080

	; read 8 bytes from the stack directly into 0x8080 (reads both 4 byte nums)
	mv Z, 8

	loop_1 {
		; get MAR value
		pop MAR

		; retrieve current byte
		pop A
		sw A, MAR 
		inc MAR
		
		; save MAR again
		push MAR

		; loop logic
		sub Z, 1
		lda MAR, loop_1
		jnz Z, MAR
	}

	; pop current value since its not used anymore
	pop MAR

	; num iterations (4 bytes)
	mv Z, 4

	; guarantee a clear of carry flag
	mv A, 0
	add A, 0

	; for l
	pusha 0x8080
	; for r
	pusha 0x8080 + 4


	; now perform byte by byte ops
	loop_2 {
		; x = left num

		; get current MAR
		pop MAR 
		sw X, MAR
		inc MAR
		; push again
		push MAR

		; y = right num

		; get current MAR
		pop MAR 
		sw Y, MAR
		inc MAR
		; push again
		push MAR

		; add with carry both
		adc X, Y

		; push result to stack
		push X

		; loop logic
		sub Z, 1
		lda MAR, loop_2
		jnz Z, MAR
	}

	; return routine
	pop MAR
	jmp MAR
}

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
