#include <assert.h>
#include <functional>
#include <iostream>
#include <stdarg.h>
#include <stdint.h>

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

struct step_t {
public:
  uint8_t bus_out = 0;
  uint8_t addr_out = 0;
  uint8_t bus_write = 0;
  uint8_t other = 0;

  // TODO: error check that each field is within bounds
  constexpr step_t(uint8_t bus_out, uint8_t addr_out, uint8_t bus_write,
                   uint8_t other)
      : bus_out(bus_out), addr_out(addr_out), bus_write(bus_write),
        other(other) {
    assert(bus_out < 16);
    assert(addr_out < 4);
    assert(bus_write < 16);
    assert(other < 16);
  }

  constexpr step_t(uint16_t data) : step_t(dataToStep(data)) {}
  constexpr step_t() : step_t(0) {}

  constexpr static step_t dataToStep(uint16_t data) {
    // clang-format off
    uint8_t bus_out   = (data & 0b0000000000001111);
    uint8_t addr_out  = (data & 0b0000000000110000) >> 4;
    uint8_t bus_write = (data & 0b0000001111000000) >> 6;
    uint8_t other     = (data & 0b0011110000000000) >> 10;
    // clang-format on
    return step_t(bus_out, addr_out, bus_write, other);
  }

  constexpr void mergeStep(const step_t &b) {
    if (bus_out != 0 && b.bus_out != 0) {
      std::cout << "conflict " << bus_out << " " << b.bus_out << std::endl;
    }
    if (bus_write != 0 && b.bus_write != 0) {
      std::cout << "bus_write conflict " << int(bus_write) << " "
                << int(b.bus_write) << std::endl;

      std::cout << "this: " << getRomData() << ", b: " << b.getRomData()
                << std::endl;
    }

    // assert((!(bus_out != 0 && b.bus_out != 0)) && "bus_out conflict");
    // assert((!(addr_out != 0 && b.addr_out != 0)) && "addr_out conflict");
    // assert((!(bus_write != 0 && b.bus_write != 0)) && "bus_write conflict");
    // assert((!(other != 0 && b.other != 0)) && "other conflict");

    bus_out |= b.bus_out;
    addr_out |= b.addr_out;
    bus_write |= b.bus_write;
    other |= b.other;
  }

  constexpr uint16_t getRomData() const {
    // TODO: mask after shifting to ensure no conflicts/ error check that each
    // field is within bounds
    uint32_t result = 0;
    result |= bus_out;
    result |= static_cast<uint32_t>(addr_out) << 4;
    result |= static_cast<uint32_t>(bus_write) << 6;
    result |= static_cast<uint32_t>(other) << 10;
    return static_cast<uint16_t>(result);
  }

  constexpr static step_t mergeSteps(const step_t &a, const step_t &b) {
    step_t result = a;
    result.mergeStep(b);
    return result;
  }

  constexpr step_t operator|(const step_t &other) const {
    return mergeSteps(*this, other);
  }

  constexpr step_t &operator|=(const step_t &other) {
    mergeStep(other);
    return *this;
  }
};

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
  std::string name;
} reg_t;

typedef struct {
  step_t aout;
  reg_t lo;
  reg_t hi;
  step_t inc;
  step_t dec;
} addr_register_t;

// ucode[instruction][step][imm][reg0][reg1]
step_t ucode[NUM_INSTRUCTIONS][MAX_NUM_STEPS][2][NUM_REG][NUM_REG];

// clang-format off
constexpr step_t empty_instruction{};

constexpr reg_t A   = {.write = step_t(0, 0, 1, 0), .bout = step_t(1, 0, 0, 0), .name = "A"};
constexpr reg_t B   = {.write = step_t(0, 0, 2, 0), .bout = step_t(2, 0, 0, 0), .name = "B"};
constexpr reg_t X   = {.write = step_t(0, 0, 3, 0), .bout = step_t(3, 0, 0, 0), .name = "X"};
constexpr reg_t Y   = {.write = step_t(0, 0, 4, 0), .bout = step_t(4, 0, 0, 0), .name = "Y"};
constexpr reg_t Z   = {.write = step_t(0, 0, 5, 0), .bout = step_t(5, 0, 0, 0), .name = "Z"};
// also requires putting some addr register on the abus
constexpr reg_t MEM = {.write = step_t(0, 0, 8, 0),  .bout = step_t(8, 0, 0, 0), .name = "MEM"};
constexpr reg_t IR  = {.write = step_t(0, 0, 13, 0), .bout = empty_instruction, .name = "IR"};

constexpr reg_t F   = {.write = empty_instruction,   .bout = step_t(9, 0, 0, 0), .name = "F"};

constexpr reg_t IR2 = {.write = step_t(0, 0, 14, 0), .bout = empty_instruction, .name = "IR2"};
constexpr reg_t FLAG= {.write = step_t(0, 0, 15, 0), .bout = step_t(10, 0, 0, 0), .name = "FLAG"};

constexpr addr_register_t MAR = {
    .aout = step_t(0, 2, 0, 0),
    .lo = {.write = step_t(0, 0, 6, 0), .bout = step_t(6, 2, 0, 0), .name = "MAR_lo"},
    .hi = {.write = step_t(0, 0, 7, 0), .bout = step_t(7, 2, 0, 0), .name = "MAR_hi"},
    .inc = step_t(0, 0, 0, 2),
    .dec = empty_instruction,
};

constexpr addr_register_t PC = {
    .aout = step_t(0, 1, 0, 0),
    .lo = {.write = step_t(0, 0, 9, 0), .bout = step_t(6, 1, 0, 0), .name = "PC_lo"},
    .hi = {.write = step_t(0, 0, 10, 0), .bout = step_t(7, 1, 0, 0), .name = "PC_lo"},
    .inc = step_t(0, 0, 0, 1),
    .dec = empty_instruction,
};

constexpr addr_register_t SP = {
    .aout = step_t(0, 3, 0, 0),
    .lo = {.write = step_t(0, 0, 11, 0), .bout = step_t(6, 3, 0, 0), .name = "SP_lo"},
    .hi = {.write = step_t(0, 0, 12, 0), .bout = step_t(7, 3, 0, 0), .name = "SP_lo"},
    .inc = step_t(0, 0, 0, 3),
    .dec = step_t(0, 0, 0, 4),
};
// clang-format on

constexpr step_t PC_FLAG_DIRECT = empty_instruction;
constexpr step_t PC_FLAG_ZERO = step_t(0, 0, 0, 5);
constexpr step_t PC_FLAG_EQ = step_t(0, 0, 0, 6);
constexpr step_t PC_FLAG_CARRY = step_t(0, 0, 0, 7);

// writes values from alu into flag
// WARN: outputs into the data bus!
constexpr step_t FLAG_WRITE_ALU = F.bout;

constexpr reg_t intToRegister(uint8_t reg) {
  if (reg == 0)
    return A;
  if (reg == 1)
    return B;
  if (reg == 2)
    return X;
  if (reg == 3)
    return Y;
  if (reg == 4)
    return Z;
  if (reg == 5)
    return MAR.lo;
  if (reg == 6)
    return MAR.hi;
  if (reg == 7)
    return FLAG;
  assert(0 && "out of range register index");
}

struct StepCreator {
private:
  step_t step;
  reg_t reg0;
  reg_t reg1;
  step_t flag;

  bool reg0_write = false;
  bool reg0_bout = false;
  bool reg1_write = false;
  bool reg1_bout = false;

  bool output_flags_selector = false;

public:
  constexpr StepCreator(const step_t &step) : step(step) {}
  constexpr StepCreator(uint8_t bus_out, uint8_t addr_out, uint8_t bus_write,
                        uint8_t other)
      : step(bus_out, addr_out, bus_write, other) {}

  constexpr StepCreator(const step_t &step, bool reg0_write, bool reg0_bout,
                        bool reg1_write, bool reg1_bout,
                        bool output_flags_selector)
      : step(step), reg0_write(reg0_write), reg0_bout(reg0_bout),
        reg1_write(reg1_write), reg1_bout(reg1_bout),
        output_flags_selector(output_flags_selector) {}

  constexpr StepCreator() : step(0) {}

  constexpr void setRegisters(reg_t reg0, reg_t reg1) {
    this->reg0 = reg0;
    this->reg1 = reg1;
  }

  constexpr void setFlag(step_t flag) { this->flag = flag; }

  constexpr void setRegisters(const split_addr_t *instruction) {
    setRegisters(intToRegister(instruction->reg0),
                 intToRegister(instruction->reg1));
  }

  constexpr step_t getStep() const {
    step_t result = step;
    if (reg0_write && reg1_write) {
      std::cout << "reg0/reg1 write conflict: " << reg0.name << ", " << reg1.name << std::endl;
    }
    if (reg0_bout && reg1_bout) {
      std::cout << "reg0/reg1 bus conflict:" << reg0.name << ", " << reg1.name << std::endl;
    }

    if (reg0_write)
      result |= reg0.write;
    if (reg0_bout)
      result |= reg0.bout;
    if (reg1_write)
      result |= reg1.write;
    if (reg1_bout)
      result |= reg1.bout;
    if (output_flags_selector)
      result |= flag;
    return result;
  }

  constexpr uint16_t getRomData() const { return getStep().getRomData(); }

  constexpr void merge(const StepCreator &other) {
    step |= other.step;
    reg0_write = reg0_write || other.reg0_write;
    reg0_bout = reg0_bout || other.reg0_bout;
    reg1_write = reg1_write || other.reg0_write;
    reg1_bout = reg1_bout || other.reg0_bout;
  }

  constexpr static StepCreator merge(const StepCreator &a,
                                     const StepCreator &b) {
    StepCreator result = a;
    result.merge(b);
    return result;
  }

  constexpr StepCreator operator|(const StepCreator &other) const {
    return merge(*this, other);
  }

  constexpr StepCreator &operator|=(const StepCreator &other) {
    merge(other);
    return *this;
  }

  constexpr StepCreator operator|(const step_t &other) const {
    StepCreator result = *this;
    result.step |= other;
    return result;
  }

  constexpr StepCreator &operator|=(const step_t &other) {
    step |= other;
    return *this;
  }
};

// used specifically when
constexpr StepCreator operator|(const step_t &lhs, const StepCreator &rhs) {
  return rhs | lhs;
}

const StepCreator reg0_write =
    StepCreator(empty_instruction, true, false, false, false, false);
const StepCreator reg0_bout =
    StepCreator(empty_instruction, false, true, false, false, false);
const StepCreator reg1_write =
    StepCreator(empty_instruction, false, false, true, false, false);
const StepCreator reg1_bout =
    StepCreator(empty_instruction, false, false, false, true, false);

const StepCreator output_flags_selector =
    StepCreator(empty_instruction, false, false, false, false, true);

// IR = [PC]
const step_t universal_step_0 = MEM.bout | PC.aout | IR.write;
// pc cnt6
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
const step_t reset = step_t(15, 0, 0, 0);
const step_t halt = step_t(14, 0, 0, 0);

const step_t error = 0xffff;

// clang-format off

// TODO: writting register to itself should be error or nop?
// reg0 = reg1
const StepCreator mw_template_reg[MAX_NUM_STEPS] = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | IR2.write, // need to load ir2 to figure out reg1
  	reg1_bout | reg0_write | PC.inc, // read from reg1 to reg0, pc cnt
	reset, reset, reset, reset
};

// reg = imm8
const StepCreator mw_template_imm[MAX_NUM_STEPS] = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | reg0_write, // write the immediate into reg0
	PC.inc, // pc cnt
	reset, reset, reset, reset
};

// reg = [MAR]
const StepCreator lw_template_mar[MAX_NUM_STEPS] = {
	universal_step_0,
	MEM.bout | MAR.aout | reg0_write | PC.inc, // read from addr MAR into register, pc cnt
	reset, reset, reset, reset, reset, reset
};

// reg = [imm16]
const StepCreator lw_template_imm[MAX_NUM_STEPS] = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	MEM.bout | MAR.aout | reg0_write |  PC.inc, // read from addr MAR into register, pc cnt
	reset, reset
};


// reg = [MAR]
const StepCreator sw_template_mar[MAX_NUM_STEPS] = {
	universal_step_0,
	MEM.write | MAR.aout | reg0_bout | PC.inc, // read from addr MAR into register, pc cnt
	reset, reset, reset, reset, reset, reset
};

// reg = [imm16]
const StepCreator sw_template_imm[MAX_NUM_STEPS] = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	MEM.write | MAR.aout | reg0_bout |  PC.inc, // read from addr MAR into register, pc cnt
	reset, reset
};


// [SP--] = reg
const StepCreator push_template_reg[MAX_NUM_STEPS] = {
	universal_step_0,
	MEM.write | SP.aout | reg0_bout | PC.inc, // read from reg into mem at SP addr, pc cnt
	SP.dec,
	reset, reset, reset, reset, reset
};

// [SP--] = imm8, overrides A reg
const StepCreator push_template_imm[MAX_NUM_STEPS] = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | A.write, // write into IR2
	MEM.write | SP.aout | A.bout | PC.inc, // read from IR2 into [SP], pc cnt
	SP.dec, // SP--
	reset, reset, reset
};

// reg0 = [SP++]
const StepCreator pop_template[MAX_NUM_STEPS] = {
	universal_step_0,
	MEM.bout | SP.aout | reg0_write | PC.inc, // write from [SP] into reg0, pc cnt
	SP.inc, // SP++
	reset, reset, reset, reset, reset
};

// MAR = imm16
const StepCreator lda_template[MAX_NUM_STEPS] = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	PC.inc, // pc cnt
	reset,reset
};

// JNZ reg -> PC = MAR if reg != 0 else NOP
const StepCreator jnz_template_reg[MAX_NUM_STEPS] = {
	universal_step_0,
	A.write | reg0_bout | PC.inc, // NOTE: pc cnt happens in case jump doesn't happens
	FLAG_WRITE_ALU, // write zero result to flag register
    MAR.lo.bout | PC.lo.write | PC_FLAG_ZERO,
    MAR.hi.bout | PC.hi.write | PC_FLAG_ZERO,
	reset, reset, reset
};

// can save instruction if A is already loaded
const StepCreator jnz_template_reg_A[MAX_NUM_STEPS] = {
	universal_step_0,
	FLAG_WRITE_ALU | PC.inc, // write zero result to flag register, pc cnt in case jump doesn't happens
    MAR.lo.bout | PC.lo.write | PC_FLAG_ZERO,
    MAR.hi.bout | PC.hi.write | PC_FLAG_ZERO,
	reset, reset, reset, reset
};

// jump if equal flag is carry flag is true
const StepCreator jmp_imm16_template[MAX_NUM_STEPS] = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	PC.inc, // NOTE: pc cnt happens in case jump doesn't happens
    MAR.lo.bout | PC.lo.write | output_flags_selector, // load from mar into pc if flag
    MAR.hi.bout | PC.hi.write | output_flags_selector, // load from mar into pc if flag
};

// jump if equal flag is true
const StepCreator jmp_mar_template[MAX_NUM_STEPS] = {
	universal_step_0,
	PC.inc, // NOTE: pc cnt in case jump doesn't happen
    MAR.lo.bout | PC.lo.write | output_flags_selector,
    MAR.hi.bout | PC.hi.write | output_flags_selector,
	reset, reset, reset, reset
};

// TODO: all math variants could have faster variants if reg0/reg1 are equal to a/b

// TODO: special case if reg0 = b, reg1 = a (impossible to swap registers without intermediate)

// reg0 = reg0 OP reg1
const StepCreator math_template_reg[MAX_NUM_STEPS] = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | IR2.write, // need to load ir2 to figure out reg1
	reg0_bout | A.write | PC.inc, // load reg0 into a
	reg1_bout | B.write, // load reg1 into b
	F.bout | reg0_write, // do math op, save to reg0, writes to flag reg
	reset, reset
};

// reg0 = reg0 OP reg1
const StepCreator math_template_imm[MAX_NUM_STEPS] = {
	universal_step_0,
	reg0_bout | A.write | PC.inc, // load reg0 into a first (in case reg0 = b), pc cnt
	MEM.bout | PC.aout | B.write, // load imm into b
	F.bout | reg0_write | PC.inc, // save F to reg0, writes to flag reg
	reset, reset, reset, reset
};

// reg0 = ~reg0
const StepCreator not_template_none[MAX_NUM_STEPS] = {
	universal_step_0,
	reg0_bout | A.write | PC.inc, // load reg0 into a, pc cnt
	F.bout | reg0_write, // do math op, save to reg0, writes to flag reg
	reset, reset, reset, reset
};

// reg0 = ~reg1
const StepCreator not_template_reg[MAX_NUM_STEPS] = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | IR2.write, // need to load ir2 to figure out reg1
	reg1_bout | A.write | PC.inc, // load reg0 into a
	F.bout | reg0_write, // do math op, save to reg1, writes to flag reg
	reset, reset, reset
};

// clang-format on

step_t *get_ucode_ptr(const split_addr_t *instruction) {
  return &ucode[instruction->instruction][instruction->step][instruction->imm]
               [instruction->reg0][instruction->reg1];
}

void create_instruction(const StepCreator step_template[MAX_NUM_STEPS],
                        const split_addr_t *instruction) {
  step_t *curr = get_ucode_ptr(instruction);
  StepCreator template_step = step_template[instruction->step];
  template_step.setRegisters(instruction);
  *curr = template_step.getStep();
}

void setStep(const StepCreator &step, const split_addr_t *instruction) {
  step_t *curr = get_ucode_ptr(instruction);
  StepCreator template_step = step;
  *curr = template_step.getStep();
}

void create_instruction(const StepCreator step_template_reg[MAX_NUM_STEPS],
                        const StepCreator step_template_imm[MAX_NUM_STEPS],
                        const split_addr_t *instruction) {
  const StepCreator *step_template = nullptr;

  if (instruction->imm == 0) {
    step_template = step_template_reg;
  } else {
    step_template = step_template_imm;
  }
  create_instruction(step_template, instruction);
}

void mw_instruction(const split_addr_t *instruction) {
  create_instruction(mw_template_reg, mw_template_imm, instruction);
}

void lw_instruction(const split_addr_t *instruction) {
  create_instruction(lw_template_mar, lw_template_imm, instruction);
}

void sw_instruction(const split_addr_t *instruction) {
  create_instruction(sw_template_mar, sw_template_imm, instruction);
}

void push_instruction(const split_addr_t *instruction) {
  create_instruction(push_template_reg, push_template_imm, instruction);
}

void pop_instruction(const split_addr_t *instruction) {
  create_instruction(pop_template, instruction);
}

void lda_instruction(const split_addr_t *instruction) {
  create_instruction(lda_template, instruction);
}

void jmp_instruction(const split_addr_t *instruction) {
  const StepCreator *step_template = nullptr;

  // jump if reg != 0
  if (instruction->imm == 0) {
    // special case -> can save step if already A
    if (instruction->reg0 == 0) {
      step_template = jnz_template_reg_A;
    } else {
      // general case -> writing to A from reg0
      step_template = jnz_template_reg;
    }
    create_instruction(step_template, instruction);
  } else {
    uint8_t flag_idx = instruction->reg0 & 0b011;
    uint8_t using_imm16_flag = instruction->reg0 & 0b100;
    const static step_t idx_to_flag[] = {PC_FLAG_DIRECT, PC_FLAG_CARRY,
                                         PC_FLAG_EQ, PC_FLAG_DIRECT};
    step_t flag_step_bits = idx_to_flag[flag_idx];

    // TODO: unconditional jump could save one step by skipping pc cnt
    if (using_imm16_flag) {
      step_template = jmp_imm16_template;
    } else {
      step_template = jmp_mar_template;
    }

    StepCreator template_step = step_template[instruction->step];
    template_step.setRegisters(instruction);
    template_step.setFlag(flag_step_bits);
    setStep(template_step, instruction);
  }
}

void math_instruction(const split_addr_t *instruction) {
  create_instruction(math_template_reg, math_template_imm, instruction);
}

void not_instruction(const split_addr_t *instruction) {
  create_instruction(not_template_none, not_template_reg, instruction);
}

using instruction_func = std::function<void(const split_addr_t *)>;

instruction_func instructions_table[16] = {
    mw_instruction,   lw_instruction,   //
    sw_instruction,   push_instruction, //
    pop_instruction,  lda_instruction,  //
    jmp_instruction,  nullptr,          //
    math_instruction, math_instruction, // add, adc
    math_instruction, math_instruction, // sub, sbc
    not_instruction,  math_instruction, // not, xor
    math_instruction, math_instruction, // or, and
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
  if (istr_func) {
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
    std::cout << addr << std::endl;
    uint16_t curr = getInstruction(addr).getRomData();
    // printf("%04X", curr);
    // if (addr % 16 == 15) {
    //   printf("\n");
    // } else {
    //   printf(" ");
    // }
  }
}

int main() {
  populate_ucode();
  write_ucode_logism();
}
