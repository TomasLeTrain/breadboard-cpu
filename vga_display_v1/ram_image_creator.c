#include <assert.h>
#include <inttypes.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

const size_t num_chars = 256;
const size_t lines_per_char = 8;
const size_t charset_size = num_chars * lines_per_char;

uint8_t **char_data;

void readCharBin() {
  FILE *file_ptr = fopen("charset.out", "rb");
  if (file_ptr == NULL) {
    fprintf(stderr, "failed!\n");
    exit(1);
    return;
  }

  char_data = (uint8_t **)malloc(num_chars * sizeof(uint8_t *));
  for (size_t character = 0; character < num_chars; character++) {
    char_data[character] = (uint8_t *)malloc(lines_per_char * sizeof(uint8_t));
    fread(char_data[character], sizeof(uint8_t), lines_per_char, file_ptr);
  }
  fclose(file_ptr);
}

const int max_x = 40;
const int max_y = 240;
const int max_bg_color = (1 << 4);
const int max_fg_color = (1 << 4);

#define ram_size (1 << 15)
uint8_t ram_contents[ram_size];

void writeCharacter(uint8_t x, uint8_t y, uint8_t character, uint8_t bg_color,
                    uint8_t fg_color) {
  assert(x < max_x);
  assert(y < max_y);
  assert(bg_color < max_bg_color);
  assert(fg_color < max_fg_color);

  uint8_t color_value = (fg_color << 4) | bg_color;
  uint8_t char_value = char_data[character][y & 0b111];

  uint16_t addr = x | (y << 6);

  ram_contents[addr | (0 << 14)] = color_value;
  ram_contents[addr | (1 << 14)] = char_value;
}

void createTestBinary() {
  // uint8_t bg_color = 0, fg_color = 0xf;
  // uint8_t character = 254;

  for (uint8_t x = 0; x < max_x; x++) {
    for (uint8_t y = 0; y < max_y; y++) {
      writeCharacter(x, y, x + 40 * ((y & (~0b111)) >> 3), y & 0xf, x & 0xf);
      // writeCharacter(x, y, character, bg_color, fg_color);
    }
  }
}

void write_image_logisim() {
  printf("v3.0 hex words plain\n");
  for (int addr = 0; addr < ram_size; addr++) {
    uint8_t curr = ram_contents[addr];
    printf("%02X", curr);
    if (addr % 16 == 15) {
      printf("\n");
    } else {
      printf(" ");
    }
  }
}

int main() {
  readCharBin();
  createTestBinary();
  write_image_logisim();
}
