; jmp ~cool_label + 5, SP
; this is a comment detailing another thing

cool_label:
	jmp global_scope_label, MAR
	whar 3
	; 1 3
global_scope_label:

block_label {
	local_scope_label:
	add A, Z
}

;  	expr_test some_const + (1 + 2 / 3 * 4 + (const2 / 5) + y6)
; 	jmp ~0x5000 
;
; block_label {
; 	char_test '\n'
; 	string_test "hello world!"
; 	literal_test some, literal
; 	label_test block_label
; 	label_test2 cool_label
; 	label_test3 inside_label
; }
;
; other_label {
; 	inside_label:
; }

; this is a comment detailing another thing
