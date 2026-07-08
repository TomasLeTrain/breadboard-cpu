#pragma once

#include <cassert>
#include <cstdint>
#include <string>

class OutputCategory {
  size_t bit_length;
  size_t offset;
  std::string name;

public:
  OutputCategory(size_t bit_length, size_t offset, std::string name)
      : bit_length(bit_length), offset(offset), name(name) {}

  uint16_t getCategoryData(uint16_t word) {
    uint16_t mask = (1 << bit_length) - 1;
    return (word >> offset) & mask;
  }

  bool dataValid(uint16_t data) { return data < (1 << bit_length); }
};
