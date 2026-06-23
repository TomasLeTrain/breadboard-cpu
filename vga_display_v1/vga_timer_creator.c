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
const uint16_t max_drawing_x = 41;
const uint16_t max_drawing_y = 480;

uint8_t getInstruction(uint32_t addr) {
  uint32_t x = addr % 50;
  uint32_t y = addr / 50;
  uint8_t result = DEFAULT;

  bool drawing_x = x < max_drawing_x;
  bool drawing_y = y < max_drawing_y;
  bool drawing = drawing_x && drawing_y;
  // true on last character of drawing
  bool end_drawing_x = (x == max_drawing_x - 1);

  if (drawing)
    result &= ~nX_CNT;

  // end of line - reset only now
  if (drawing_y && x == 49) {
    result &= ~nX_RESET;
    // count y on every other line to mantain 240
    if (y % 2 == 1)
      result &= ~nY_CNT;
  }

  if (41 <= x && x < 47)
    result |= HSYNC;

  if (490 <= y && y < 492)
    result |= VSYNC;

  // if ((x == 49 && y == 524) || y > 524) {
  if (y >= 525) {
    result &= ~nROM_RESET;
    result &= ~nX_RESET;
    result &= ~nY_RESET;
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
