registers:
- A			000: special register written to by math operations, gp register
- B			001: holds operand for math ops, gp register
- X			010: gp register
- Y			011: gp register
- Z			100: gp register
- MAR.LO	101: low bits of MAR addr register, could use as gp register
- MAR.HI	110: high bits of MAR addr register, could use as gp register
- FLAGS		111: holds 4 bits of flags after math ops (or whatever gets written to it)


www - all possible registers (can r/w):
- A (constantly overriden)
- B (constantly overriden)
- X
- Y
- Z
- MAR.LO (constantly overriden)
- MAR.HI (constantly overriden)
- PC.LO (constantly overriden)
- PC.HI (constantly overriden)
- SP.LO
- SP.HI
- FLAGS (can only r)
- KEYB (can only r)

move register to register - 11 instructions:
mv A, B
mv A, X
mv A, Y
mv A, Z
mv A, MAR.LO
mv A, MAR.HI
mv A, PC.LO
mv A, PC.HI
mv A, SP.LO
mv A, SP.HI
mv A, FLAGS
mv A, KEYB
120 = 12 * 10 (FLAGS and KEYB is read only) individual instructions - fits in 7 bits (128 size)

move imm8 to register - 11 instructions:
mv A, imm8
mv B, imm8
mv X, imm8
mv Y, imm8
mv Z, imm8
mv MAR.LO, imm8
mv MAR.HI, imm8
mv PC.LO, imm8
mv PC.HI, imm8
mv SP.LO, imm8
mv SP.HI, imm8
11 = 11 individual instructions

load xxx <- mem[imm16]:
load A, imm16
load B, imm16
load X, imm16
load Y, imm16
load Z, imm16
load MAR.LO, imm16
load MAR.HI, imm16
load PC.LO, imm16
load PC.HI, imm16
load SP.LO, imm16
load SP.HI, imm16
22 = 11 * 2(vram variant) individual instructions

load xxx <- mem[mar]:
load A
load B
load X
load Y
load Z
load MAR.LO
load MAR.HI
load PC.LO
load PC.HI
load SP.LO
load SP.HI
22 = 11 * 2(vram variant) instructions

store xxx -> mem[imm16]:
store A, imm16
store B, imm16
store X, imm16
store Y, imm16
store Z, imm16
store MAR.LO, imm16
store MAR.HI, imm16
store PC.LO, imm16
store PC.HI, imm16
store SP.LO, imm16
store SP.HI, imm16
store FLAGS, imm16
store KEYB, imm16
26 = 13 * 2(vram variant) individual instructions

store xxx -> mem[mar]:
store A
store B
store X
store Y
store Z
store MAR.LO
store MAR.HI
store PC.LO
store PC.HI
store SP.LO
store SP.HI
store FLAGS
store KEYB
26 = 13 * 2(vram variant) instructions

push xxx -> mem[SP],SP--:
push A
push B
push X
push Y
push Z
push MAR.LO
push MAR.HI
push PC.LO
push PC.HI
push SP.LO
push SP.HI
push FLAGS
push KEYB
13 = 13 instructions

pop xxx <- mem[SP],SP++:
pop A
pop B
pop X
pop Y
pop Z
pop MAR.LO
pop MAR.HI
pop PC.LO
pop PC.HI
pop SP.LO
pop SP.HI
11 = 11 instructions

jnz PC <- MAR: xxx != 0:
jnz A
jnz B
jnz X
jnz Y
jnz Z
jnz MAR.LO
jnz MAR.HI
jnz PC.LO
jnz PC.HI
jnz SP.LO
jnz SP.HI
jnz FLAGS
jnz KEYB
13 = 13 instructions


misc. single:
push imm8
INC SP
INC MAR
DEC SP
LDA MAR, PC
LDA MAR, SP
LDA SP, MAR
SET FLAG
NOP
HALT
10 = 10 instructions

jump single:
JMP
JC
JEQ
JNZ
J?
J?
J?
J?
8 = 8 instructions

lda imm16:
LDA MAR, imm16
LDA SP, imm16
2 = 2 instructions

jmp imm16:
JMP imm16
JC  imm16
JEQ imm16
JNZ imm16
J? imm16
J? imm16
J? imm16
J? imm16
8 = 8 instructions

cmp:
cmp X, Y
cmp X, Z
cmp X, MAR.LO
cmp X, MAR.HI
cmp X, SP.LO
cmp X, SP.HI
cmp X, FLAGS
cmp X, KEYB
100 = 10 * 10 (a and b overriden) = individual instructions - fits in 7 bits (128 size)

cmp with imm8:
cmp X, imm8
cmp Y, imm8
cmp Z, imm8
cmp MAR.LO, imm8
cmp MAR.HI, imm8
cmp SP.LO, imm8
cmp SP.HI, imm8
cmp FLAGS, imm8
cmp KEYB, imm8
22 = 11 * 2 (reversed op order) instructions


math (no not):
math A, A
math A, B
math A, X
math A, Y
math A, Z
math A, MAR.LO
math A, MAR.HI
math A, SP.LO
math A, SP.HI
math A, FLAGS
math A, KEYB
692 = 693 - 1(math B,A is impossible) = 7(math ops) * 11(first op) * 9(sec op; flag and keyb read only) individual instructions

math imm8 (no not):
math A, imm8
math B, imm8
math X, imm8
math Y, imm8
math Z, imm8
math MAR.LO, imm8
math MAR.HI, imm8
math SP.LO, imm8
math SP.HI, imm8
126 = 7(math ops) * 2(order) * 9(reg) individual instructions

not:
not A
not B
not X
not Y
not Z
not MAR.LO
not MAR.HI
not SP.LO
not SP.HI
9 = 9 individual instructions

not:
not A, B
not A, X
not A, Y
not A, Z
not A, MAR.LO
not A, MAR.HI
not A, SP.LO
not A, SP.HI
not A, FLAGS
not A, KEYB
80 = 10(first op) * 8(sec op; flag and keyb read only) individual instructions



= 1350 total instructions
