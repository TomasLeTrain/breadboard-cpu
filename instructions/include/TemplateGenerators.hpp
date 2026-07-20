#pragma once

#include "Opcode.hpp"
#include "Templates.hpp"
#include <array>
#include <memory>
#include <queue>

class TemplateGeneratorInterface {
public:
  virtual IstrTemplateType opcodeToTemplate(const Opcode &opcode) = 0;
};

class Instruction : public TemplateGeneratorInterface {
public:
  Instruction(IstrTemplateType instruction_template, std::string name)
      : instruction_template(instruction_template), name(name) {}

  IstrTemplateType opcodeToTemplate(const Opcode &opcode) override {
    return instruction_template;
  }

private:
  IstrTemplateType instruction_template;
  std::string name;
};

class ExtendedInstruction : public TemplateGeneratorInterface {
public:
  IstrTemplateType opcodeToTemplate(const Opcode &opcode) override {
    return instructions.at(opcode.ir2)->opcodeToTemplate(opcode);
  }

  void setTemplateGenerator(int ir2_idx,
                            std::unique_ptr<TemplateGeneratorInterface> gen) {
    instructions.at(ir2_idx) = std::move(gen);
  }

private:
  std::array<std::unique_ptr<TemplateGeneratorInterface>, 16> instructions;
};

class InstructionSet : public TemplateGeneratorInterface {
public:
  InstructionSet() {}

  IstrTemplateType opcodeToTemplate(const Opcode &opcode) override {
    return instructions.at(opcode.ir)->opcodeToTemplate(opcode);
  }

  void setTemplateGenerator(int ir_idx,
                            std::unique_ptr<TemplateGeneratorInterface> gen) {
    instructions.at(ir_idx) = std::move(gen);
  }

private:
  std::array<std::unique_ptr<TemplateGeneratorInterface>, 256> instructions;
};
