#once

#subruledef register
{
    a => 0b000
    b => 0b001
    x => 0b010
	y => 0b011
	z => 0b100
	mar_lo => 0b101
	mar_hi => 0b110
	flags => 0b111
}

#ruledef
{
    ; r0 = r1
    move {r0: register}, {r1: register} => 0b0000 @ 0b0 @ r0 @ 0b00000 @ r1
	; r0 = imm8
    move {r0: register}, {imm: i8} => 0b0000 @ 0b1 @ r0 @ imm

	; a = r0,b = r1, flg udpate
    cmp {r0: register}, {r1: register} => 0b0001 @ 0b0 @ r0 @ 0b00000 @ r1
	; a = r0,b = imm8, flg udpate
    cmp {r0: register}, {imm: i8} => 0b0001 @ 0b1 @ r0 @ imm
	
	; store reg0 = [MAR]
    store {r0: register} => 0b0010 @ 0b0 @ r0 @ imm
	; store reg0 = [imm16]
    store {r0: register}, {address: u16} => 0b0010 @ 0b1 @ r0 @ address

    push {r0: register} => 0b0011 @ 0b0 @ r0
    push {imm: i8}          => 0b00111 @ 0`3
    inc sp                  => 0b00111 @ 1`3
    inc mar                 => 0b00111 @ 2`3
    dec sp                  => 0b00111 @ 3`3
    lda mar, pc             => 0b00111 @ 4`3
    lda mar, sp             => 0b00111 @ 5`3
    lda mar, {address: u16} => 0b00111 @ 6`3 @ address
    lda sp,  {address: u16} => 0b00111 @ 7`3 @ address

    pop {r0: register} => 0b01000 @ r0
    lda sp, mar        => 0b01001 @ 0`3
    update_flags       => 0b01001 @ 1`3
    nop                => 0b01001 @ 2`3
    vram_read  z       => 0b01001 @ 3`3
    vram_write z       => 0b01001 @ 4`3
    vram_read  y       => 0b01001 @ 5`3
    vram_write y       => 0b01001 @ 6`3
    halt               => 0b01001 @ 7`3

	
	; jumping based on register
    jnz {r0: register} => 0b0101 @ 0b0 @ r0
	
	jmp => 0b01011 @ 0b000
	jc  => 0b01011 @ 0b001
	jeq => 0b01011 @ 0b010
	jnz => 0b01011 @ 0b011
	
	jmp {address: u16} => 0b01011 @ 0b100 @ address
	jc  {address: u16} => 0b01011 @ 0b101 @ address
	jeq {address: u16} => 0b01011 @ 0b110 @ address
	jnz {address: u16} => 0b01011 @ 0b111 @ address
	
	move {r0: register}, keyb => 0b0110 @ 0b0 @ register

    load {r0: register}                 => 0b0111 @ 0b0 @ r0
    load {r0: register}, {address: u16} => 0b0111 @ 0b1 @ r0 @ address

    sbc {r0: register}, {r1: register} => 0b1000 @ 0b0 @ r0 @ 0b00000 @ r1
    sbc {r0: register}, {imm: i8}      => 0b1000 @ 0b1 @ r0 @ imm

    sub {r0: register}, {r1: register} => 0b1001 @ 0b0 @ r0 @ 0b00000 @ r1
    sub {r0: register}, {imm: i8}      => 0b1001 @ 0b1 @ r0 @ imm

    adc {r0: register}, {r1: register} => 0b1010 @ 0b0 @ r0 @ 0b00000 @ r1
    adc {r0: register}, {imm: i8}      => 0b1010 @ 0b1 @ r0 @ imm

    add {r0: register}, {r1: register} => 0b1011 @ 0b0 @ r0 @ 0b00000 @ r1
    add {r0: register}, {imm: i8}      => 0b1011 @ 0b1 @ r0 @ imm

    not {r0: register}, {r1: register} => 0b1100 @ 0b0 @ r0 @ 0b00000 @ r1
    not {r0: register}                 => 0b1100 @ 0b1 @ r0

    xor {r0: register}, {r1: register} => 0b1101 @ 0b0 @ r0 @ 0b00000 @ r1
    xor {r0: register}, {imm: i8}      => 0b1101 @ 0b1 @ r0 @ imm

    or  {r0: register}, {r1: register} => 0b1110 @ 0b0 @ r0 @ 0b00000 @ r1
    or  {r0: register}, {imm: i8}      => 0b1110 @ 0b1 @ r0 @ imm

    and {r0: register}, {r1: register} => 0b1111 @ 0b0 @ r0 @ 0b00000 @ r1
    and {r0: register}, {imm: i8}      => 0b1111 @ 0b1 @ r0 @ imm
}
