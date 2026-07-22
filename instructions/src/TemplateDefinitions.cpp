#include "TemplateDefinitions.hpp"
#include "Opcode.hpp"
#include "TemplateGenerators.hpp"
#include "Templates.hpp"
#include <memory>
#include <string>
#include <vector>

class readWriteStructure {
public:
  Action bout;
  Action write;
  std::string name;
};
bool operator==(const readWriteStructure &lhs, const readWriteStructure &rhs) {
  return lhs.name == rhs.name;
}

readWriteStructure a_reg{a::bout, a::write, "a"};
readWriteStructure b_reg{b::bout, b::write, "b"};
readWriteStructure x_reg{x::bout, x::write, "x"};
readWriteStructure y_reg{y::bout, y::write, "y"};
readWriteStructure z_reg{z::bout, z::write, "z"};
readWriteStructure mar_lo_reg{mar::lo::bout, mar::lo::write, "mar_lo"};
readWriteStructure mar_hi_reg{mar::hi::bout, mar::hi::write, "mar_hi"};
readWriteStructure sp_lo_reg{sp::lo::bout, sp::lo::write, "sp_lo"};
readWriteStructure sp_hi_reg{sp::hi::bout, sp::hi::write, "sp_hi"};
readWriteStructure pc_lo_reg{pc::lo::bout, pc::lo::write, "pc_lo"};
readWriteStructure pc_hi_reg{pc::hi::bout, pc::hi::write, "pc_hi"};
readWriteStructure keyb_reg{keyb_bout, nop, "keyb"};
readWriteStructure flags_reg{flags::bout, nop, "flags"};

IstrTemplateType copyTemplate(const IstrTemplateType &temp) { return temp; }

void setReg0(IstrTemplateType &temp, Action bout_reg0, Action write_reg0) {
  for (StepTemplateType &step : temp) {
    for (Action &action : step) {
      if (action == reg0_bout)
        action = bout_reg0;
      if (action == reg0_write)
        action = write_reg0;
    }
  }
}

void setReg1(IstrTemplateType &temp, Action bout_reg1, Action write_reg1) {
  for (StepTemplateType &step : temp) {
    for (Action &action : step) {
      if (action == reg1_bout)
        action = bout_reg1;
      if (action == reg1_write)
        action = write_reg1;
    }
  }
}

void setSelectedFlag(IstrTemplateType &temp, Action selected_flag) {
  for (StepTemplateType &step : temp) {
    for (Action &action : step) {
      if (action == output_flags_selector)
        action = selected_flag;
    }
  }
}

IstrTemplateType fillTemplate(const IstrTemplateType &temp,
                              const readWriteStructure &reg0) {
  IstrTemplateType result = copyTemplate(temp);
  setReg0(result, reg0.bout, reg0.write);
  return result;
}

IstrTemplateType fillTemplate(const IstrTemplateType &temp,
                              const readWriteStructure &reg0,
                              const readWriteStructure &reg1) {
  IstrTemplateType result = copyTemplate(temp);
  setReg0(result, reg0.bout, reg0.write);
  setReg1(result, reg1.bout, reg1.write);
  return result;
}

IstrTemplateType fillTemplate(const IstrTemplateType &temp,
                              const readWriteStructure &reg0,
                              const readWriteStructure &reg1,
                              Action selected_flag) {
  IstrTemplateType result = copyTemplate(temp);
  setReg0(result, reg0.bout, reg0.write);
  setReg1(result, reg1.bout, reg1.write);
  setSelectedFlag(result, selected_flag);
  return result;
}

// IR = [PC]
StepTemplateType universal_step_0 = {mem::read, pc::addr, ir::write};

// pc cnt6
StepTemplateType universal_step_1 = {pc::cnt};
StepTemplateType loadIr2 = {mem::read, pc::addr, ir2::write};

// start steps for any instruction that loads an imm16
// warn: must perform pc cnt after
IstrTemplateType load_address_procedure = {
    universal_step_0,
    {pc::cnt},
    {mem::read, pc::addr, mar::hi::write}, // first byte has msb
    {pc::cnt},                             // pc cnt
    {mem::read, pc::addr, mar::lo::write}, // second byte has lsb
};

// move register to register (reg0 = reg1)
void addMoveWordRegInstructions(
    std::vector<std::unique_ptr<InstructionWrapper>> &dest) {
  static const IstrTemplateType base_template = {
      universal_step_0,
      universal_step_1,
      loadIr2,                                 // load ir 2
      {reg1_bout, reg0_write, pc::cnt, reset}, // read from reg1 to reg0, pc cnt
  };

  std::vector<readWriteStructure> lhs = {
      a_reg,      b_reg,     x_reg,     y_reg,     z_reg,    mar_lo_reg,
      mar_hi_reg, pc_lo_reg, pc_hi_reg, sp_lo_reg, sp_hi_reg};

  std::vector<readWriteStructure> rhs = {
      a_reg,      b_reg,      x_reg,     y_reg,     z_reg,
      mar_lo_reg, mar_hi_reg, pc_lo_reg, pc_hi_reg, sp_lo_reg,
      sp_hi_reg,  flags_reg,  keyb_reg};

  for (const readWriteStructure &l : lhs) {
    for (const readWriteStructure &r : rhs) {
      // avoid duplicates
      if (l == r)
        continue;
      IstrTemplateType current = copyTemplate(base_template);
      fillTemplate(current, l, r);

      Instruction istr(current, "mv " + l.name + ", " + r.name);
      dest.push_back(std::make_unique<InstructionWrapper>(istr));
    }
  }
}

// reg = imm8
IstrTemplateType mw_template_imm = {
    universal_step_0,
    universal_step_1,
    {mem::read, pc::addr, reg0_write}, // write the immediate into reg0
    {reset, pc::cnt},                  // pc cnt
};

void mw_instruction_imm(std::vector<std::unique_ptr<Instruction>> &dest) {
  const IstrTemplateType &base_template = mw_template_imm;

  std::vector<readWriteStructure> lhs = {
      a_reg,      b_reg,     x_reg,     y_reg,     z_reg,    mar_lo_reg,
      mar_hi_reg, pc_lo_reg, pc_hi_reg, sp_lo_reg, sp_hi_reg};

  for (const auto &l : lhs) {
    auto curr_temp = copyTemplate(base_template);
    fillTemplate(curr_temp, l);
    std::string name = "";
    dest.push_back(
        std::make_unique<Instruction>(curr_temp, "mv " + l.name + ", imm8"));
  }
}

// reg = [mar]
IstrTemplateType lw_template_mar = {
    universal_step_0,
    {mem::read, mar::addr, reg0_write, pc::cnt,
     reset}, // read from addr mar into register, pc cnt
};

// reg = [imm16]
IstrTemplateType lw_template_imm = {
    load_address_procedure[0],
    load_address_procedure[1],
    load_address_procedure[2],
    load_address_procedure[3],
    load_address_procedure[4],
    {mem::read, mar::addr, reg0_write, pc::cnt,
     reset}, // read from addr mar into register, pc cnt
};

// reg = [mar]
IstrTemplateType sw_template_mar = {
    universal_step_0,
    {mem::write, mar::addr, reg0_bout, pc::cnt,
     reset}, // read from addr mar into register, pc cnt
};

// reg = [imm16]
IstrTemplateType sw_template_imm = {
    load_address_procedure[0],
    load_address_procedure[1],
    load_address_procedure[2],
    load_address_procedure[3],
    load_address_procedure[4],
    {mem::write, mar::addr, reg0_bout, pc::cnt,
     reset}, // read from addr mar into register, pc cnt
};

// [sp--] = reg
IstrTemplateType push_template_reg = {
    universal_step_0,
    {pc::cnt, sp::dec}, // decrement before pushing value
    {mem::write, sp::addr, reg0_bout,
     reset}, // read from reg into mem at sp addr, pc cnt
};

// [sp--] = imm8, overrides a reg
IstrTemplateType push_template_imm8 = {
    universal_step_0,
    {pc::cnt, sp::dec},              // decrement before pushing value
    {mem::read, pc::addr, a::write}, // write into ir2
    {mem::write, sp::addr, a::bout, pc::cnt,
     reset}, // read from ir2 into [sp], pc cnt
};

// reg0 = [sp++]
IstrTemplateType pop_template = {
    universal_step_0,
    {mem::read, sp::addr, reg0_write,
     pc::cnt},        // write from [sp] into reg0, pc cnt
    {reset, sp::inc}, // cntrement after popping value
};

// mar = imm16
IstrTemplateType mar_template_imm16 = {
    load_address_procedure[0], load_address_procedure[1],
    load_address_procedure[2], load_address_procedure[3],
    load_address_procedure[4], {reset, pc::cnt}, // pc cnt
};

// jnz reg -> pc = mar if reg != 0 else nop
IstrTemplateType jnz_template_reg = {
    universal_step_0,
    {a::write, reg0_bout,
     pc::cnt},          // note: pc cnt happens in case jump doesn't happens
    {flags::alu_write}, // write zero result to flag register
    {mar::hi::bout, pc::hi::write, flags::select::zero},
    {mar::lo::bout, pc::lo::write, flags::select::zero, reset},
};

// can save instruction if a is already loaded
IstrTemplateType jnz_template_reg_a = {
    universal_step_0,
    {flags::alu_write, pc::cnt}, // update flag register
    {mar::hi::bout, pc::hi::write, flags::select::zero},
    {mar::lo::bout, pc::lo::write, flags::select::zero, reset},
};

// jump if equal flag is carry flag is true
IstrTemplateType jmp_imm16_template = {
    load_address_procedure[0],
    load_address_procedure[1],
    load_address_procedure[2],
    load_address_procedure[3],
    load_address_procedure[4],
    {pc::cnt}, // note: pc cnt happens in case jump doesn't happens
    {mar::hi::bout, pc::hi::write,
     output_flags_selector}, // load from mar into pc if flag
    {mar::lo::bout, pc::lo::write, output_flags_selector,
     reset}, // load from mar into pc if flag
};

// jump if equal flag is true
IstrTemplateType jmp_mar_template = {
    universal_step_0,
    {pc::cnt}, // note: pc cnt in case jump doesn't happen
    {mar::hi::bout, pc::hi::write, output_flags_selector},
    {mar::lo::bout, pc::lo::write, output_flags_selector, reset},
};

// todo: all math variants could have faster variants if reg0/reg1 are equal to
// a/b todo: special case if reg0 = b, reg1 = a (impossible to swap registers
// without intermediate)

// reg0 = reg0 op reg1
IstrTemplateType math_carry_template_reg = {
    universal_step_0,
    universal_step_1,
    {mem::read, pc::addr, ir2::write}, // need to load ir2 to figure out reg1
    {reg0_bout, a::write, pc::cnt},    // load reg0 into a
    {reg1_bout, b::write},             // load reg1 into b
    {f_alu_bout, flags::alu_write, reg0_write,
     flags::select::carry}, // do math op, save to reg0, writes to flag reg
    {reset},
};

IstrTemplateType math_no_carry_template_reg = {
    universal_step_0,
    universal_step_1,
    {mem::read, pc::addr, ir2::write}, // need to load ir2 to figure out reg1
    {reg0_bout, a::write, pc::cnt},    // load reg0 into a
    {reg1_bout, b::write},             // load reg1 into b
    {f_alu_bout, flags::alu_write, reg0_write,
     flags::select::direct}, // do math op, save to reg0, writes to flag reg
    {reset},
};

// reg0 = reg0 op reg1
IstrTemplateType math_carry_template_imm = {
    universal_step_0,
    {reg0_bout, a::write,
     pc::cnt}, // load reg0 into a first (in case reg0 = b), pc cnt
    {mem::read, pc::addr, b::write}, // load imm into b
    {f_alu_bout, flag_write_alu, reg0_write, flags::select::carry,
     pc::cnt}, // save f to reg0, writes to flag reg
    {reset},
};

IstrTemplateType math_no_carry_template_imm = {
    universal_step_0,
    {reg0_bout, a::write,
     pc::cnt}, // load reg0 into a first (in case reg0 = b), pc cnt
    {mem::read, pc::addr, b::write}, // load imm into b
    {f_alu_bout, flag_write_alu, reg0_write, flags::select::direct,
     pc::cnt}, // save f to reg0, writes to flag reg
    {reset},
};

// reg0 = ~reg0
IstrTemplateType not_template_none = {
    universal_step_0,
    {reg0_bout, a::write, pc::cnt}, // load reg0 into a, pc cnt
    {f_alu_bout, flag_write_alu,
     reg0_write}, // do math op, save to reg0, writes to flag reg
    {reset},
};

// reg0 = ~reg1
IstrTemplateType not_template_reg = {
    universal_step_0,
    universal_step_1,
    {mem::read, pc::addr, ir2::write}, // need to load ir2 to figure out reg1
    {reg1_bout, a::write, pc::cnt},    // load reg0 into a
    {f_alu_bout, flag_write_alu,
     reg0_write}, // do math op, save to reg1, writes to flag reg
    {reset},
};

// sp dec
IstrTemplateType sp_dec_template = {
    universal_step_0,
    {pc::cnt, sp::dec, reset}, // decrement sp
};

// sp cnt
IstrTemplateType sp_inc_template = {
    universal_step_0,
    {pc::cnt, sp::inc, reset},
};

// mar cnt
IstrTemplateType mar_cnt_template = {
    universal_step_0,
    {pc::cnt, mar::cnt, reset}, // cntrement mar
};

// mar <- pc
IstrTemplateType pc_to_mar_template = {
    universal_step_0,
    universal_step_1,
    {pc::hi::bout, mar::hi::write},
    {pc::lo::bout, mar::lo::write, reset},
};

// mar <- sp
IstrTemplateType sp_to_mar_template = {
    universal_step_0,
    {sp::hi::bout, mar::hi::write, pc::cnt},
    {sp::lo::bout, mar::lo::write, reset},
};

// sp <- mar
IstrTemplateType mar_to_sp_template = {
    universal_step_0,
    {mar::hi::bout, sp::hi::write, pc::cnt},
    {mar::lo::bout, sp::lo::write, reset},
};

// sp <- imm16
IstrTemplateType sp_template_imm16 = {
    universal_step_0,
    {pc::cnt},
    {mem::read, pc::addr,
     sp::hi::write}, // write first part of address to sp lo
    {pc::cnt},       // pc cnt
    {mem::read, pc::addr,
     sp::lo::write}, // write second part of address to sp hi
    {reset, pc::cnt},
};

// reg0 = reg0 op reg1
IstrTemplateType cmp_template_reg = {
    universal_step_0,
    universal_step_1,
    {mem::read, pc::addr, ir2::write}, // need to load ir2 to figure out reg1
    {reg0_bout, a::write, pc::cnt},    // load reg0 into a
    {reg1_bout, b::write},             // load reg1 into b
    {flags::alu_write},                // writes to flag reg
    {reset},
};

// reg0 = reg0 op reg1
IstrTemplateType cmp_template_imm = {
    universal_step_0,
    {reg0_bout, a::write,
     pc::cnt}, // load reg0 into a first (in case reg0 = b), pc cnt
    {mem::read, pc::addr, b::write}, // load imm into b
    {flags::alu_write, pc::cnt},
    {reset},
};

// reg0 = keyboard input
IstrTemplateType keyboard_template = {
    universal_step_0,
    {keyb_bout, reg0_write, pc::cnt, reset},
};

// reg0 = keyboard input
IstrTemplateType update_flag_register_template = {
    universal_step_0,
    {flag_write_alu, pc::cnt},
    {reset},
};

IstrTemplateType halt_template = {
    universal_step_0,
    {halt},
};

// 2 instruction nop
IstrTemplateType nop_template = {
    universal_step_0,
    {pc::cnt, reset},
};

IstrTemplateType vram_read_template_no_delay = {
    universal_step_0,
    {vram::read, mar::addr}, // note: must add register write manually
    {nop},
    {pc::cnt, reset},
};

IstrTemplateType vram_read_template_delay = {
    universal_step_0,
    {nop},
    {vram::read, mar::addr}, // note: must add register write manually
    {pc::cnt, reset},
};

IstrTemplateType vram_write_template = {
    universal_step_0,
    {vram::write, mar::addr},
    {vram::write, mar::addr}, // note: must add register bout manually
    {pc::cnt, reset},         /// todo: can add mar::cnt
};

// global instance of the instruction set generated by the function below
static InstructionSet istr_set;
// void mw_instruction() {
//   create_instruction(&mw_template_reg, &mw_template_imm);
// }
//
// void lw_instruction(const split_addr_t *instruction) {
//   create_instruction(&lw_template_mar, &lw_template_imm, instruction);
// }
//
// void sw_instruction(const split_addr_t *instruction) {
//   if (std::string name = intToRegister(instruction->reg0).name;
//       name == "MAR_lo" || name == "MAR_hi") {
//     // can't implement loading to either mar with this instruction, makes
//     more
//     // sense to do so with lda
//     setError(instruction);
//     return;
//   }
//   create_instruction(&sw_template_mar, &sw_template_imm, instruction);
// }
//
// void push_special_instruction(const split_addr_t *instruction) {
//   if (instruction->imm == 1) {
//     // imm push takes one instruction, the other possible 7 are used for
//     // special functions
//
//     const template_t *templates[8] = {
//         &push_template_imm8, // push imm8
//         &sp_inc_template,    &mar_inc_template,   &sp_dec_template,
//         &pc_to_mar_template, &sp_to_mar_template, &mar_template_imm16,
//         &sp_template_imm16,
//     };
//
//     create_instruction(templates[instruction->reg0], instruction);
//   } else {
//     if (std::string name = intToRegister(instruction->reg0).name;
//         name == "MAR_lo" || name == "MAR_hi") {
//       // can't implement since bus is taken
//       setError(instruction);
//       return;
//     }
//     create_instruction(&push_template_reg, instruction);
//   }
// }
//
// void vram_read_instruction(const split_addr_t *instruction,
//                            const reg_t *output_register) {
//   const template_t *curr_template = nullptr;
//   bool delayed;
//   if (instruction->step == 1 && instruction->not_vram_active == 0) {
//     // vram active right now, use no delay version
//     curr_template = &vram_read_template_no_delay;
//     delayed = false;
//   } else {
//     // have to wait one cycle, nop version instead
//     curr_template = &vram_read_template_delay;
//     delayed = true;
//   }
//
//   // create_instruction(curr_template, instruction);
//
//   step_t *curr = get_ucode_ptr(instruction);
//   StepCreator template_step = curr_template->at(instruction->step);
//   if (instruction->step == 1 && !delayed)
//     template_step |= output_register->write;
//   if (instruction->step == 2 && delayed)
//     template_step |= output_register->write;
//
//   *curr = template_step.getStep();
// }
//
// void vram_write_instruction(const split_addr_t *instruction,
//                             const reg_t *output_register) {
//   const template_t *curr_template = &vram_write_template;
//
//   step_t *curr = get_ucode_ptr(instruction);
//   StepCreator template_step = curr_template->at(instruction->step);
//
//   if (instruction->step == 1 || instruction->step == 2)
//     template_step |= output_register->bout;
//
//   *curr = template_step.getStep();
// }
//
// void pop_instruction(const split_addr_t *instruction) {
//   if (instruction->imm == 0) {
//     create_instruction(&pop_template, instruction);
//   } else {
//     if (instruction->reg0 == 3) {
//       vram_read_instruction(instruction, &Z);
//       return;
//     }
//     if (instruction->reg0 == 4) {
//       vram_write_instruction(instruction, &Z);
//       return;
//     }
//     if (instruction->reg0 == 5) {
//       vram_read_instruction(instruction, &Y);
//       return;
//     }
//     if (instruction->reg0 == 6) {
//       vram_write_instruction(instruction, &Y);
//       return;
//     }
//
//     // more special purpose instructions
//     const template_t *templates[8] = {
//         &mar_to_sp_template, &update_flag_register_template, &nop_template,
//         &nop_template, // Z = vram[MAR]
//         &nop_template, // vram[MAR] = Z
//         &nop_template, // Y = vram[MAR]
//         &nop_template, // vram[MAR] = Y
//         &halt_template,
//     };
//
//     create_instruction(templates[instruction->reg0], instruction);
//   }
// }
//
// void jmp_instruction(const split_addr_t *instruction) {
//   const template_t *step_template = nullptr;
//
//   // jump if reg != 0
//   if (instruction->imm == 0) {
//     // special case -> can save step if already A
//     if (instruction->reg0 == 0) {
//       step_template = &jnz_template_reg_A;
//     } else {
//       // general case -> writing to A from reg0
//       step_template = &jnz_template_reg;
//     }
//     create_instruction(step_template, instruction);
//   } else {
//     uint8_t flag_idx = instruction->reg0 & 0b011;
//     uint8_t using_imm16_flag = instruction->reg0 & 0b100;
//     const static step_t idx_to_flag[] = {PC_FLAG_DIRECT, PC_FLAG_CARRY,
//                                          PC_FLAG_EQ, PC_FLAG_ZERO};
//     step_t flag_step_bits = idx_to_flag[flag_idx];
//
//     // TODO: unconditional jump could save one step by skipping pc cnt
//     if (using_imm16_flag) {
//       step_template = &jmp_imm16_template;
//     } else {
//       step_template = &jmp_mar_template;
//     }
//
//     StepCreator template_step = step_template->at(instruction->step);
//     template_step.setRegisters(instruction);
//     template_step.setFlag(flag_step_bits);
//     setStep(template_step, instruction);
//   }
// }
//
// void math_instruction(const split_addr_t *instruction) {
//   const template_t *reg_template = nullptr;
//   const template_t *imm_template = nullptr;
//
//   if (instruction->instruction & 1) {
//     reg_template = &math_no_carry_template_reg;
//     imm_template = &math_no_carry_template_imm;
//   } else {
//     reg_template = &math_carry_template_reg;
//     imm_template = &math_carry_template_imm;
//   }
//
//   create_instruction(reg_template, imm_template, instruction);
// }
//
// void not_instruction(const split_addr_t *instruction) {
//   create_instruction(&not_template_reg, &not_template_none, instruction);
// }
//
// void cmp_instruction(const split_addr_t *instruction) {
//   create_instruction(&cmp_template_reg, &cmp_template_imm, instruction);
// }
//
// void keyb_other_instruction(const split_addr_t *instruction) {
//   create_instruction(&keyboard_template, &keyboard_template, instruction);
// }
//
// using instruction_func = std::function<void(const split_addr_t *)>;
//
// instruction_func instructions_table[16] = {
//     mw_instruction,         cmp_instruction,          // 0, 1
//     sw_instruction,         push_special_instruction, // 2, 3
//     pop_instruction,        jmp_instruction,          // 4, 5
//     keyb_other_instruction, lw_instruction,           // 6, 7
//     math_instruction,       math_instruction,         // sub, sbc
//     math_instruction,       math_instruction,         // add, adc
//     not_instruction,        math_instruction,         // not, xor
//     math_instruction,       math_instruction,         // or, and
// };

void instantiateTemplates() {
  // TODO: implement all the different templates
  size_t istr_idx = 0;

  // TODO: make functions to fill placeholders in templates and automatically
  // create all permutations of functions

  fillTemplate(mw_template_imm, a_reg);
  fillTemplate(mw_template_imm, a_reg, b_reg);
  fillTemplate(mw_template_imm, a_reg, b_reg, flags::select::carry);

  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(mw_template_imm);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(lw_template_mar);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(lw_template_imm);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(sw_template_mar);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(sw_template_imm);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(push_template_reg);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(push_template_imm8);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(pop_template);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(mar_template_imm16);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(jnz_template_reg);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(jnz_template_reg_a);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(jmp_imm16_template);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(jmp_mar_template);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(math_carry_template_reg);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(math_no_carry_template_reg);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(math_carry_template_imm);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(math_no_carry_template_imm);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(not_template_none);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(not_template_reg);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(sp_dec_template);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(sp_inc_template);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(mar_cnt_template);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(pc_to_mar_template);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(sp_to_mar_template);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(mar_to_sp_template);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(sp_template_imm16);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(cmp_template_reg);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(cmp_template_imm);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(keyboard_template);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(update_flag_register_template);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(halt_template);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(nop_template);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(vram_read_template_no_delay);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(vram_read_template_delay);
  // istr_set.instructions[istr_idx++] =
  //     std::make_unique<Instruction>(vram_write_template);
}

IstrTemplateType opcodeToTemplate(const Opcode &opcode) {
  return istr_set.opcodeToInstruction(opcode).getTemplate();
}
