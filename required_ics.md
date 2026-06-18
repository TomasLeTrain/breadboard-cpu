# 74HC00 - NAND gate
1: control logic
1: x/y select logic
1: memory logic
= 3

# 74HC04 - NOT gate
1: memory logic
1: alu logic
2: control logic (both schmitt?)
= 4

# 74HC08 - AND gate
2: alu logic
= 2

# 74HC32 - OR gate
2: alu logic
1: memory logic
= 3

# 74HC299 - universal shift registers
3: x/y/z
= 3

# 74HC377 - reg with clk en
2: IR and IR2
2: A and B
1: flag reg
= 5

# 74HC273 - reg with async reset
2: rom latch in
= 2

# 74HC181 - alu ic
= 2

# 74HC138 - 3 to 8 decoder (inverted outputs)
5: control logic
= 5

# 74HC139 - dual 2 to 4 decoder (inverted outputs)
1: control logic
= 1

# 74HC161 - 4 bit counter with async clear
4: PC counter
4: MAR counter
1: control logic

= 9


# 74HC151 - 8 to 1 mux for flag reg
= 1
# 74HC157 - quad 2-line to 1 selector for flag
= 1

# 74HC191 - 4 bit up/down counter
4: SP counter

= 4

# 74HC245 - octal buffer
1: a reg
1: flag reg
1: b reg
1: f reg

2: PC counter
2: MAR counter
2: SP counter
2: ABUS -> BBUS
= 12


# CY7C199 - ram chip
= 1

# SST39SF010A - flash rom chip
2: control logic
1: memory

= 3
