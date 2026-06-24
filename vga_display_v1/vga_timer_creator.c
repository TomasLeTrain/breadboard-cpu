#include <assert.h>
#include <inttypes.h>
#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define MAX_ADDR_LEN 15
#define MAX_ADDR ((1 << MAX_ADDR_LEN) - 1)

const uint8_t VSYNC = (1 << 0);
const uint8_t HSYNC = (1 << 1);
const uint8_t nX_CNT = (1 << 2);
const uint8_t nY_CNT = (1 << 3);
const uint8_t nX_RESET = (1 << 4);
const uint8_t nY_RESET = (1 << 5);
const uint8_t nROM_RESET = (1 << 6);
const uint8_t DEFAULT = nY_CNT | nX_CNT | nX_RESET | nY_RESET | nROM_RESET;

// counts once past last character to have ram output blank when not drawing
const uint16_t max_drawing_x = 38;
const uint16_t max_drawing_y = 479;

uint8_t getInstruction(uint32_t addr) {
  uint32_t x = addr % 50;
  uint32_t y = addr / 50;
  uint8_t result = DEFAULT;

  bool drawing_column = x <= max_drawing_x;
  bool drawing_line = y <= max_drawing_y;
  bool drawing_character = drawing_column && drawing_line;
  // true on last character of drawing
  bool end_drawing_x = (x == max_drawing_x);

  if (drawing_character)
    result &= ~nX_CNT;

  if (drawing_line && x == 39) {
    // x counter is at 40 latching data for 41
    // therefore x should be reset to 0 to latch in empty data
    result &= ~nX_RESET;
    // can also increment y here since x = 0 is empty everywhere
    // count y on every other line to mantain 240 resolution
    if (y % 2 == 1)
      result &= ~nY_CNT;
  }

  if (drawing_line && x == 48) {
    // incrementing now puts x to zero on next clock, meaning that by 49 it will
    // be clocking data for 0
    result &= ~nX_CNT;
  }

  if (41 <= x && x < 47)
    result |= HSYNC;

  if (490 <= y && y < 492)
    result |= VSYNC;

  if (x == 49 && y == 524) {
    result &= ~nY_RESET;
  }

  if (y >= 525) {
    result &= ~nROM_RESET;
  }

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
