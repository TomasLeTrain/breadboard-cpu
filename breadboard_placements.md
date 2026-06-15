min of 1 space between chips
63 space per breadboard

# Left (rotated)
## xyz registers - ? left
- 10: x register
- 1:  space
- 10: y register
- 1:  space
- 10: z register
=    32 (31 left)

## clock
???

# Right

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
// enough space to house keyboard shift register?
=    45 (18 left)

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
