#include <assert.h>
#include <functional>
#include <iostream>
#include <stdarg.h>
#include <stdint.h>

constexpr uint32_t VRAM_BITS = 1;
constexpr uint32_t IR2_NUM_BITS = 4;
constexpr uint32_t STEP_NUM_BITS = 4;
constexpr uint32_t REG_NUM_BITS = 3;
constexpr uint32_t ISTR_NUM_BITS = 4;
constexpr uint32_t IMM_NUM_BITS = 1;

constexpr uint32_t ADDR_BITS = VRAM_BITS + STEP_NUM_BITS + IMM_NUM_BITS +
                               ISTR_NUM_BITS + REG_NUM_BITS + IR2_NUM_BITS;

// max included address
constexpr uint32_t MAX_ADDR = (1 << ADDR_BITS);
constexpr uint32_t MAX_ADDR_INC = MAX_ADDR - 1;

constexpr uint32_t NUM_VRAM_BITS = (1 << VRAM_BITS);
constexpr uint32_t NUM_IR2_BITS = (1 << IR2_NUM_BITS);
constexpr uint32_t MAX_NUM_STEPS = (1 << STEP_NUM_BITS);
constexpr uint32_t NUM_INSTRUCTIONS = (1 << ISTR_NUM_BITS);
constexpr uint32_t NUM_REG = (1 << REG_NUM_BITS);

constexpr uint32_t BUS_OUT_BITS = 4;
constexpr uint32_t ADDR_OUT_BITS = 2;
constexpr uint32_t BUS_WRITE_BITS = 4;
constexpr uint32_t OTHER_BITS = 2;
constexpr uint32_t FLAG_SELECT_BITS = 3;
constexpr uint32_t PC_CNT_BIT = 1;

/*
clang-format off

TODO: might be better to have arbitrary instruction order to maximize possible space use

registers:
- A			000: special register written to by math operations, gp register
- B			001: holds operand for math ops, gp register
- X			010: gp register
- Y			011: gp register
- Z			100: gp register
- MAR.LO	101: low bits of MAR addr register, could use as gp register
- MAR.HI	110: high bits of MAR addr register, could use as gp register
- FLAGS		111: holds 4 bits of flags after math ops (or whatever gets written to it)

every yyy has 8 unused
4 unused, 1 half unused

single unused * 16 combinations


every unused can be a full instruction (without yyy support)


possible instruction additions


Instructions:
0000:
move word: 	xxx <- yyy			| [0000 0 xxx] [yyy.....]		| MV xxx, yyy
move word: 	xxx <- imm8			| [0000 1 xxx] [imm8    ]		| MV xxx, imm8

0001:
CMP: 	a=xxx,b=yyy,flg udpate	| [0001 0 xxx]					| CMP xxx, yyy
CMP: 	a=xxx,b=imm8,flg udpate | [0001 1 xxx][imm8]			| CMP xxx, imm8

0010:
store word:	xxx -> mem[mar]		| [0010 0 xxx]					| STR xxx
store word:	xxx -> mem[imm16]	| [0010 1 xxx] [imm16][imm16]	| STR xxx, imm16

0011:
push:		xxx -> mem[SP],SP--	| [0011 0 xxx]					| PUSH xxx
push:		imm8 -> mem[SP],SP--| [0011 1 000]					| PUSH imm8 // overrides A reg
SP INC:							| [0011 1 001]					| INC SP
MAR INC:						| [0011 1 010]					| INC MAR
SP DEC:							| [0011 1 011]					| DEC SP
MAR <- PC:						| [0011 1 100]					| LDA MAR, PC
MAR <- SP:						| [0011 1 101]					| LDA MAR, SP
MAR <- imm16:				 	| [0011 1 110][imm16][imm16]	| LDA MAR, imm16
SP <- imm16						| [0011 1 111][imm16][imm16]	| LDA SP, imm16

0100:
pop:		xxx <- mem[SP],SP++	| [0100 0 xxx]					| POP xxx
LDA SP:		SP <- MAR			| [0101 1 000]					| LDA SP, MAR (one instruction)
update flag reg:				| [0100 1 001]					| SET FLAG
nop:							| [0100 1 010]					| NOP
Z = vram[MAR]:					| [0100 1 011]					| VRAM_READ Z
vram[MAR] = Z:					| [0100 1 100]					| VRAM_WRITE Z
Y = vram[MAR]:					| [0100 1 011]					| VRAM_READ Y
vram[MAR] = Y:					| [0100 1 100]					| VRAM_WRITE Y
halt:							| [0100 1 111]					| HALT

0101:
JNZ:		PC <- MAR: xxx != 0	| [0101 0 xxx]					| JNZ xxx

JMP:		PC <- MAR			| [0101 1 000]					| JMP ; LDA PC, MAR
JC:			PC <- MAR: CRRY FLG	| [0101 1 001]					| JC
JEQ:		PC <- MAR: EQ FLG	| [0101 1 010]					| JEQ
JNZ:		PC <- MAR: ZRO FLG	| [0101 1 011]					| JNZ

JMP:		PC <- imm16			| [0101 1 100][imm16][imm16]	| JMP imm16 ; LDA PC, imm16
JC:			PC <- imm16:CRRY FLG| [0101 1 101][imm16][imm16]	| JC  imm16
JEQ:		PC <- imm16: EQ FLG	| [0101 1 110][imm16][imm16]	| JEQ imm16
JNZ:		PC <- imm16: ZRO FLG| [0101 1 111][imm16][imm16]    | JNZ imm16

0110:
keyb input:	xxx <- KEYB			| [0110 0 xxx]					| KEYB xxx
unused half:					| [0110 1 xxx]					|

0111:
load word: 	xxx <- mem[mar]		| [0001 0 xxx]					| LOAD xxx
load word: 	xxx <- mem[imm16]	| [0001 1 xxx] [imm16][imm16]	| LOAD xxx, imm16

1000:
sub flg crry:	xxx <- xxx - yyy	| [1000 0 xxx][yyy.....]	| SBC xxx, yyy 
sub flg crry:	xxx <- xxx - imm8	| [1000 1 xxx][imm8]		| SBC xxx, imm8

1001: 
sub carry on:	xxx <- xxx + yyy	| [1001 0 xxx][yyy.....]	| SUB xxx, yyy 
sub carry on:	xxx <- xxx + imm8	| [1001 1 xxx][imm8]		| SUB xxx, imm8

1010:
add flg crry:	xxx <- xxx + yyy	| [1010 0 xxx][yyy.....]	| ADC xxx, yyy 
add flg crry:	xxx <- xxx + imm8	| [1010 1 xxx][imm8]		| ADC xxx, imm8

1011:
add no carry:	xxx <- xxx + yyy	| [1000 0 xxx][yyy.....]	| ADD xxx, yyy 
add no carry:	xxx <- xxx + imm8	| [1000 1 xxx][imm8]		| ADD xxx, imm8

1100:
not:			xxx <- ~yyy			| [1100 0 xxx]				| NOT xxx, yyy
not:			xxx <- ~xxx			| [1100 1 xxx]				| NOT xxx

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
private:
  uint8_t bus_out = 0;
  uint8_t addr_out = 0;
  uint8_t bus_write = 0;
  uint8_t other = 0;
  uint8_t flag_select = 0;
  uint8_t pc_cnt = 0;

public:
  constexpr step_t(uint8_t bus_out, uint8_t addr_out, uint8_t bus_write,
                   uint8_t other, uint8_t flag_select, uint8_t pc_cnt)
      : bus_out(bus_out), addr_out(addr_out), bus_write(bus_write),
        other(other), flag_select(flag_select), pc_cnt(pc_cnt) {
    assert(bus_out < (1 << BUS_OUT_BITS));
    assert(addr_out < (1 << ADDR_OUT_BITS));
    assert(bus_write < (1 << BUS_OUT_BITS));
    assert(other < (1 << OTHER_BITS));
    assert(flag_select < (1 << FLAG_SELECT_BITS));
    assert(pc_cnt < (1 << PC_CNT_BIT));
  }

  std::string toString() const {
    std::string res = "bus_out: ";
    res += std::to_string(bus_out) + ", addr_out: ";
    res += std::to_string(addr_out) + ",  bus_write: ";
    res += std::to_string(bus_write) + ", other: ";
    res += std::to_string(other) + ", flag_select: ";
    res += std::to_string(flag_select) + ", pc_cnt: ";
    res += std::to_string(pc_cnt);
    return res;
  }

  // constexpr step_t(uint8_t bus_out, uint8_t addr_out, uint8_t bus_write,
  //                  uint8_t other)
  //     : step_t(bus_out, addr_out, bus_write, other, 0) {}

  constexpr step_t() : step_t(0, 0, 0, 0, 0, 0) {}

  constexpr void mergeStep(const step_t &b) {
    if (bus_out != 0 && b.bus_out != 0) {
      // can cause compile-time error
      if consteval {
        throw "bus_out conflict!";
      }
      std::cout << "bout conflict " << int(bus_out) << " " << int(b.bus_out)
                << std::endl;
      std::cout << "this: " << getRomData() << ", b: " << b.getRomData()
                << std::endl;
    }
    if (bus_write != 0 && b.bus_write != 0) {
      // can cause compile-time error
      if consteval {
        throw "bus_write conflict!";
      }
      std::cout << "bus_write conflict " << int(bus_write) << " "
                << int(b.bus_write) << std::endl;
      std::cout << "this: " << getRomData() << ", b: " << b.getRomData()
                << std::endl;
    }

    if (addr_out != 0 && b.addr_out != 0) {
      // can cause compile-time error
      if consteval {
        throw "addr_out conflict!";
      }
      std::cout << "bus_write conflict " << int(addr_out) << " "
                << int(b.addr_out) << std::endl;
      std::cout << "this: " << getRomData() << ", b: " << b.getRomData()
                << std::endl;
    }

    if (other != 0 && b.other != 0) {
      // can cause compile-time error
      if consteval {
        throw "other conflict!";
      }
      std::cout << "other conflict " << int(other) << " " << int(b.other)
                << std::endl;
      std::cout << "this: " << getRomData() << ", b: " << b.getRomData()
                << std::endl;
    }

    if (flag_select != 0 && b.flag_select != 0) {
      // can cause compile-time error
      if consteval {
        throw "flag_select conflict!";
      }
      std::cout << "flag_select conflict " << int(other) << " " << int(b.other)
                << std::endl;
      std::cout << "this: " << getRomData() << ", b: " << b.getRomData()
                << std::endl;
    }

    if (pc_cnt != 0 && b.pc_cnt != 0) {
      // can cause compile-time error
      if consteval {
        throw "pc_cnt conflict!";
      }
      std::cout << "pc_cnt conflict " << int(other) << " " << int(b.other)
                << std::endl;
      std::cout << "this: " << getRomData() << ", b: " << b.getRomData()
                << std::endl;
    }

    bus_out |= b.bus_out;
    addr_out |= b.addr_out;
    bus_write |= b.bus_write;
    other |= b.other;
    flag_select |= b.flag_select;
    pc_cnt |= b.pc_cnt;
  }

  constexpr uint16_t getRomData() const {
    uint32_t result = 0;
    result |= bus_out;                                  // 4 bits wide
    result |= static_cast<uint32_t>(bus_write) << 4;    // 4 bits wide
    result |= static_cast<uint32_t>(addr_out) << 8;     // 2 bits wide
    result |= static_cast<uint32_t>(other) << 10;       // 2 bits wide
    result |= static_cast<uint32_t>(flag_select) << 12; // 3 bits wide
    result |= static_cast<uint32_t>(pc_cnt) << 15;      // 1 bit wide
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
  uint32_t original_address;
  // 4 bits wide
  uint8_t step;
  // 3 bits wide
  uint8_t reg0;
  // 1 bit wide
  uint8_t imm;
  // 4 bits wide
  uint8_t instruction;
  // 3 bits wide
  uint8_t reg1;
  // 1 bit wide
  uint8_t ir2_extra_bits;
  // 1 bit wide
  uint8_t not_vram_active;
} split_addr_t;

struct reg_t {
  step_t write;
  step_t bout;
  std::string name;
};

struct shift_reg_t : reg_t {
  step_t shift_left;
  step_t shift_right;
};

typedef struct {
  step_t aout;
  reg_t lo;
  reg_t hi;
  step_t inc;
  step_t dec;
} addr_register_t;

// ucode[instruction][step][imm][reg0][reg1][ir2 extra bits]
step_t ucode[NUM_INSTRUCTIONS][MAX_NUM_STEPS][2][NUM_REG][NUM_REG][NUM_IR2_BITS]
            [NUM_VRAM_BITS];

constexpr step_t bout(uint8_t bout_idx) {
  return step_t(bout_idx, 0, 0, 0, 0, 0);
}
constexpr step_t aout(uint8_t aout_idx) {
  return step_t(0, aout_idx, 0, 0, 0, 0);
}
constexpr step_t write(uint8_t write_idx) {
  return step_t(0, 0, write_idx, 0, 0, 0);
}
constexpr step_t other(uint8_t other_idx) {
  return step_t(0, 0, 0, other_idx, 0, 0);
}
constexpr step_t flag_select(uint8_t flag_select_idx) {
  return step_t(0, 0, 0, 0, flag_select_idx, 0);
}

constexpr step_t pc_cnt(uint8_t pc_cnt_idx) {
  return step_t(0, 0, 0, 0, 0, pc_cnt_idx);
}

// clang-format off
constexpr step_t empty_instruction{};

constexpr reg_t A   = {.write = write(0b1000 | 0), .bout = bout(0b1000 | 0), .name = "A"};
constexpr reg_t B   = {.write = write(0b1000 | 1), .bout = bout(0b1000 | 1), .name = "B"};

constexpr shift_reg_t X = {
	{
	.write = write(0b1000 | 5),
	.bout = bout(0b1000 | 5),
	.name = "X"
	},
	other(3) | flag_select(0), // shift left
	other(3) | flag_select(1)  // shift right
};
constexpr shift_reg_t Y = {
	{
	.write = write(0b1000 | 6),
	.bout = bout(0b1000 | 6),
	.name =  "Y"
	},
	other(3) | flag_select(2), // shift left
	other(3) | flag_select(3)  // shift right
};

constexpr reg_t Z   = {.write = write(7), .bout = bout((0b1000 | 7)), .name = "Z"};

// also requires putting some addr register on the abus
constexpr reg_t MEM = {.write = write(6), .bout = bout(1), .name = "MEM"};
constexpr reg_t VRAM = {.write = other(3) | flag_select(4), .bout = other(3) | flag_select(5), .name = "VRAM"};

constexpr reg_t F   = {.write = empty_instruction,   .bout = bout(0b1000 | 3),.name = "F"};

constexpr reg_t IR  = {.write = write(0b1000 | 7), .bout = empty_instruction, .name = "IR"};
constexpr reg_t IR2 = {.write = write(1), .bout = empty_instruction, .name = "IR2"};

constexpr reg_t FLAG = {.write = write(0b1000 | 4), .bout = bout(0b1000 | 4),.name = "FLAG"};
constexpr reg_t KEYB  = {.write = empty_instruction, .bout = bout(0b1000 | 2), .name = "KEYB"};

constexpr addr_register_t MAR = {
    .aout = aout(2),
    .lo = {.write = write(5), .bout = aout(2) | bout(2), .name = "MAR_lo"},
    .hi = {.write = write(4), .bout = aout(2) | bout(3), .name = "MAR_hi"},
    .inc = bout(4),
    .dec = empty_instruction,
};

constexpr addr_register_t PC = {
    .aout = aout(1),
    .lo = {.write = write(0b1000 | 3), .bout =  aout(1) | bout(2), .name = "PC_lo"},
    .hi = {.write = write(0b1000 | 2), .bout = aout(1) | bout(3), .name = "PC_hi"},
    .inc = pc_cnt(1),
    .dec = empty_instruction,
};

constexpr addr_register_t SP = {
    .aout = aout(3),
    .lo = {.write = write(3), .bout = aout(3) | bout(2), .name = "SP_lo"},
    .hi = {.write = write(2), .bout = aout(3) | bout(3), .name = "SP_hi"},
    .inc = bout(6),
    .dec = bout(7)
};
// clang-format on

constexpr step_t PC_FLAG_DIRECT = flag_select(0);
constexpr step_t PC_FLAG_CARRY = flag_select(1);
constexpr step_t PC_FLAG_EQ = flag_select(2);
constexpr step_t PC_FLAG_ZERO = flag_select(3);
constexpr step_t PC_FLAG_X_LEFT = flag_select(4);
constexpr step_t PC_FLAG_X_RIGHT = flag_select(5);

// writes values from alu into flag
constexpr step_t FLAG_WRITE_ALU = other(1);

constexpr step_t reset = other(2);
constexpr step_t halt = bout(5);
constexpr step_t error = halt;

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

  constexpr StepCreator(const step_t &step, bool reg0_write, bool reg0_bout,
                        bool reg1_write, bool reg1_bout,
                        bool output_flags_selector)
      : step(step), reg0_write(reg0_write), reg0_bout(reg0_bout),
        reg1_write(reg1_write), reg1_bout(reg1_bout),
        output_flags_selector(output_flags_selector) {}

  // defaults to reset to avoid having to set on all templates
  constexpr StepCreator() : step(halt) {}

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
    // TODO: see if reg0/1 write and bout are on at same time, could turn into
    // nop?

    if (reg0_write && reg1_write) {
      std::cout << "reg0/reg1 write conflict: " << reg0.name << ", "
                << reg1.name << std::endl;
    }
    if (reg0_bout && reg1_bout) {
      std::cout << "reg0/reg1 bus conflict:" << reg0.name << ", " << reg1.name
                << std::endl;
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
    reg1_write = reg1_write || other.reg1_write;
    reg1_bout = reg1_bout || other.reg1_bout;
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

constexpr StepCreator reg0_write =
    StepCreator(empty_instruction, true, false, false, false, false);
constexpr StepCreator reg0_bout =
    StepCreator(empty_instruction, false, true, false, false, false);
constexpr StepCreator reg1_write =
    StepCreator(empty_instruction, false, false, true, false, false);
constexpr StepCreator reg1_bout =
    StepCreator(empty_instruction, false, false, false, true, false);

constexpr StepCreator output_flags_selector =
    StepCreator(empty_instruction, false, false, false, false, true);

// IR = [PC]
constexpr step_t universal_step_0 = MEM.bout | PC.aout | IR.write;
// pc cnt6
constexpr step_t universal_step_1 = PC.inc;
constexpr step_t nop = empty_instruction;

// start steps for any instruction that loads an imm16
// WARN: MUST PERFORM PC CNT AFTER
constexpr step_t load_address_procedure[5] = {
    universal_step_0,
    PC.inc,
    MEM.bout | PC.aout | MAR.hi.write, // first byte has msb
    PC.inc,                            // pc cnt
    MEM.bout | PC.aout | MAR.lo.write, // second byte has lsb
};

using template_t = std::array<StepCreator, MAX_NUM_STEPS>;

// clang-format off


// TODO: writting register to itself should be error or nop?
// reg0 = reg1
constexpr template_t mw_template_reg = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | IR2.write, // need to load ir2 to figure out reg1
	reg1_bout | reg0_write | PC.inc | reset, // read from reg1 to reg0, pc cnt
};

// reg = imm8
constexpr template_t mw_template_imm = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | reg0_write, // write the immediate into reg0
	reset | PC.inc, // pc cnt
};

// reg = [MAR]
constexpr template_t lw_template_mar = {
	universal_step_0,
	MEM.bout | MAR.aout | reg0_write | PC.inc | reset, // read from addr MAR into register, pc cnt
};

// reg = [imm16]
constexpr template_t lw_template_imm = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	MEM.bout | MAR.aout | reg0_write | PC.inc | reset, // read from addr MAR into register, pc cnt
};


// reg = [MAR]
constexpr template_t sw_template_mar = {
	universal_step_0,
	MEM.write | MAR.aout | reg0_bout | PC.inc | reset, // read from addr MAR into register, pc cnt
};


// reg = [imm16]
constexpr template_t sw_template_imm = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	MEM.write | MAR.aout | reg0_bout | PC.inc | reset, // read from addr MAR into register, pc cnt
};

// [SP--] = reg
constexpr template_t push_template_reg = {
	universal_step_0,
	MEM.write | SP.aout | reg0_bout | PC.inc, // read from reg into mem at SP addr, pc cnt
	reset | SP.dec,
};

// [SP--] = imm8, overrides A reg
constexpr template_t push_template_imm8 = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | A.write, // write into IR2
	MEM.write | SP.aout | A.bout | PC.inc, // read from IR2 into [SP], pc cnt
	reset | SP.dec, // SP--
};

// reg0 = [SP++]
constexpr template_t pop_template = {
	universal_step_0,
	MEM.bout | SP.aout | reg0_write | PC.inc, // write from [SP] into reg0, pc cnt
	reset | SP.inc, // SP++
};

// MAR = imm16
constexpr template_t mar_template_imm16 = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	reset | PC.inc, // pc cnt
};

// JNZ reg -> PC = MAR if reg != 0 else NOP
constexpr template_t jnz_template_reg = {
	universal_step_0,
	A.write | reg0_bout | PC.inc, // NOTE: pc cnt happens in case jump doesn't happens
	FLAG_WRITE_ALU, // write zero result to flag register
    MAR.hi.bout | PC.hi.write | PC_FLAG_ZERO,
    MAR.lo.bout | PC.lo.write | PC_FLAG_ZERO | reset,
};

// can save instruction if A is already loaded
constexpr template_t jnz_template_reg_A = {
	universal_step_0,
	FLAG_WRITE_ALU | PC.inc, // update flag register
    MAR.hi.bout | PC.hi.write | PC_FLAG_ZERO,
    MAR.lo.bout | PC.lo.write | PC_FLAG_ZERO | reset,
};

// jump if equal flag is carry flag is true
constexpr template_t jmp_imm16_template = {
	load_address_procedure[0],
	load_address_procedure[1],
	load_address_procedure[2],
	load_address_procedure[3],
	load_address_procedure[4],
	PC.inc, // NOTE: pc cnt happens in case jump doesn't happens
    MAR.hi.bout | PC.hi.write | output_flags_selector, // load from mar into pc if flag
    MAR.lo.bout | PC.lo.write | output_flags_selector | reset, // load from mar into pc if flag
};

// jump if equal flag is true
constexpr template_t jmp_mar_template = {
	universal_step_0,
	PC.inc, // NOTE: pc cnt in case jump doesn't happen
    MAR.hi.bout | PC.hi.write | output_flags_selector,
    MAR.lo.bout | PC.lo.write | output_flags_selector | reset,
};

// TODO: all math variants could have faster variants if reg0/reg1 are equal to a/b
// TODO: special case if reg0 = b, reg1 = a (impossible to swap registers without intermediate)

// reg0 = reg0 OP reg1
constexpr template_t math_template_reg = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | IR2.write, // need to load ir2 to figure out reg1
	reg0_bout | A.write | PC.inc, // load reg0 into a
	reg1_bout | B.write, // load reg1 into b
	F.bout | FLAG_WRITE_ALU | reg0_write, // do math op, save to reg0, writes to flag reg
	reset,
};

// reg0 = reg0 OP reg1
constexpr template_t math_template_imm = {
	universal_step_0,
	reg0_bout | A.write | PC.inc, // load reg0 into a first (in case reg0 = b), pc cnt
	MEM.bout | PC.aout | B.write, // load imm into b
	F.bout | FLAG_WRITE_ALU | reg0_write | PC.inc, // save F to reg0, writes to flag reg
	reset,
};

// reg0 = ~reg0
constexpr template_t not_template_none = {
	universal_step_0,
	reg0_bout | A.write | PC.inc, // load reg0 into a, pc cnt
	F.bout | FLAG_WRITE_ALU | reg0_write, // do math op, save to reg0, writes to flag reg
	reset,
};

// reg0 = ~reg1
constexpr template_t not_template_reg = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | IR2.write, // need to load ir2 to figure out reg1
	reg1_bout | A.write | PC.inc, // load reg0 into a
	F.bout | FLAG_WRITE_ALU  | reg0_write, // do math op, save to reg1, writes to flag reg
	reset,
};


// SP dec
constexpr template_t sp_dec_template = {
	universal_step_0,
	PC.inc | SP.dec | reset, // decrement sp
};

// SP inc
constexpr template_t sp_inc_template = {
	universal_step_0,
	PC.inc | SP.inc | reset,
};


// MAR inc
constexpr template_t mar_inc_template = {
	universal_step_0,
	PC.inc | MAR.inc | reset, // increment mar
};

// MAR <- PC
constexpr template_t pc_to_mar_template = {
	universal_step_0,
	universal_step_1,
	PC.hi.bout | MAR.hi.write,
	PC.lo.bout | MAR.lo.write | reset,
};

// MAR <- SP
constexpr template_t sp_to_mar_template = {
	universal_step_0,
	SP.hi.bout | MAR.hi.write | PC.inc,
	SP.lo.bout | MAR.lo.write | reset,
};

// SP <- MAR
constexpr template_t mar_to_sp_template = {
	universal_step_0,
	MAR.hi.bout | SP.hi.write | PC.inc,
	MAR.lo.bout | SP.lo.write | reset,
};

// SP <- imm16
constexpr template_t sp_template_imm16 = {
    universal_step_0,
    PC.inc,
    MEM.bout | PC.aout | SP.hi.write, // write first part of address to sp lo
    PC.inc,                            // pc cnt
    MEM.bout | PC.aout | SP.lo.write, // write second part of address to sp hi
	reset | PC.inc,
};


// reg0 = reg0 OP reg1
constexpr template_t cmp_template_reg = {
	universal_step_0,
	universal_step_1,
	MEM.bout | PC.aout | IR2.write, // need to load ir2 to figure out reg1
	reg0_bout | A.write | PC.inc, // load reg0 into a
	reg1_bout | B.write, // load reg1 into b
	FLAG_WRITE_ALU, // writes to flag reg
	reset,
};

// reg0 = reg0 OP reg1
constexpr template_t cmp_template_imm = {
	universal_step_0,
	reg0_bout | A.write | PC.inc, // load reg0 into a first (in case reg0 = b), pc cnt
	MEM.bout | PC.aout | B.write, // load imm into b
	FLAG_WRITE_ALU | PC.inc,
	reset,
};

// reg0 = keyboard input
constexpr template_t keyboard_template = {
	universal_step_0,
	KEYB.bout | reg0_write | PC.inc | reset,
};


// reg0 = keyboard input
constexpr template_t update_flag_register_template = {
	universal_step_0,
	FLAG_WRITE_ALU | PC.inc,
	reset,
};


constexpr template_t halt_template = {
	universal_step_0,
	halt,
};

// 2 instruction nop
constexpr template_t nop_template = {
	universal_step_0,
	PC.inc | reset,
};

constexpr template_t vram_read_template_no_delay = {
	universal_step_0,
	VRAM.bout | MAR.aout, // NOTE: must add register write manually
	nop,
	PC.inc | reset,
};

constexpr template_t vram_read_template_delay = {
	universal_step_0,
	nop,
	VRAM.bout | MAR.aout, // NOTE: must add register write manually
	PC.inc | reset,
};


constexpr template_t vram_write_template = {
	universal_step_0,
	VRAM.write | MAR.aout,
	VRAM.write | MAR.aout, // NOTE: must add register bout manually
	PC.inc | reset, /// TODO: can add MAR.cnt
};

// clang-format on
step_t *get_ucode_ptr(const split_addr_t *instruction) {
  return &ucode[instruction->instruction][instruction->step][instruction->imm]
               [instruction->reg0][instruction->reg1]
               [instruction->ir2_extra_bits][instruction->not_vram_active];
}

void setStep(const StepCreator &step, const split_addr_t *instruction) {
  step_t *curr = get_ucode_ptr(instruction);
  StepCreator template_step = step;
  *curr = template_step.getStep();
}

// TODO: determine if will be implemented
void setError(const split_addr_t *instruction) {
  step_t *curr = get_ucode_ptr(instruction);
  *curr = error;
}

void create_instruction(const template_t *step_template,
                        const split_addr_t *instruction) {
  step_t *curr = get_ucode_ptr(instruction);
  StepCreator template_step = step_template->at(instruction->step);
  template_step.setRegisters(instruction);
  *curr = template_step.getStep();
}

void create_instruction(const template_t *step_template_reg,
                        const template_t *step_template_imm,
                        const split_addr_t *instruction) {
  const template_t *step_template = nullptr;

  if (instruction->imm == 0) {
    step_template = step_template_reg;
  } else {
    step_template = step_template_imm;
  }

  create_instruction(step_template, instruction);
}

void mw_instruction(const split_addr_t *instruction) {
  create_instruction(&mw_template_reg, &mw_template_imm, instruction);
}

void lw_instruction(const split_addr_t *instruction) {
  create_instruction(&lw_template_mar, &lw_template_imm, instruction);
}

void sw_instruction(const split_addr_t *instruction) {
  if (std::string name = intToRegister(instruction->reg0).name;
      name == "MAR_lo" || name == "MAR_hi") {
    // can't implement loading to either mar with this instruction, makes more
    // sense to do so with lda
    setError(instruction);
    return;
  }
  create_instruction(&sw_template_mar, &sw_template_imm, instruction);
}

void push_special_instruction(const split_addr_t *instruction) {
  if (instruction->imm == 1) {
    // imm push takes one instruction, the other possible 7 are used for
    // special functions

    const template_t *templates[8] = {
        &push_template_imm8, // push imm8
        &sp_inc_template,    &mar_inc_template,   &sp_dec_template,
        &pc_to_mar_template, &sp_to_mar_template, &mar_template_imm16,
        &sp_template_imm16,
    };

    create_instruction(templates[instruction->reg0], instruction);
  } else {
    if (std::string name = intToRegister(instruction->reg0).name;
        name == "MAR_lo" || name == "MAR_hi") {
      // can't implement since bus is taken
      setError(instruction);
      return;
    }
    create_instruction(&push_template_reg, instruction);
  }
}

void vram_read_instruction(const split_addr_t *instruction,
                           const reg_t *output_register) {
  const template_t *curr_template = nullptr;
  bool delayed;
  if (instruction->step == 1 && instruction->not_vram_active == 0) {
    // vram active right now, use no delay version
    curr_template = &vram_read_template_no_delay;
    delayed = false;
  } else {
    // have to wait one cycle, nop version instead
    curr_template = &vram_read_template_delay;
    delayed = true;
  }

  // create_instruction(curr_template, instruction);

  step_t *curr = get_ucode_ptr(instruction);
  StepCreator template_step = curr_template->at(instruction->step);
  if (instruction->step == 1 && !delayed)
    template_step |= output_register->write;
  if (instruction->step == 2 && delayed)
    template_step |= output_register->write;

  *curr = template_step.getStep();
}

void vram_write_instruction(const split_addr_t *instruction,
                            const reg_t *output_register) {
  const template_t *curr_template = &vram_write_template;

  step_t *curr = get_ucode_ptr(instruction);
  StepCreator template_step = curr_template->at(instruction->step);

  if (instruction->step == 1 || instruction->step == 2)
    template_step |= output_register->bout;

  *curr = template_step.getStep();
}

void pop_instruction(const split_addr_t *instruction) {
  if (instruction->imm == 0) {
    create_instruction(&pop_template, instruction);
  } else {
    if (instruction->reg0 == 3) {
      vram_read_instruction(instruction, &Z);
      return;
    }
    if (instruction->reg0 == 4) {
      vram_write_instruction(instruction, &Z);
      return;
    }
    if (instruction->reg0 == 5) {
      vram_read_instruction(instruction, &Y);
      return;
    }
    if (instruction->reg0 == 6) {
      vram_write_instruction(instruction, &Y);
      return;
    }

    // more special purpose instructions
    const template_t *templates[8] = {
        &mar_to_sp_template, // push imm8
        &update_flag_register_template,
        &nop_template,
        &nop_template, // Z = vram[MAR]
        &nop_template, // vram[MAR] = Z
        &nop_template, // Y = vram[MAR]
        &nop_template, // vram[MAR] = Y
        &halt_template,
    };

    create_instruction(templates[instruction->reg0], instruction);
  }
}

void jmp_instruction(const split_addr_t *instruction) {
  const template_t *step_template = nullptr;

  // jump if reg != 0
  if (instruction->imm == 0) {
    // special case -> can save step if already A
    if (instruction->reg0 == 0) {
      step_template = &jnz_template_reg_A;
    } else {
      // general case -> writing to A from reg0
      step_template = &jnz_template_reg;
    }
    create_instruction(step_template, instruction);
  } else {
    uint8_t flag_idx = instruction->reg0 & 0b011;
    uint8_t using_imm16_flag = instruction->reg0 & 0b100;
    const static step_t idx_to_flag[] = {PC_FLAG_DIRECT, PC_FLAG_CARRY,
                                         PC_FLAG_EQ, PC_FLAG_ZERO};
    step_t flag_step_bits = idx_to_flag[flag_idx];

    // TODO: unconditional jump could save one step by skipping pc cnt
    if (using_imm16_flag) {
      step_template = &jmp_imm16_template;
    } else {
      step_template = &jmp_mar_template;
    }

    StepCreator template_step = step_template->at(instruction->step);
    template_step.setRegisters(instruction);
    template_step.setFlag(flag_step_bits);
    setStep(template_step, instruction);
  }
}

void math_instruction(const split_addr_t *instruction) {
  create_instruction(&math_template_reg, &math_template_imm, instruction);
}

void not_instruction(const split_addr_t *instruction) {
  create_instruction(&not_template_reg, &not_template_none, instruction);
}

void cmp_instruction(const split_addr_t *instruction) {
  create_instruction(&cmp_template_reg, &cmp_template_imm, instruction);
}

void keyb_other_instruction(const split_addr_t *instruction) {
  create_instruction(&keyboard_template, &keyboard_template, instruction);
}

using instruction_func = std::function<void(const split_addr_t *)>;

instruction_func instructions_table[16] = {
    mw_instruction,         cmp_instruction,          // 0, 1
    sw_instruction,         push_special_instruction, // 2, 3
    pop_instruction,        jmp_instruction,          // 4, 5
    keyb_other_instruction, lw_instruction,           // 6, 7
    math_instruction,       math_instruction,         // sub, sbc
    math_instruction,       math_instruction,         // add, adc
    not_instruction,        math_instruction,         // not, xor
    math_instruction,       math_instruction,         // or, and
};

// returns 1 << y_bit if the x_bit bit of x is on
uint32_t bitTransform(uint32_t x, uint32_t x_bit, uint32_t y_bit) {
  return (x & (1 << x_bit)) ? (1 << y_bit) : 0;
}

void rom_addr_to_instruction(uint32_t addr, split_addr_t *instruction_ptr) {
  uint32_t not_vram_active = bitTransform(addr, 0, 0);

  uint32_t STEP = bitTransform(addr, 13, 0) | bitTransform(addr, 14, 1) |
                  bitTransform(addr, 15, 2) | bitTransform(addr, 16, 3);

  // lower half
  uint32_t IR = bitTransform(addr, 5, 3) | bitTransform(addr, 6, 2) |
                bitTransform(addr, 7, 1) | bitTransform(addr, 12, 0) |
                // upper half
                bitTransform(addr, 8, 4) | bitTransform(addr, 9, 5) |
                bitTransform(addr, 11, 6) | bitTransform(addr, 10, 7);

  uint32_t IR2 = bitTransform(addr, 1, 3) | bitTransform(addr, 2, 2) |
                 bitTransform(addr, 3, 1) | bitTransform(addr, 4, 0);

  // clang-format off
  instruction_ptr->step           = STEP;
  instruction_ptr->reg0           = (IR & 0b00000111);
  instruction_ptr->imm            = (IR & 0b00001000) >> 3;
  instruction_ptr->instruction    = (IR & 0b11110000) >> 4;
  instruction_ptr->reg1           = (IR2 & 0b0111);
  instruction_ptr->ir2_extra_bits = (IR2 & 0b1000) >> 3;
  instruction_ptr->not_vram_active = not_vram_active;
  instruction_ptr->original_address = addr;
  // clang-format on
}

void addr_to_instruction(uint32_t addr, split_addr_t *instruction_ptr) {
  rom_addr_to_instruction(addr, instruction_ptr);
}

void process_address(uint32_t addr) {
  split_addr_t instruction;
  addr_to_instruction(addr, &instruction);

  // move word
  instruction_func istr_func = instructions_table[instruction.instruction];
  if (istr_func) {
    istr_func(&instruction);
  } else {
    setError(&instruction);
  }
}

void populate_ucode() {
  for (uint32_t addr = 0; addr <= MAX_ADDR_INC; addr++) {
    // split_addr_t instruction;
    // addr_to_instruction(addr, &instruction);
    // std::cout << "step/reg0/imm/istr/reg1 " << int(instruction.step) << " "
    //           << int(instruction.reg0) << " " << int(instruction.imm) << "
    //           "
    //           << int(instruction.instruction) << " " <<
    //           int(instruction.reg1)
    //           << std::endl;

    process_address(addr);
  }
}

step_t getInstruction(int addr) {
  split_addr_t instruction;
  addr_to_instruction(addr, &instruction);
  return *get_ucode_ptr(&instruction);
}

void write_ucode_roms_logisim() {
  // Open a file in append mode
  FILE *rom0 = fopen("rom_images/rom0.img", "wb");
  FILE *rom1 = fopen("rom_images/rom1.img", "wb");

  fprintf(rom0, "v3.0 hex words plain\n");
  fprintf(rom1, "v3.0 hex words plain\n");

  for (int addr = 0; addr <= MAX_ADDR_INC; addr++) {
    step_t curr = getInstruction(addr);
    uint16_t data = curr.getRomData();
    uint16_t rom0_data = data & 0xff;
    uint16_t rom1_data = (data & 0xff00) >> 8;

    fprintf(rom0, "%02X", rom0_data);
    fprintf(rom1, "%02X", rom1_data);

    if (addr % 16 == 15) {
      fprintf(rom0, "\n");
      fprintf(rom1, "\n");
    } else {
      fprintf(rom0, " ");
      fprintf(rom1, " ");
    }
  }
  // Close the file
  fclose(rom0);
  fclose(rom1);
}

// void write_ucode_logism() {
//   printf("v3.0 hex words plain\n");
//   for (int addr = 0; addr <= MAX_ADDR_INC; addr++) {
//     uint16_t curr = getInstruction(addr).getRomData();
//     printf("%04X", curr);
//     if (addr % 16 == 15) {
//       printf("\n");
//     } else {
//       printf(" ");
//     }
//   }
// }

void logStep(const step_t &step) { std::cout << step.toString() << std::endl; }
void logInstruction(const split_addr_t &instruction) {
  std::string str = "step: ";
  str += std::to_string(instruction.step) + ", istr: ";
  str += std::to_string(instruction.instruction) + ", imm: ";
  str += std::to_string(instruction.imm) + ", ir2_extra_bits: ";
  str += std::to_string(instruction.ir2_extra_bits) + ", reg0: ";
  str += std::to_string(instruction.reg0) + ", reg1: ";
  str += std::to_string(instruction.reg1) + ", not vram active: ";
  str += std::to_string(instruction.not_vram_active);
  std::cout << str << std::endl;
}

void interactive_address_lookup() {
  while (true) {
    std::cout << "Enter address: ";

    std::string str_addr;
    std::cin >> str_addr;

    uint32_t addr = std::stoul(str_addr, nullptr, 16);

    step_t curr_step = getInstruction(addr);

    logStep(curr_step);

    split_addr_t instruction;
    addr_to_instruction(addr, &instruction);
    logInstruction(instruction);
  }
}

int main(int argc, char *argv[]) {
  populate_ucode();
  write_ucode_roms_logisim();
  std::cout << "wrote rom images" << std::endl;

  bool interactive_enabled = false;

  for (int i = 1; i < argc; i++) {
    if (std::string(argv[i]) == "--interactive") {
      interactive_enabled = true;
    }
  }

  if (interactive_enabled) {
    interactive_address_lookup();
  }
}
