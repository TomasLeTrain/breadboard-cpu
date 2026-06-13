#include <assert.h>
#include <inttypes.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>

#define MAX_ADDR_LEN 14
#define MAX_ADDR ((1 << MAX_ADDR_LEN) - 1)
#define NUM_REG 8
#define MAX_NUM_STEPS 8
#define NUM_INSTRUCTIONS 16

/*
clang-format off

notes:
mem write to rom area results in video write
ir2 doesn't need bout? (maybe wire flag bits to 4 msb to set flag on a math instruction with reg1 instr)
	- would require adding additional chip, probably not worth


registers:
- A			000: special register written to by math operations, gp register
- B			001: holds operand for math ops, gp register
- X			010: gp register
- Y			011: gp register
- Z			100: gp register
- MAR.LO	101: low bits of MAR addr register, could use as gp register
- MAR.HI	110: high bits of MAR addr register, could use as gp register
- FLAGS		111: holds flags after math ops, could be gp register


possible instruction additions:
keyboard into xxx (takes up half instruction)
halt (one instruction)
CMP xxx and yyy/imm8 - same as sub but does not save result (full instruction)
update flags register with alu values (one instruction)

// store word in current mar page (half instruction)
store word:	xxx ->mem[mar1|imm8]| [???? ? xxx] [imm8]			| STR xxx, imm8

// two instructions
JNZ:		PC <- MAR: ZRO FLG	| [???? ? ???]					| JNZ
JNZ:		PC <- imm16: ZRO FLG| [???? ? ???]					| JNZ imm16

// NOTE: not possible to implement instruction within 8 step limit (half instruction)
JNZ:		PC <- imm16: xxx!=0 | [0101 1 xxx]					| JNZ xxx, imm16


Instructions:
0000:
move word: 	xxx <- yyy			| [0000 0 xxx] [yyy.....]		| MV xxx, yyy
move word: 	xxx <- imm8			| [0000 1 xxx] [imm8    ]		| MV xxx, imm8

0001:
load word: 	xxx <- mem[mar]		| [0001 0 xxx]					| LOAD xxx
load word: 	xxx <- mem[imm16]	| [0001 1 xxx] [imm16][imm16]	| LOAD xxx, imm16

0010:
store word:	xxx -> mem[mar]		| [0010 0 xxx]					| STR xxx
store word:	xxx -> mem[imm16]	| [0010 1 xxx] [imm16][imm16]	| STR xxx, imm16

0011:
push:		xxx -> mem[SP],SP--	| [0011 0 xxx]					| PUSH xxx
push:		imm8 -> mem[SP],SP--| [0011 1 000]					| PUSH imm8 // overrides A reg

NOTE: not yet implemented:
SP INC:							| [0011 1 001]					| INC SP
MAR INC:						| [0011 1 010]					| INC MAR
SP DEC:							| [0011 1 011]					| DEC SP
MAR <- PC:						| [0011 1 100]					| LDA MAR, PC
MAR <- SP:						| [0011 1 101]					| LDA MAR, SP
MAR <- imm16:				 	| [0011 1 110][imm16][imm16]	| LDA MAR, imm16
unused 1:						| [0011 1 111]					|

0100:
pop:		xxx <- mem[SP],SP++	| [0100 0 xxx]					| POP xxx
NOTE: not yet implemented:
unused half:					| [0100 1 xxx]					|

0101:
JNZ:		PC <- MAR: xxx != 0	| [0101 0 xxx]					| JNZ xxx

JMP:		PC <- MAR			| [0101 1 000]					| JMP ; LDA PC, MAR
JC:			PC <- MAR: CRRY FLG	| [0101 1 001]					| JC
JEQ:		PC <- MAR: EQ FLG	| [0101 1 010]					| JEQ
NOTE: not yet implemented:
LDA SP:		SP <- MAR			| [0101 1 011]					| LDA SP, MAR

JMP:		PC <- imm16			| [0101 1 100][imm16][imm16]	| JMP imm16 ; LDA PC, imm16
JC:			PC <- imm16:CRRY FLG| [0101 1 101][imm16][imm16]	| JC  imm16
JEQ:		PC <- imm16: EQ FLG	| [0101 1 110][imm16][imm16]	| JEQ imm16
NOTE: not yet implemented:
LDA SP:		SP <- imm16			| [0101 1 111][imm16][imm16]	| LDA SP, imm16

0110:
???
0111:
???

1000:
add no carry:	xxx <- xxx + yyy	| [1000 0 xxx][yyy.....]	| ADD xxx, yyy 
add no carry:	xxx <- xxx + imm8	| [1000 1 xxx][imm8]		| ADD xxx, imm8

1001: 
sub no carry:	xxx <- xxx + yyy	| [1001 0 xxx][yyy.....]	| SUB xxx, yyy 
sub no carry:	xxx <- xxx + imm8	| [1001 1 xxx][imm8]		| SUB xxx, imm8

1010:
add flg crry:	xxx <- xxx + yyy	| [1010 0 xxx][yyy.....]	| ADC xxx, yyy 
add flg crry:	xxx <- xxx + imm8	| [1010 1 xxx][imm8]		| ADC xxx, imm8

1011:
sub flg crry:	xxx <- xxx - yyy	| [1011 0 xxx][yyy.....]	| SBC xxx, yyy 
sub flg crry:	xxx <- xxx - imm8	| [1011 1 xxx][imm8]		| SBC xxx, imm8

1100:
not:			xxx <- ~xxx			| [1100 0 xxx]				| NOT xxx
not:			xxx <- ~yyy			| [1100 1 xxx]				| NOT xxx, yyy

1101:
xor:			xxx <- xxx ^ yyy	| [1101 0 xxx][yyy.....]	| XOR xxx, yyy 
xor:			xxx <- xxx ^ imm8	| [1101 1 xxx][imm8]		| XOR xxx, imm8

1110:
or:				xxx <- xxx | yyy	| [1110 0 xxx][yyy.....]	| OR xxx, yyy 
or:				xxx <- xxx | imm8	| [1110 1 xxx][imm8]		| OR xxx, imm8

1111:
and:			xxx <- xxx & yyy	| [1111 0 xxx][yyy.....]	| AND xxx, yyy 
and:			xxx <- xxx & imm8	| [1111 1 xxx][imm8]		| AND xxx, imm8

clang-format on
 */

typedef uint16_t step_t;

// ucode[instruction][step][imm][reg0][reg1]
step_t ucode[NUM_INSTRUCTIONS][MAX_NUM_STEPS][2][NUM_REG][NUM_REG];

#define create_step(bus_out, addr_out, bus_write, other)                       \
  ((bus_out) | ((addr_out) << 4) | ((bus_write) << 6) | ((other) << 10))

typedef struct {
  // 3 bits wide
  uint8_t step;
  // 3 bits wide
  uint8_t reg0;
  // 1 bit wide
  uint8_t imm;
  // 4 bits wide
  uint8_t instruction;
  // 3 bits wide
  uint8_t reg1;
} split_addr_t;

typedef struct {
  step_t write;
  step_t bout;
} register_t;

typedef struct {
  step_t aout;
  register_t lo;
  register_t hi;
  step_t inc;
  step_t dec;
} addr_register_t;

// clang-format off
const step_t empty_instruction = create_step(0, 0, 0, 0);

const register_t A   = {.write = create_step(0, 0, 1, 0), .bout = create_step(1, 0, 0, 0)};
const register_t B   = {.write = create_step(0, 0, 2, 0), .bout = create_step(2, 0, 0, 0)};
const register_t X   = {.write = create_step(0, 0, 3, 0), .bout = create_step(3, 0, 0, 0)};
const register_t Y   = {.write = create_step(0, 0, 4, 0), .bout = create_step(4, 0, 0, 0)};
const register_t Z   = {.write = create_step(0, 0, 5, 0), .bout = create_step(5, 0, 0, 0)};
// also requires putting some addr register on the abus
const register_t MEM = {.write = create_step(0, 0, 8, 0),  .bout = create_step(8, 0, 0, 0)};
const register_t IR  = {.write = create_step(0, 0, 13, 0), .bout = empty_instruction};

const register_t F   = {.write = empty_instruction,        .bout = create_step(9, 0, 0, 0)};

const register_t IR2 = {.write = create_step(0, 0, 14, 0), .bout = empty_instruction};
const register_t FLAG= {.write = create_step(0, 0, 15, 0), .bout = create_step(10, 0, 0, 0)};

// const register_t dummy_reg = {.write = empty_instruction, .bout = empty_instruction};
const register_t dummy_reg0 = {.write = empty_instruction, .bout = empty_instruction};
const register_t dummy_reg1 = {.write = empty_instruction, .bout = empty_instruction};
// clang-format on

const addr_register_t MAR = {
    .aout = create_step(0, 2, 0, 0),
    .lo = {.write = create_step(0, 0, 6, 0), .bout = create_step(6, 2, 0, 0)},
    .hi = {.write = create_step(0, 0, 7, 0), .bout = create_step(7, 2, 0, 0)},
    .inc = create_step(0, 0, 0, 2),
    .dec = empty_instruction,
};

const addr_register_t PC = {
    .aout = create_step(0, 1, 0, 0),
    .lo = {.write = create_step(0, 0, 9, 0), .bout = create_step(6, 1, 0, 0)},
    .hi = {.write = create_step(0, 0, 10, 0), .bout = create_step(7, 1, 0, 0)},
    .inc = create_step(0, 0, 0, 1),
    .dec = empty_instruction,
};

// TODO: update
const step_t PC_FLAG_DIRECT = empty_instruction;
const step_t PC_FLAG_ZERO = create_step(0, 0, 0, 5);
const step_t PC_FLAG_EQ = create_step(0, 0, 0, 6);
const step_t PC_FLAG_CARRY = create_step(0, 0, 0, 7);
const step_t PC_FLAG_DUMMY = empty_instruction;

// writes values from alu into flag
// WARN: outputs into the data bus!
const step_t FLAG_WRITE_ALU = F.bout;

const addr_register_t SP = {
    .aout = create_step(0, 3, 0, 0),
    .lo = {.write = create_step(0, 0, 11, 0), .bout = create_step(6, 3, 0, 0)},
    .hi = {.write = create_step(0, 0, 12, 0), .bout = create_step(7, 3, 0, 0)},
    .inc = create_step(0, 0, 0, 3),
    .dec = create_step(0, 0, 0, 4),
};

const register_t *int_to_reg(uint8_t reg) {
  if (reg == 0)
    return &A;
  if (reg == 1)
    return &B;
  if (reg == 2)
    return &X;
  if (reg == 3)
    return &Y;
  if (reg == 4)
    return &Z;
  if (reg == 5)
    return &MAR.lo;
  if (reg == 6)
    return &MAR.hi;
  if (reg == 7)
    return &FLAG;
  assert(0 && "out of range register index");
}

step_t reg_write(uint8_t int_reg) {
  const register_t *reg = int_to_reg(int_reg);
  return reg->write;
}

step_t reg_bout(uint8_t int_reg) {
  const register_t *reg = int_to_reg(int_reg);
  return reg->bout;
}

// IR = [PC]
const step_t universal_step_0 = MEM.bout | PC.aout | IR.write;
// pc cnt
const step_t universal_step_1 = PC.inc;

// start steps for any instruction that loads an imm16
// WARN: MUST PERFORM PC CNT AFTER
const step_t load_address_procedure[5] = {
    universal_step_0,
    universal_step_1,
    MEM.bout | PC.aout | MAR.lo.write, // write first part of address to mar lo
    PC.inc,                            // pc cnt
    MEM.bout | PC.aout | MAR.hi.write, // write second part of address to mar hi
};

// TODO: determine?
const step_t reset = create_step(15, 0, 0, 0);
const step_t halt = create_step(14, 0, 0, 0);

const step_t error = 0xffff;

// math instructions
// const step_t ADD = create_step(0, 0, 0, 0b1000 | 0);
// const step_t ADC = create_step(0, 0, 0, 0b1000 | 1);
// const step_t SUB = create_step(0, 0, 0, 0b1000 | 2);
// const step_t SBC = create_step(0, 0, 0, 0b1000 | 3);
// const step_t NOT = create_step(0, 0, 0, 0b1000 | 4);
// const step_t XOR = create_step(0, 0, 0, 0b1000 | 5);
// const step_t OR = create_step(0, 0, 0, 0b1000 | 6);
// const step_t AND = create_step(0, 0, 0, 0b1000 | 7);
//
// math op is defined by 3 lsb's of instruction word, so no need to define math op
// const step_t MATH_OP = empty_instruction;

// clang-format off

// TODO: writting register to itself should be error or nop?
// reg0 = reg1
const step_t mw_template_reg[MAX_NUM_STEPS] = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | IR2.write, // need to load ir2 to figure out reg1
  	dummy_reg1.bout | dummy_reg0.write | PC.inc, // read from reg1 to reg0, pc cnt
	reset, reset, reset, reset
};

// reg = imm8
const step_t mw_template_imm[MAX_NUM_STEPS] = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | dummy_reg0.write, // write the immediate into reg0
	PC.inc, // pc cnt
	reset, reset, reset, reset
};

// reg = [MAR]
const step_t lw_template_mar[MAX_NUM_STEPS] = {
	universal_step_0,
	MEM.bout | MAR.aout | dummy_reg0.write | PC.inc, // read from addr MAR into register, pc cnt
	reset, reset, reset, reset, reset, reset
};

// reg = [imm16]
const step_t lw_template_imm[MAX_NUM_STEPS] = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	MEM.bout | MAR.aout | dummy_reg0.write |  PC.inc, // read from addr MAR into register, pc cnt
	reset, reset
};


// reg = [MAR]
const step_t sw_template_mar[MAX_NUM_STEPS] = {
	universal_step_0,
	MEM.write | MAR.aout | dummy_reg0.bout | PC.inc, // read from addr MAR into register, pc cnt
	reset, reset, reset, reset, reset, reset
};

// reg = [imm16]
const step_t sw_template_imm[MAX_NUM_STEPS] = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	MEM.write | MAR.aout | dummy_reg0.bout |  PC.inc, // read from addr MAR into register, pc cnt
	reset, reset
};


// [SP--] = reg
const step_t push_template_reg[MAX_NUM_STEPS] = {
	universal_step_0,
	MEM.write | SP.aout | dummy_reg0.bout | PC.inc, // read from reg into mem at SP addr, pc cnt
	SP.dec,
	reset, reset, reset, reset, reset
};

// [SP--] = imm8, overrides A reg
const step_t push_template_imm[MAX_NUM_STEPS] = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | A.write, // write into IR2
	MEM.write | SP.aout | A.bout | PC.inc, // read from IR2 into [SP], pc cnt
	SP.dec, // SP--
	reset, reset, reset
};

// reg0 = [SP++]
const step_t pop_template[MAX_NUM_STEPS] = {
	universal_step_0,
	MEM.bout | SP.aout | dummy_reg0.write | PC.inc, // write from [SP] into reg0, pc cnt
	SP.inc, // SP++
	reset, reset, reset, reset, reset
};

// MAR = imm16
const step_t lda_template[MAX_NUM_STEPS] = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	PC.inc, // pc cnt
	reset,reset
};

// JNZ reg -> PC = MAR if reg != 0 else NOP
const step_t jnz_template_reg[MAX_NUM_STEPS] = {
	universal_step_0,
	A.write | dummy_reg0.bout | PC.inc, // NOTE: pc cnt happens in case jump doesn't happens
	FLAG_WRITE_ALU, // write zero result to flag register
    MAR.lo.bout | PC.lo.write | PC_FLAG_ZERO,
    MAR.hi.bout | PC.hi.write | PC_FLAG_ZERO,
	reset, reset, reset
};

// can save instruction if A is already loaded
const step_t jnz_template_reg_A[MAX_NUM_STEPS] = {
	universal_step_0,
	FLAG_WRITE_ALU | PC.inc, // write zero result to flag register, pc cnt in case jump doesn't happens
    MAR.lo.bout | PC.lo.write | PC_FLAG_ZERO,
    MAR.hi.bout | PC.hi.write | PC_FLAG_ZERO,
	reset, reset, reset, reset
};

// jump if equal flag is carry flag is true
const step_t jmp_imm16_template[MAX_NUM_STEPS] = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	PC.inc, // NOTE: pc cnt happens in case jump doesn't happens
    MAR.lo.bout | PC.lo.write | PC_FLAG_DUMMY, // load from mar into pc if flag
    MAR.hi.bout | PC.hi.write | PC_FLAG_DUMMY, // load from mar into pc if flag
};

// jump if equal flag is true
const step_t jmp_mar_template[MAX_NUM_STEPS] = {
	universal_step_0,
	PC.inc, // NOTE: pc cnt in case jump doesn't happen
    MAR.lo.bout | PC.lo.write | PC_FLAG_DUMMY,
    MAR.hi.bout | PC.hi.write | PC_FLAG_DUMMY,
	reset, reset, reset, reset
};

// TODO: all math variants could have faster variants if reg0/reg1 are equal to a/b

// TODO: special case if reg0 = b, reg1 = a (impossible to swap registers without intermediate)

// reg0 = reg0 OP reg1
const step_t math_template_reg[MAX_NUM_STEPS] = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | IR2.write, // need to load ir2 to figure out reg1
	dummy_reg0.bout | A.write | PC.inc, // load reg0 into a
	dummy_reg1.bout | B.write, // load reg1 into b
	F.bout | dummy_reg0.write, // do math op, save to reg0, writes to flag reg
	reset, reset
};

// reg0 = reg0 OP reg1
const step_t math_template_imm[MAX_NUM_STEPS] = {
	universal_step_0,
	dummy_reg0.bout | A.write | PC.inc, // load reg0 into a first (in case reg0 = b), pc cnt
	MEM.bout | PC.aout | B.write, // load imm into b
	F.bout | dummy_reg0.write | PC.inc, // save F to reg0, writes to flag reg
	reset, reset, reset, reset
};

// reg0 = ~reg0
const step_t not_template_none[MAX_NUM_STEPS] = {
	universal_step_0,
	dummy_reg0.bout | A.write | PC.inc, // load reg0 into a, pc cnt
	F.bout | dummy_reg0.write, // do math op, save to reg0, writes to flag reg
	reset, reset, reset, reset
};

// reg0 = ~reg1
const step_t not_template_reg[MAX_NUM_STEPS] = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | IR2.write, // need to load ir2 to figure out reg1
	dummy_reg1.bout | A.write | PC.inc, // load reg0 into a
	F.bout | dummy_reg0.write, // do math op, save to reg1, writes to flag reg
	reset, reset, reset
};

// clang-format on

step_t *get_ucode_ptr(const split_addr_t *instruction) {
  return &ucode[instruction->instruction][instruction->step][instruction->imm]
               [instruction->reg0][instruction->reg1];
}

void mw_instruction(const split_addr_t *instruction) {
  step_t *curr = get_ucode_ptr(instruction);
  if (instruction->imm == 0) {
    *curr = mw_template_reg[instruction->step];

    if (instruction->step == 3) {
      *curr |= reg_bout(instruction->reg1);
      *curr |= reg_write(instruction->reg0);
    }
  } else {
    // have to populate reg1
    *curr = mw_template_imm[instruction->step];

    if (instruction->step == 2) {
      *curr |= reg_write(instruction->reg0);
    }
  }
}

void lw_instruction(const split_addr_t *instruction) {
  step_t *curr = get_ucode_ptr(instruction);

  if (instruction->imm == 0) {
    *curr = lw_template_mar[instruction->step];

    if (instruction->step == 1) {
      *curr |= reg_write(instruction->reg0);
    }
  } else {
    *curr = lw_template_imm[instruction->step];

    if (instruction->step == 5) {
      *curr |= reg_write(instruction->reg0);
    }
  }
}

void sw_instruction(const split_addr_t *instruction) {
  step_t *curr = get_ucode_ptr(instruction);
  if (instruction->imm == 0) {
    *curr = sw_template_mar[instruction->step];

    if (instruction->step == 1) {
      *curr |= reg_bout(instruction->reg0);
    }
  } else {
    *curr = sw_template_imm[instruction->step];

    if (instruction->step == 5) {
      *curr |= reg_bout(instruction->reg0);
    }
  }
}

void push_instruction(const split_addr_t *instruction) {
  step_t *curr = get_ucode_ptr(instruction);

  if (instruction->imm == 0) {
    *curr = push_template_reg[instruction->step];

    if (instruction->step == 1) {
      *curr |= reg_bout(instruction->reg0);
    }
  } else {
    *curr = push_template_imm[instruction->step];
  }
}

void pop_instruction(const split_addr_t *instruction) {
  if (instruction->imm == 1) {
    // TODO:
    // technically unused combination, could use as extra instruction
    // maybe make this the lda instruction instead with variations to free up
    // one
  }

  // reg1 does not matter in any case
  step_t *curr = get_ucode_ptr(instruction);

  *curr = pop_template[instruction->step];

  if (instruction->step == 1) {
    *curr |= reg_bout(instruction->reg0);
  }
}

// lda, imm = 0, reg0[2] = 0, reg0 = x -> load imm16 x (mar/pc/sp)
// lda, imm = 0, reg0[2] = 1, reg0 = x -> increase x (mar/pc/sp)
// lda, imm = 1, reg0[2] = 0, reg0 = x -> decrease x (mar/pc/sp)
// lda, imm = 1, reg0[2] = 1, reg0 = x, reg1 = y -> load x into y (uses imm8)

// TODO: any combination of imm and reg0 (4 bits) is a valid different
// instruction, could utilize for math?
// could utilize to load direct to SP or PC in one instruction (instruction +
// imm16)
void lda_instruction(const split_addr_t *instruction) {
  if (instruction->imm == 1) {
    // TODO:
    // technically unused combination, could use as extra instruction
  }
  // reg0 and reg1 don't matter

  step_t *curr = get_ucode_ptr(instruction);

  *curr = lda_template[instruction->step];
}

void jmp_instruction(const split_addr_t *instruction) {
  // reg1 does not matter in any case
  step_t *curr = get_ucode_ptr(instruction);
  // jump if reg != 0
  if (instruction->imm == 0) {
    // special case -> can save step if already A
    if (int_to_reg(instruction->reg0) == &A) {
      *curr = jnz_template_reg_A[instruction->step];
    } else {
      // general case -> writing to A from reg0
      *curr = jnz_template_reg[instruction->step];
      if (instruction->step == 1) {
        *curr |= reg_bout(instruction->reg0);
      }
    }
  } else {
    // TODO: if error states get added, 0b11 state should become invalid?
    // imm = 1
    // we can ignore the imm bit and create a new bit layout for the 3 reg bits:
    //
    // 1st and 2nd bit determine type of jump:
    // 0b00 -> unconditional jump
    // 0b01 -> if carry jump
    // 0b10 -> if equal jump
    // 0b11 -> ununsed, defaults to unconditional jump
    // 3rd bit: 0 = MAR address, 1 = imm16 address
    //
    // here we can use the reg bits to determine what type of jump

    uint8_t flag_idx = instruction->reg0 & 0b011;
    uint8_t using_imm16_flag = instruction->reg0 & 0b100;
    const static step_t idx_to_flag[] = {PC_FLAG_DIRECT, PC_FLAG_CARRY,
                                         PC_FLAG_EQ, PC_FLAG_DIRECT};
    step_t flag_step_bits = idx_to_flag[flag_idx];

    // TODO: unconditional jump could save one step by skipping pc cnt
    if (using_imm16_flag) {
      // imm16 address
      *curr = jmp_imm16_template[instruction->step];
      if (instruction->step == 6 || instruction->step == 7) {
        *curr |= flag_step_bits;
      }
    } else {
      // using mar address
      *curr = jmp_mar_template[instruction->step];
      if (instruction->step == 2 || instruction->step == 3) {
        *curr |= flag_step_bits;
      }
    }
  }
}

void add_instruction(const split_addr_t *instruction) {
  step_t *curr = get_ucode_ptr(instruction);
  if (instruction->imm == 0) {
    *curr = math_template_reg[instruction->step];

    // TODO: determine
    if (instruction->step == 10000) {
      *curr |= reg_bout(instruction->reg1);
      *curr |= reg_write(instruction->reg0);
    }
  } else {
    *curr = mw_template_imm[instruction->step];

    // TODO: determine
    if (instruction->step == 10000) {
      *curr |= reg_write(instruction->reg0);
    }
  }
}

typedef void (*instruction_func)(const split_addr_t *);

instruction_func instructions_table[16] = {
    mw_instruction,           lw_instruction,
    sw_instruction,           push_instruction,
    pop_instruction,          lda_instruction,
    jmp_instruction,          (instruction_func)(NULL),
    (instruction_func)(NULL), (instruction_func)(NULL),
    (instruction_func)(NULL), (instruction_func)(NULL),
    (instruction_func)(NULL), (instruction_func)(NULL),
    (instruction_func)(NULL), (instruction_func)(NULL),
};

void addr_to_instruction(uint32_t addr, split_addr_t *instruction_ptr) {
  // clang-format off
  //                              |r1|   ir   |stp|
  instruction_ptr->step         = (addr & 0b00000000000111);
  instruction_ptr->reg0         = (addr & 0b00000000111000) >> 3;
  instruction_ptr->imm          = (addr & 0b00000001000000) >> 6;
  instruction_ptr->instruction  = (addr & 0b00011110000000) >> 7;
  instruction_ptr->reg1         = (addr & 0b11100000000000) >> 11;
  // clang-format on
}

void process_address(int addr) {
  split_addr_t instruction;
  addr_to_instruction(addr, &instruction);

  // move word
  instruction_func istr_func = instructions_table[instruction.instruction];
  if (istr_func != (instruction_func)NULL) {
    istr_func(&instruction);
  } else {
    step_t *curr = get_ucode_ptr(&instruction);
    *curr = error;
  }
}

void populate_ucode() {
  for (int addr = 0; addr <= MAX_ADDR; addr++) {
    process_address(addr);
  }
}

step_t getInstruction(int addr) {
  split_addr_t instruction;
  addr_to_instruction(addr, &instruction);
  return *get_ucode_ptr(&instruction);
}

void write_ucode_logism() {
  printf("v3.0 hex words plain\n");
  for (int addr = 0; addr <= MAX_ADDR; addr++) {
    printf("%04X", getInstruction(addr));
    if (addr % 16 == 15) {
      printf("\n");
    } else {
      printf(" ");
    }
  }
}

int main() {
  populate_ucode();
  write_ucode_logism();
}
