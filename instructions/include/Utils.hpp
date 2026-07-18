#pragma once
#include <cstdint>

// returns 1 << y_bit if the x_bit bit of x is on
inline uint32_t bitTransform(uint32_t x, uint32_t x_bit, uint32_t y_bit) {
  return (x & (1 << x_bit)) ? (1 << y_bit) : 0;
}
