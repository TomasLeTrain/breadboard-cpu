#include <inttypes.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>

#define MAX_ADDR_LEN 14
#define MAX_ADDR ((1 << MAX_ADDR_LEN) - 1)
#define NUM_REG 8
#define MAX_NUM_STEPS 8
#define NUM_INSTRUCTIONS 8

// ucode[instruction][step][imm][reg0][reg1]
uint16_t ucode[NUM_INSTRUCTIONS][MAX_NUM_STEPS][2][NUM_REG][NUM_REG];

// int slice_num(int n, int lo, int hi){
// 	int low_bits_mask = (1 << lo) - 1;
// 	int high_bits_mask = ~((1 << hi) - 1);
// 	return n & low_bits_mask & high_bits_mask;
// }

#define create_step(bus_out, addr_out, bus_write, other)                       \
  (bus_out | (addr_out << 4) | (bus_write << 6) | (other << 10))

typedef struct {
  uint16_t write;
  uint16_t bout;
} register_t;

typedef struct {
  uint16_t aout;
  uint16_t write_lo;
  uint16_t write_hi;
  uint16_t bout_lo;
  uint16_t bout_hi;
  uint16_t inc;
  uint16_t dec;
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
    .write_lo = create_step(0, 0, 6, 0),
    .write_hi = create_step(0, 0, 7, 0),
    .bout_lo = create_step(6, 2, 0, 0), // need to out on abus for bout
    .bout_hi = create_step(7, 2, 0, 0), // need to out on abus for bout
    .inc = create_step(0, 0, 0, 2),
    .dec = create_step(0, 0, 0, 0),
};

const addr_register_t PC = {
    .aout = create_step(0, 1, 0, 0),
    .write_lo = create_step(0, 0, 9, 0),
    .write_hi = create_step(0, 0, 10, 0),
    .bout_lo = create_step(6, 1, 0, 0), // need to out on abus for bout
    .bout_hi = create_step(7, 1, 0, 0), // need to out on abus for bout
    .inc = create_step(0, 0, 0, 1),
    .dec = create_step(0, 0, 0, 0),
};

const addr_register_t SP = {
    .aout = create_step(0, 3, 0, 0),
    .write_lo = create_step(0, 0, 11, 0),
    .write_hi = create_step(0, 0, 12, 0),
    .bout_lo = create_step(6, 3, 0, 0), // need to out on abus for bout
    .bout_hi = create_step(7, 3, 0, 0), // need to out on abus for bout
    .inc = create_step(0, 0, 0, 3),
    .dec = create_step(0, 0, 0, 4),
};

// IR = [PC]
const uint16_t universal_step_0 = MEM.bout | PC.aout | IR.write;
// pc cnt
const uint16_t universal_step_1 = PC.inc;

// TODO: determine?
const uint16_t reset = 0;

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
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | MAR.write_lo, // write first part of address to mar lo
	PC.inc, // pc cnt
	MEM.bout | PC.aout | MAR.write_hi, // write second part of address to mar hi
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
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | MAR.write_lo, // write first part of address to mar lo
	PC.inc, // pc cnt
	MEM.bout | PC.aout | MAR.write_hi, // write second part of address to mar hi
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


// clang-format on

int reg_write(int reg) {
  if (reg == 0)
    return A.write;
  if (reg == 1)
    return B.write;
  if (reg == 2)
    return X.write;
  if (reg == 3)
    return Y.write;
  if (reg == 4)
    return Z.write;
  if (reg == 5)
    return MAR.write_lo;
  if (reg == 6)
    return MAR.write_hi;
  return 0;
}

int reg_bout(int reg) {
  if (reg == 0)
    return A.bout;
  if (reg == 1)
    return B.bout;
  if (reg == 2)
    return X.bout;
  if (reg == 3)
    return Y.bout;
  if (reg == 4)
    return Z.bout;
  if (reg == 5)
    return MAR.bout_lo;
  if (reg == 6)
    return MAR.bout_hi;
  return 0;
}

void mw_instruction(int step, int instruction, int imm, int reg0, int reg1) {
  // reg1 does not matter in any case
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
  }

  // reg1 does not matter in any case
  uint16_t *curr = &ucode[instruction][step][imm][reg0][reg1];

  *curr = pop_template[step];

  if (step == 1) {
    *curr |= reg_bout(reg0);
  }
}

void process_address(int addr) {
  // clang-format off
  int step        = (addr & 0b00000000000111);
  int instruction = (addr & 0b00000000111000) >> 3;
  int imm         = (addr & 0b00000001000000) >> 6;
  int reg0        = (addr & 0b00001110000000) >> 7;
  int reg1        = (addr & 0b11110000000000) >> 10;
  // clang-format on 

  // move word
  if (instruction == 0) {
    mw_instruction(step, instruction, imm, reg0, reg1);
  } else if (instruction == 1) {
    // over all other combinations this is the same
    lw_instruction(step, instruction, imm, reg0, reg1);
  } else if (instruction == 2){
	// store word
    sw_instruction(step, instruction, imm, reg0, reg1);
	}

  // ucode[instruction][step][imm][reg0][reg1]
}

void populate_ucode() {
  for (int addr = 0; addr <= MAX_ADDR; addr++) {
    process_address(addr);
  }
}

int main() {}
