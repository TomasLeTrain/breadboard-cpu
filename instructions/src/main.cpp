#include <cassert>
#include <cstdint>
#include <iostream>
#include <map>
#include <string>
#include <vector>

#include "Action.h"
#include "CategoryData.h"
#include "OutputCategory.h"

// effectively any output wire is represented as one action type
// TODO: add special actions that do not actually exist, but signal special
// flags that get used to fill info in?
// or use variants instead to avoid polluting the enum
enum ActionType {
  // TODO: halt = 0 so the default action is to halt if nothing else is
  // specified?
  halt = 0,
  pc_cnt,
  mar_cnt,
  sp_dec,
  sp_inc,
  // etc
};

using IstrTemplateType = std::vector<std::vector<ActionType>>;

std::vector<std::vector<ActionType>> test_template = {
    {pc_cnt, mar_cnt}, {sp_dec, sp_inc}, {}, {}, {},
};

struct OutputType {
public:
  OutputType(uint8_t bout, uint8_t write, uint8_t addr, uint8_t other,
             uint8_t flag_select, bool pc_cnt)
      : _bout(bout), _write(write), _addr(addr), _misc(other),
        _flag_select(flag_select), _pc_cnt(pc_cnt) {
    assert(_bout < 1 << bout_size);
    assert(_write < 1 << write_size);
    assert(_addr < 1 << addr_size);
    assert(_misc < 1 << misc_size);
    assert(_flag_select < 1 << flag_select_size);
    assert(_pc_cnt < 1 << pc_cnt_size);
  }

  bool intersect(const OutputType &other) const {
    if ((_bout > 0) && (other._bout > 0))
      return true;
    if ((_write > 0) && (other._write > 0))
      return true;
    if ((_addr > 0) && (other._addr > 0))
      return true;
    if ((_flag_select > 0) && (other._flag_select > 0))
      return true;
    if ((_pc_cnt > 0) && (other._pc_cnt > 0))
      return true;
    return false;
  }

  void merge(const OutputType &other) {
    // make sure there is no intersection before continuing
    assert(!intersect(other));

    _bout |= other._bout;
    _write |= other._write;
    _addr |= other._addr;
    _misc |= other._misc;
    _flag_select |= other._flag_select;
    _pc_cnt |= other._pc_cnt;
  }

  static OutputType createEmpty() { return OutputType(0, 0, 0, 0, 0, 0); }

private:
  uint8_t _bout;
  uint8_t _write;
  uint8_t _addr;
  uint8_t _misc;
  uint8_t _flag_select;
  uint8_t _pc_cnt;

  // number of bits each number occupies
  static const size_t bout_size = 4;
  static const size_t write_size = 4;
  static const size_t addr_size = 2;
  static const size_t misc_size = 2;
  static const size_t flag_select_size = 3;
  static const size_t pc_cnt_size = 1;
};

const std::map<ActionType, OutputType> action_to_output_map{
    {pc_cnt, OutputType(0, 0, 0, 0, 0, 0)},
    {mar_cnt, OutputType(0, 0, 0, 0, 0, 0)},
    {sp_dec, OutputType(0, 0, 0, 0, 0, 0)},
    {sp_inc, OutputType(0, 0, 0, 0, 0, 0)},
};

const OutputType &actionToOutput(const ActionType &action) {
  return action_to_output_map.at(action);
}

#define action_to_string_map_macro(s) {s, #s}

const std::map<ActionType, std::string> action_to_string_map{
    action_to_string_map_macro(halt),    action_to_string_map_macro(pc_cnt),
    action_to_string_map_macro(mar_cnt), action_to_string_map_macro(sp_dec),
    action_to_string_map_macro(sp_inc),
};

const std::string &actionToString(const ActionType &action) {
  return action_to_string_map.at(action);
}

// TODO: this is only one possible representation of the bits
// might be good to abstract the specific meaning of the bits to different
// possible representations
// struct ComputerState {
//   uint8_t step;
//   uint8_t instruction;
//   uint8_t reg0;
//   uint8_t imm;
//   uint8_t reg1;
//   uint8_t ir2_extra_bits;
//   uint8_t not_vram_active;
// };
struct ComputerState {
  uint8_t step;
  uint8_t ir;
  uint8_t ir2;
  uint8_t not_vram_active;
};

std::string stateToString(const ComputerState &istr) {
  // TODO: implement
  return "";
}

// TODO: return optional for error checking
OutputType templateToOutput(const IstrTemplateType &istr_temp,
                            const ComputerState &istr) {
  // FIXME: step can be greater than the istr temp size
  // will likely refactor this anyway
  const auto &curr = istr_temp.at(istr.step);
  OutputType outputs = OutputType::createEmpty();

  // check for no intersections
  for (size_t i = 0; i < size(curr); i++) {
    const ActionType &action_i = curr[i];
    const OutputType &output_i = actionToOutput(action_i);
    for (size_t j = i + 1; j < size(curr); j++) {
      const ActionType &action_j = curr[j];
      const OutputType &output_j = actionToOutput(action_j);
      if (output_i.intersect(output_j)) {
        std::cerr << "Interesction when merging: " << actionToString(action_i)
                  << ", " << actionToString(action_j) << " - "
                  << stateToString(istr) << "\n";
        return outputs;
      }
    }
  }

  // perform merging logic
  for (const ActionType &action : curr) {
    outputs.merge(actionToOutput(action));
  }

  return outputs;
}

// instructionToTemplate

IstrTemplateType stateToTemplate(const ComputerState &istr_temp) {
  // TODO: implement
  return test_template;
}

void fillTemplate(const IstrTemplateType &istr_temp,
                  const ComputerState &state) {
  // TODO: implement
}

// returns 1 << y_bit if the x_bit bit of x is on
uint32_t bitTransform(uint32_t x, uint32_t x_bit, uint32_t y_bit) {
  return (x & (1 << x_bit)) ? (1 << y_bit) : 0;
}

ComputerState addrToState(const uint32_t addr) {
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

  ComputerState state;

  state.step = STEP;
  state.ir = IR;
  state.ir2 = IR2;
  state.not_vram_active = not_vram_active;

  // state.reg0           = (IR & 0b00000111);
  // state.imm            = (IR & 0b00001000) >> 3;
  // state.instruction    = (IR & 0b11110000) >> 4;
  // state.reg1           = (IR2 & 0b0111);
  // state.ir2_extra_bits = (IR2 & 0b1000) >> 3;
  // state.not_vram_active = not_vram_active;

  return state;
}

// TODO: represent line as stepTemplate, have stepTemplateToOutput?

OutputType stateToOutput(const ComputerState &state) {
  IstrTemplateType istr_temp = stateToTemplate(state);
  fillTemplate(istr_temp, state);

  return templateToOutput(istr_temp, state);
}

OutputType addrToOutputs(const uint32_t addr) {
  const ComputerState state = addrToState(addr);
  stateToOutput(state);
}

// unique mapping ComputerState -> OutputType
// meaning some f(ComputerState) = OutputType, can make pipeline for this
// ComputerState -> unfilled_template -> template filled with registers/etc. ->
// OutputType

// addr -> desired output types -> ucode rom outputs -> output rom image

// sim:
// input rom image -> ucode rom outputs -> ActionTypes -> state changes

// query -> effectively sim logic without running (instead just printing
// what should happen)

// void rom_addr_to_instruction(uint32_t addr) {
// }

// void addr_to_instruction(uint32_t addr, split_addr_t *instruction_ptr) {
//   rom_addr_to_instruction(addr, instruction_ptr);
// }
