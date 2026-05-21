#include <inttypes.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>

#define MAX_ADDR_LEN 14
#define MAX_ADDR ((1 << MAX_ADDR_LEN) - 1)
#define NUM_REG 8
#define MAX_NUM_STEPS 8
#define NUM_INSTRUCTIONS 16 

// ucode[instruction][step][imm][reg0][reg1]
uint16_t ucode[NUM_INSTRUCTIONS][MAX_NUM_STEPS][2][NUM_REG][NUM_REG];

// int slice_num(int n, int lo, int hi){
// 	int low_bits_mask = (1 << lo) - 1;
// 	int high_bits_mask = ~((1 << hi) - 1);
// 	return n & low_bits_mask & high_bits_mask;
// }

#define create_step(bus_out, addr_out, bus_write, other)                       \
  (bus_out | (addr_out << 4) | (bus_write << 6) | (other << 10))

typedef uint16_t step_t;

typedef struct {
  step_t write;
  step_t bout;
} register_t;

typedef struct {
  step_t aout;
  register_t lo;
  register_t hi;
  // uint16_t write_lo;
  // uint16_t write_hi;
  // uint16_t bout_lo;
  // uint16_t bout_hi;
  step_t inc;
  step_t dec;
} addr_register_t;

// clang-format off
const register_t A   = {.write = create_step(0, 0, 1, 0), .bout = create_step(1, 0, 0, 0)};
const register_t B   = {.write = create_step(0, 0, 2, 0), .bout = create_step(2, 0, 0, 0)};
const register_t X   = {.write = create_step(0, 0, 3, 0), .bout = create_step(3, 0, 0, 0)};
const register_t Y   = {.write = create_step(0, 0, 4, 0), .bout = create_step(4, 0, 0, 0)};
const register_t Z   = {.write = create_step(0, 0, 5, 0), .bout = create_step(5, 0, 0, 0)};
// also requires putting some addr register on the abus
const register_t MEM = {.write = create_step(0, 0, 8, 0), .bout = create_step(8, 0, 0, 0)};
const register_t IR  = {.write = create_step(0, 0, 13, 0), .bout = create_step(0, 0, 0, 0)};
const register_t IR2 = {.write = create_step(0, 0, 14, 0), .bout = create_step(11, 0, 0, 0)};
const register_t FLAG= {.write = create_step(0, 0, 0, 0), .bout = create_step(10, 0, 0, 0)};

const register_t dummy_reg = {.write = create_step(0, 0, 0, 0), .bout = create_step(0, 0, 0, 0)};
// clang-format on

const addr_register_t MAR = {
    .aout = create_step(0, 2, 0, 0),
    .lo = {.write = create_step(0, 0, 6, 0), .bout = create_step(6, 2, 0, 0)},
    .hi = {.write = create_step(0, 0, 7, 0), .bout = create_step(7, 2, 0, 0)},
    // .write_lo = create_step(0, 0, 6, 0),
    // .write_hi = create_step(0, 0, 7, 0),
    // .bout_lo = create_step(6, 2, 0, 0), // need to out on abus for bout
    // .bout_hi = create_step(7, 2, 0, 0), // need to out on abus for bout
    .inc = create_step(0, 0, 0, 2),
    .dec = create_step(0, 0, 0, 0),
};

const addr_register_t PC = {
    .aout = create_step(0, 1, 0, 0),
    .lo = {.write = create_step(0, 0, 9, 0), .bout = create_step(6, 1, 0, 0)},
    .hi = {.write = create_step(0, 0, 10, 0), .bout = create_step(7, 1, 0, 0)},
    // .write_lo = create_step(0, 0, 9, 0),
    // .write_hi = create_step(0, 0, 10, 0),
    // .bout_lo = create_step(6, 1, 0, 0), // need to out on abus for bout
    // .bout_hi = create_step(7, 1, 0, 0), // need to out on abus for bout
    .inc = create_step(0, 0, 0, 1),
    .dec = create_step(0, 0, 0, 0),
};

const step_t PC_FLAG_DIRECT = create_step(0, 0, 0, 0);
const step_t PC_FLAG_ZERO = create_step(0, 0, 0, 5);
const step_t PC_FLAG_EQ = create_step(0, 0, 0, 6);
const step_t PC_FLAG_CARRY = create_step(0, 0, 0, 7);

const addr_register_t SP = {
    .aout = create_step(0, 3, 0, 0),
    .lo = {.write = create_step(0, 0, 11, 0), .bout = create_step(6, 3, 0, 0)},
    .hi = {.write = create_step(0, 0, 12, 0), .bout = create_step(7, 3, 0, 0)},
    // .write_lo = create_step(0, 0, 11, 0),
    // .write_hi = create_step(0, 0, 12, 0),
    // .bout_lo = create_step(6, 3, 0, 0), // need to out on abus for bout
    // .bout_hi = create_step(7, 3, 0, 0), // need to out on abus for bout
    .inc = create_step(0, 0, 0, 3),
    .dec = create_step(0, 0, 0, 4),
};

const register_t *int_to_reg(int reg) {
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
  return NULL;
}

int reg_write(int int_reg) {
  const register_t *reg = int_to_reg(int_reg);
  if (reg != NULL) {
    return reg->write;
  }
  return 0;
}

int reg_bout(int int_reg) {
  const register_t *reg = int_to_reg(int_reg);
  if (reg != NULL) {
    return reg->bout;
  }
  return 0;
}

// IR = [PC]
const uint16_t universal_step_0 = MEM.bout | PC.aout | IR.write;
// pc cnt
const uint16_t universal_step_1 = PC.inc;

// start steps for any instruction that loads an imm16
// WARN: MUST PERFORM PC CNT AFTER
const uint16_t load_address_procedure[5] = {
    universal_step_0,
    universal_step_1,
    MEM.bout | PC.aout | MAR.lo.write, // write first part of address to mar lo
    PC.inc,                            // pc cnt
    MEM.bout | PC.aout | MAR.hi.write, // write second part of address to mar hi
};

// TODO: determine?
const uint16_t reset = create_step(15, 0, 0, 0);
const uint16_t halt = create_step(14, 0, 0, 0);

// clang-format off

// reg = reg1
const uint16_t mw_template_reg[MAX_NUM_STEPS] = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | IR2.write, // need to load ir2 to figure out reg1
  	dummy_reg.bout | dummy_reg.write | PC.inc, // read from reg1 to reg0, pc cnt
	reset, reset, reset, reset
};

// reg = imm8
const uint16_t mw_template_imm[MAX_NUM_STEPS] = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | dummy_reg.write, // write the immediate into reg0
	PC.inc, // pc cnt
	reset, reset, reset, reset
};

// reg = [MAR]
const uint16_t lw_template_mar[MAX_NUM_STEPS] = {
	universal_step_0,
	MEM.bout | MAR.aout | dummy_reg.write, PC.inc, // read from addr MAR into register, pc cnt
	reset, reset, reset, reset, reset
};

// reg = [imm16]
const uint16_t lw_template_imm[MAX_NUM_STEPS] = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	MEM.bout | MAR.aout | dummy_reg.write |  PC.inc, // read from addr MAR into register, pc cnt
	reset, reset
};


// reg = [MAR]
const uint16_t sw_template_mar[MAX_NUM_STEPS] = {
	universal_step_0,
	MEM.write | MAR.aout | dummy_reg.bout | PC.inc, // read from addr MAR into register, pc cnt
	reset, reset, reset, reset, reset, reset
};

// reg = [imm16]
const uint16_t sw_template_imm[MAX_NUM_STEPS] = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	MEM.write | MAR.aout | dummy_reg.bout |  PC.inc, // read from addr MAR into register, pc cnt
	reset, reset
};


// [SP--] = reg
const uint16_t push_template_reg[MAX_NUM_STEPS] = {
	universal_step_0,
	MEM.write | SP.aout | dummy_reg.bout | PC.inc, // read from reg into mem at SP addr, pc cnt
	SP.dec,
	reset, reset, reset, reset, reset
};

// [SP--] = imm8
const uint16_t push_template_imm[MAX_NUM_STEPS] = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | IR2.write, // write into IR2
	MEM.write | SP.aout | IR2.bout | PC.inc, // read from IR2 into [SP], pc cnt
	SP.dec, // SP--
	reset, reset, reset
};

// [SP++] = imm8
const uint16_t pop_template[MAX_NUM_STEPS] = {
	universal_step_0,
	MEM.bout | SP.aout | dummy_reg.write | PC.inc, // write from [SP] into reg0, pc cnt
	SP.inc, // SP++
	reset, reset, reset, reset, reset
};

// MAR = imm16
const uint16_t lda_template[MAX_NUM_STEPS] = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	PC.inc, // pc cnt
	reset,reset
};

// JNZ reg -> PC = MAR if reg != 0 else NOP
// WARN: should check if dummy reg is A!!!
const uint16_t jnz_template_reg[MAX_NUM_STEPS] = {
	universal_step_0,
	A.write | dummy_reg.bout | PC.inc, // NOTE: pc cnt happens in case jump doesn't happens
    SP.lo.bout | PC.lo.write | PC_FLAG_ZERO,
    SP.hi.bout | PC.hi.write | PC_FLAG_ZERO,
	reset, reset, reset, reset
};

// jump if equal flag is carry flag is true
const uint16_t jmp_carry_template[MAX_NUM_STEPS] = {
	universal_step_0,
	PC.inc, // pc cnt in case jump doesn't happen
    SP.lo.bout | PC.lo.write | PC_FLAG_CARRY,
    SP.hi.bout | PC.hi.write | PC_FLAG_CARRY,
	reset, reset, reset, reset
};

// jump if equal flag is true
const uint16_t jmp_equal_template[MAX_NUM_STEPS] = {
	universal_step_0,
	PC.inc, // pc cnt in case jump doesn't happen
    SP.lo.bout | PC.lo.write | PC_FLAG_EQ,
    SP.hi.bout | PC.hi.write | PC_FLAG_EQ,
	reset, reset, reset, reset
};

// unconditional jump
const uint16_t jmp_direct_template[MAX_NUM_STEPS] = {
	universal_step_0,
    SP.lo.bout | PC.lo.write | PC_FLAG_DIRECT, // no need for pc cnt since its unconditional
    SP.hi.bout | PC.hi.write | PC_FLAG_DIRECT,
	reset, reset, reset, reset, reset
};

// clang-format on

void mw_instruction(int step, int instruction, int imm, int reg0, int reg1) {
  uint16_t *curr = &ucode[instruction][step][imm][reg0][reg1];
  if (imm == 0) {
    *curr = mw_template_reg[step];

    if (step == 3) {
      *curr |= reg_bout(reg1);
      *curr |= reg_write(reg0);
    }
  } else {
    // have to populate reg1
    *curr = mw_template_imm[step];

    if (step == 2) {
      *curr |= reg_write(reg0);
    }
  }
}

void lw_instruction(int step, int instruction, int imm, int reg0, int reg1) {
  // reg1 does not matter in any case
  uint16_t *curr = &ucode[instruction][step][imm][reg0][reg1];
  if (imm == 0) {
    *curr = lw_template_mar[step];

    if (step == 1) {
      *curr |= reg_write(reg0);
    }
  } else {
    *curr = lw_template_imm[step];

    if (step == 5) {
      *curr |= reg_write(reg0);
    }
  }
}

void sw_instruction(int step, int instruction, int imm, int reg0, int reg1) {
  // reg1 does not matter in any case
  uint16_t *curr = &ucode[instruction][step][imm][reg0][reg1];
  if (imm == 0) {
    *curr = sw_template_mar[step];

    if (step == 1) {
      *curr |= reg_bout(reg0);
    }
  } else {
    *curr = sw_template_imm[step];

    if (step == 5) {
      *curr |= reg_bout(reg0);
    }
  }
}

void push_instruction(int step, int instruction, int imm, int reg0, int reg1) {
  // reg1 does not matter in any case
  uint16_t *curr = &ucode[instruction][step][imm][reg0][reg1];
  if (imm == 0) {
    *curr = push_template_reg[step];

    if (step == 1) {
      *curr |= reg_bout(reg0);
    }
  } else {
    *curr = push_template_imm[step];
  }
}

void pop_instruction(int step, int instruction, int imm, int reg0, int reg1) {
  if (imm == 1) {
    // TODO:
    // technically unused combination, could use as extra instruction
    // maybe make this the lda instruction instead with variations to free up
    // one
  }

  // reg1 does not matter in any case
  uint16_t *curr = &ucode[instruction][step][imm][reg0][reg1];

  *curr = pop_template[step];

  if (step == 1) {
    *curr |= reg_bout(reg0);
  }
}

// TODO: any combination of imm and reg0 (4 bits) is a valid different
// instruction, could utilize for math?
// could utilize to load direct to SP or PC in one instruction (instruction +
// imm16)
void lda_instruction(int step, int instruction, int imm, int reg0, int reg1) {
  if (imm == 1) {
    // TODO:
    // technically unused combination, could use as extra instruction
  }
  // reg0 and reg1 don't matter

  uint16_t *curr = &ucode[instruction][step][imm][reg0][reg1];

  *curr = lda_template[step];
}

void jmp_instruction(int step, int instruction, int imm, int reg0, int reg1) {
  // reg1 does not matter in any case
  uint16_t *curr = &ucode[instruction][step][imm][reg0][reg1];
  if (imm == 0) {
    // jump if reg != 0
    *curr = jnz_template_reg[step];

    if (step == 1) {
      if (reg0 == 0) {
        // writing and reading into A, only perform increase
        *curr = PC.inc;
      } else {
        // writing to A from reg0
        *curr |= reg_bout(reg0);
      }
    }
  } else {
    // can use reg bits to jump based on flags
    if (reg0 == 0) {
      *curr = jmp_carry_template[step];
    } else if (reg0 == 1) {
      *curr = jmp_equal_template[step];
    } else if (reg0 == 2) {
      *curr = jmp_direct_template[step];
    }
    // defaults to direct jump
    *curr = jmp_direct_template[step];
  }
}

typedef void (*instruction_func)(int, int, int, int, int);

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

void process_address(int addr) {
  // clang-format off
  int step        = (addr & 0b00000000000111);
  int instruction = (addr & 0b00000001111000) >> 3;
  int imm         = (addr & 0b00000010000000) >> 7;
  int reg0        = (addr & 0b00011100000000) >> 8;
  int reg1        = (addr & 0b11100000000000) >> 11;
  // clang-format on

  // move word
  instruction_func istr_func = instructions_table[instruction];
  if (istr_func != (instruction_func)NULL) {
    istr_func(step, instruction, imm, reg0, reg1);
  } else {
    // TODO: could add error istr of some sort that triggers in unreachable
    // cases, maybe sets register a to an error code and halts
  }
}

void populate_ucode() {
  for (int addr = 0; addr <= MAX_ADDR; addr++) {
    process_address(addr);
  }
}

uint16_t getInstruction(int addr) {
  int step = (addr & 0b00000000000111);
  int instruction = (addr & 0b00000001111000) >> 3;
  int imm = (addr & 0b00000010000000) >> 7;
  int reg0 = (addr & 0b00011100000000) >> 8;
  int reg1 = (addr & 0b11100000000000) >> 11;
  // printf("%d %d %d %d %d\n", step, instruction, imm, reg0, reg1);

  return ucode[instruction][step][imm][reg0][reg1];
}

void write_ucode_logism() {
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
