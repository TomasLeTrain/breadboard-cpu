cool_label:
	jmp block_label || 2
	jmp global_scope_label, MAR
	; lol ; some other comment

global_scope_label:

block_label {
	local_scope_label:
	add A, Z
}

; expr_test some_const + (1 + 2 / 3 * 4 + (const2 / 5) + y6)
jmp ~0x5000 

block_label2 {
	char_test '\n'
	string_test "hello world!"
	; literal_test some, literal
	; label_test block_label
	; label_test2 cool_label
	; label_test3 inside_label
}

other_label {
	inside_label:
}

; this is a comment detailing another thing
