#include <assert.h>
#include <inttypes.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define MAX_ADDR_LEN 15
#define MAX_ADDR ((1 << MAX_ADDR_LEN) - 1)

const uint8_t DRAWING = 0b00000100;
const uint8_t COUNT_RESET = 0b00001000;
const uint8_t VSYNC = 0b00000001;
const uint8_t HSYNC = 0b00000010;
const uint8_t DEFAULT = DRAWING | COUNT_RESET;

uint8_t getInstruction(uint32_t addr) {
  uint32_t x = addr % 50;
  uint32_t y = addr / 50;
  uint8_t result = DEFAULT;

  if (x < 40)
    result &= ~DRAWING;
  if (41 <= x && x < 47)
    result |= HSYNC;

  if (490 <= y && y < 492)
    result |= HSYNC;

  // need to send signal clock before to avoid drift
  if (y == 524 && x == 49 || y > 524)
    result &= ~COUNT_RESET;

  return result;
}

void write_ucode_logism() {
  printf("v3.0 hex words plain\n");
  for (uint32_t addr = 0; addr <= MAX_ADDR; addr++) {
    printf("%02X", getInstruction(addr));
    if (addr % 16 == 15) {
      printf("\n");
    } else {
      printf(" ");
    }
  }
}

void write_binary_image() {
  FILE *fptr;
  // Open a file in append mode
  fptr = fopen("vga_rom.bin", "wb");

  for (int addr = 0; addr <= MAX_ADDR; addr++) {
    uint8_t curr = getInstruction(addr);
    fputc(curr, fptr);
  }
  // Close the file
  fclose(fptr);
}

int main() {
  // readImageBin();
  write_ucode_logism();
  write_binary_image();
}
