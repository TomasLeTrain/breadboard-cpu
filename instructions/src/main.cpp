#include <array>
#include <cassert>
#include <cstdint>
#include <iostream>
#include <map>
#include <string>
#include <vector>

#include "Action.hpp"
#include "ActionDefinitions.hpp"
#include "Opcode.hpp"
#include "Output.hpp"
#include "TemplateDefinitions.hpp"
#include "Templates.hpp"
#include "Utils.hpp"

// TODO: return optional for error checking
Output stepTemplateToOutput(const StepTemplateType &step_istr,
                            const Opcode &istr) {
  Output output = Output::createEmpty();

  // check for no intersections
  for (size_t i = 0; i < size(step_istr); i++) {
    const Action &action_i = step_istr.at(i);
    const Output &output_i = actionToOutput(action_i);
    for (size_t j = i + 1; j < size(step_istr); j++) {
      const Action &action_j = step_istr.at(j);
      const Output &output_j = actionToOutput(action_j);
      if (output_i.intersect(output_j)) {
        std::cerr << "Interesction when merging: " << actionToString(action_i)
                  << ", " << actionToString(action_j) << " - "
                  << stateToString(istr) << "\n";
        return output;
      }
    }
  }

  // perform merging logic
  for (const Action &action : step_istr) {
    output.merge(actionToOutput(action));
  }

  return output;
}

Opcode addrToOpcode(const uint32_t addr) {
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

  Opcode opcode;

  opcode.step = STEP;
  opcode.ir = IR;
  opcode.ir2 = IR2;
  opcode.not_vram_active = not_vram_active;

  return opcode;
}

Output opcodeToOutput(const Opcode &opcode) {
  IstrTemplateType istr_temp = opcodeToTemplate(opcode);

  // FIXME: step can be greater than the istr temp size
  const auto &step_istr = istr_temp.at(opcode.step);

  return stepTemplateToOutput(step_istr, opcode);
}

Output addrToOutputs(const uint32_t addr) {
  return opcodeToOutput(addrToOpcode(addr));
}

int main() {}
