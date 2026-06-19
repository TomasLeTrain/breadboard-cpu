min of 1 space between chips
63 space per breadboard

# Left (rotated)

# left ucode rom section, clock
- 16: flash rom
- 1:  space
- 8:  step counter
- 1:  space

- 7:  nand ic
- 1:  space

- 7:  hex inverter
- 1:  space

- 20: clock 

=    62 (1 left)

# left ucode rom decoders
- 10: ucode rom latch
- 1:  space
- 8:  write 1 decoder
- 1:  space
- 8:  read 1 decoder
- 1:  space
- 8:  aout decoder
- 1:  space
- 8:  other decoder

- 1:  space
- 7:  hex inverter
- 1:  space

- 10:  rom bar led

=    66 (-3 left, if ic's are small profile could fit bar led)

# right ucode rom section, xy regs
- 16: flash rom
- 1:  space
- 10: IR reg
- 1:  space
- 10: IR2 reg

- 1:  space
- 10: Y reg
- 1:  space
- 10: Z reg

=    60 (3 left)

# right ucode decoders, z reg
- 10: ucode rom latch
- 1:  space

- 8:  write decoder 0
- 1:  space

- 8:  read decoder 0
- 1:  space

- 7:  xy selector or-gate ic
- 1:  space
- 10: z reg
- 1:  space

- 10:  rom bar led

=   58 (5 left, could maybe fit another ic)

## a register - 1 right
- 10: octal bus buffer
- 1:  space
- 10: register
- 1:  space
- 7: zero or-gate
- 1:  space
- 7: zero or-gate
- 1:  space
- 7: and gate
- 1:  space

- 8: shift register - keyboard

= 54 (9 left, space for ic or maybe a bar led)

## alu - 2 right
- 8:  octal bus buffer
- 1:  space
- 12: alu
- 1:  space
- 12: alu
- 1:  space
- 8:  math decoder
- 1:  space
- 7:  decoder or-gate
- 1:  space
- 7:  inverter
=     59 (4 left)

## b register - 3 right
- 10: octal bus buffer
- 1:  space
- 10: register
- 1:  space
- 8:  quad 2-1 selector (flags)
- 1:  space
- 8:  flag register
- 1:  space
- 10: flag bus buffer
- 1:  space
- 8:  8-1 selector
=    59 (4 left)

# pc register
- 8:  4 bit counter
- 1:  space
- 8:  4 bit counter
- 1:  space

- 10: octal bus buffer
- 1:  space
- 10: octal bus buffer
- 1:  space

- 8:  4 bit counter
- 1:  space
- 8:  4 bit counter
- 1:  space

=    57 (6 left)

# mar register
- 8:  4 bit counter
- 1:  space
- 8:  4 bit counter
- 1:  space

- 10: octal bus buffer
- 1:  space
- 10: octal bus buffer
- 1:  space

- 8:  4 bit counter
- 1:  space
- 8:  4 bit counter

=    57 (6 left)

# sp register
- 8:  4 bit up/down counter
- 1:  space
- 8:  4 bit up/down counter
- 1:  space

- 10: octal bus buffer
- 10: octal bus buffer (no space needed with some ic's)
- 1:  space

- 8:  4 bit up/down counter
- 1:  space
- 8:  4 bit up/down counter

- 7:  ??? gate

=    63 (0 left)


# memory stuff
- 14: ram
- 1:  space

- 16: flash eeprom
- 1:  space


- 10: octal bus buffer
- 1:  space
- 10: octal bus buffer

enough space for one gate ic or bar led

=    53 (10 left)
