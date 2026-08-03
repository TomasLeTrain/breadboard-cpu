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

; instruction set definitions
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
	
    ; TODO: add restriction about mar_lo and mar_hi not being implemented with this instruction
	; store [MAR] = reg0
    store {r0: register} => 0b0010 @ 0b0 @ r0 @ imm
	; store [imm16] = reg0
    store {r0: register}, {address: u16} => 0b0010 @ 0b1 @ r0 @ address

    ; TODO: add restriction about mar_lo and mar_hi not being implemented with this instruction
    push {r0: register} => 0b0011 @ 0b0 @ r0
    ; TODO: overwrites a!!
    push {imm: i8}          => 0b00111 @ 0`3 @ imm
    inc sp                  => 0b00111 @ 1`3
    inc mar                 => 0b00111 @ 2`3
    dec sp                  => 0b00111 @ 3`3
    lda mar, pc             => 0b00111 @ 4`3
    lda mar, sp             => 0b00111 @ 5`3
    lda mar, {address: u16} => 0b00111 @ 6`3 @ address
    lda sp,  {address: u16} => 0b00111 @ 7`3 @ address

    ; TODO: add restriction about mar_lo and mar_hi not being implemented with this instruction
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
	
    ; jump to mar address
	jmp => 0b01011 @ 0b000
	jnc => 0b01011 @ 0b001
	jeq => 0b01011 @ 0b010
	jnz => 0b01011 @ 0b011

	jmp mar => asm {jmp}
	jnc mar => asm {jnc}
	jeq mar => asm {jeq}
	jnz mar => asm {jnz}
	
    ; jump to imm16
	jmp {address: u16} => 0b01011 @ 0b100 @ address
	jnc {address: u16} => 0b01011 @ 0b101 @ address
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

; some constants

; defines where sp gest
sp_save_addr = 0xfff0
sp_start_addr = 0xffe0

#fn high_byte(value) => ((value & 0xff00) >> 8)`8
#fn low_byte(value)  => (value & 0x00ff)`8

; macros and other defs
#ruledef
{
	; pushes address to jump to after the procedure is done
    call {address: u16} => asm {
        push high_byte(call_end)
        push low_byte(call_end)
        jmp {address}
		call_end:
    }

	; retrieves address from stack and jumps there
    return => asm {
        pop a
        move mar_lo, a
        pop a
        move mar_hi, a
        jmp mar
    }

    init_sp => asm {
        lda sp, sp_start_addr
    }

	; store the current location of sp into sp_save_addr
    save_sp_addr => asm {
        lda mar, sp
        move a, mar_hi
        move b, mar_lo
        store a, sp_save_addr   ; mar_hi
        store b, sp_save_addr+1 ; mar_lo
    }

    restore_sp_addr => asm {
        load a, sp_save_addr   ; mar_hi
        load b, sp_save_addr+1 ; mar_lo
        move mar_hi, a
        move mar_lo, b
        lda sp, mar
    }
}
