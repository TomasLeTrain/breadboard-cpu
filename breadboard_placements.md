min of 1 space between chips
63 space per breadboard

# Right

## a register - 0 right
- 10: octal bus buffer
- 1:  space
- 10: register
- 1:  space
- 7: zero or-gate
- 1:  space
- 7: zero or-gate
=    37 (26 left)

## alu - 1 right
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

## b register - 2 right
- 10: octal bus buffer
- 1:  space
- 10: register
=    21 (42 left)

## control related - 3 right
- 10: octal bus buffer
- 1:  space
- 10: register
=    21 (42 left)

## xyz registers - 4 right
- 10: octal bus buffer
- 1:  space
- 10: register
=    21 (42 left)
