#pragma once

#include "Opcode.hpp"
#include "Templates.hpp"
#include <array>
#include <memory>
#include <queue>

class Instruction {
public:
  Instruction(IstrTemplateType instruction_template, std::string name)
      : instruction_template(instruction_template), name(name) {}

  const IstrTemplateType &getTemplate() const { return instruction_template; }
  std::string getName() const { return name; }

private:
  IstrTemplateType instruction_template;
  std::string name;
};

class TemplateGeneratorInterface {
public:
  virtual const Instruction &opcodeToInstruction(const Opcode &opcode) = 0;
};

class InstructionWrapper : public TemplateGeneratorInterface {
public:
  InstructionWrapper(const Instruction &instruction)
      : instruction(instruction) {}

  const Instruction &opcodeToInstruction(const Opcode &opcode) override {
    return instruction;
  }

private:
  const Instruction &instruction;
};

class ExtendedInstruction : public TemplateGeneratorInterface {
public:
  const Instruction &opcodeToInstruction(const Opcode &opcode) override {
    return instructions.at(opcode.ir2)->opcodeToInstruction(opcode);
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

  const Instruction &opcodeToInstruction(const Opcode &opcode) override {
    return instructions.at(opcode.ir)->opcodeToInstruction(opcode);
  }

  void setTemplateGenerator(int ir_idx,
                            std::unique_ptr<TemplateGeneratorInterface> gen) {
    instructions.at(ir_idx) = std::move(gen);
  }

private:
  std::array<std::unique_ptr<TemplateGeneratorInterface>, 256> instructions;
};
