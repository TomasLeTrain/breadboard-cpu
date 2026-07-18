#pragma once

#include "Opcode.hpp"
#include "Templates.hpp"
#include <array>
#include <memory>

class TemplateGeneratorInterface {
public:
  virtual IstrTemplateType opcodeToTemplate(const Opcode &state) = 0;
};

class Instruction : public TemplateGeneratorInterface {
public:
  Instruction(IstrTemplateType instruction_template)
      : instruction_template(instruction_template) {}

  IstrTemplateType opcodeToTemplate(const Opcode &state) override {
    return instruction_template;
  }

private:
  IstrTemplateType instruction_template;
};

class ExtendedInstruction : public TemplateGeneratorInterface {
public:
  std::array<std::unique_ptr<TemplateGeneratorInterface>, 16> instructions;

  IstrTemplateType opcodeToTemplate(const Opcode &opcode) override {
    return instructions.at(opcode.ir2)->opcodeToTemplate(opcode);
  }

  void setTemplateGenerator(int ir2_idx,
                            std::unique_ptr<TemplateGeneratorInterface> gen) {
    instructions.at(ir2_idx) = std::move(gen);
  }
};

class InstructionSet : public TemplateGeneratorInterface {
public:
  std::array<std::unique_ptr<TemplateGeneratorInterface>, 256> instructions;

  IstrTemplateType opcodeToTemplate(const Opcode &opcode) override {
    return instructions.at(opcode.ir)->opcodeToTemplate(opcode);
  }

  void setTemplateGenerator(int ir_idx,
                            std::unique_ptr<TemplateGeneratorInterface> gen) {
    instructions.at(ir_idx) = std::move(gen);
  }
};
