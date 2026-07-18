#pragma once

#include <cstdint>
#include <string>
#include <sys/types.h>

struct Opcode {
  uint8_t step;
  uint8_t ir;
  uint8_t ir2;
  uint8_t not_vram_active;
};

std::string stateToString(const Opcode &istr);
