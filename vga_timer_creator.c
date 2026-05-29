#include <assert.h>
#include <inttypes.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define MAX_ADDR_LEN 15
#define MAX_ADDR ((1 << MAX_ADDR_LEN) - 1)

// mask that excludes lower 2 bits
const int VIDEO_BITS_MASK = ~(0b11);

// horizontal timings
const int actual_hor_visible_area_dur = 640;
const int actual_hor_front_porch_dur = 16;
const int actual_hor_sync_pulse_dur = 96;
const int actual_hor_back_porch_dur = 48;
// 640 (inclusive)
const int actual_hor_front_porch_addr = actual_hor_visible_area_dur;
// 656 (inclusive)
const int actual_hor_sync_pulse_addr =
    actual_hor_front_porch_addr + actual_hor_front_porch_dur;
// 752 (inclusive)
const int actual_hor_back_porch_addr =
    actual_hor_sync_pulse_addr + actual_hor_sync_pulse_dur;
// 800 (inclusive)
const int actual_hor_whole_line_addr =
    actual_hor_back_porch_addr + actual_hor_back_porch_dur;

// vertical timings
const int actual_ver_visible_area_dur = 480;
const int actual_ver_front_porch_dur = 10;
const int actual_ver_sync_pulse_dur = 2;
const int actual_ver_back_porch_dur = 33;
// 480 (inclusive) - right after visible area
const int actual_ver_front_porch_addr = actual_ver_visible_area_dur;
// 490 (inclusive) - 12 lines after drawing ended
const int actual_ver_sync_pulse_addr =
    actual_ver_front_porch_addr + actual_ver_front_porch_dur;
// 492 (inclusive) - 2 lines after pulse started
const int actual_ver_back_porch_addr =
    actual_ver_sync_pulse_addr + actual_ver_sync_pulse_dur;
// 525 (inclusive)
const int actual_ver_whole_line_addr =
    actual_ver_back_porch_addr + actual_ver_back_porch_dur;

const int hor_front_porch_addr = actual_hor_front_porch_addr & VIDEO_BITS_MASK;
const int hor_sync_pulse_addr = actual_hor_sync_pulse_addr & VIDEO_BITS_MASK;
const int hor_back_porch_addr = actual_hor_back_porch_addr & VIDEO_BITS_MASK;
const int hor_whole_line_addr = actual_hor_whole_line_addr & VIDEO_BITS_MASK;

const int ver_front_porch_addr = actual_ver_front_porch_addr & VIDEO_BITS_MASK;
const int ver_sync_pulse_addr = actual_ver_sync_pulse_addr & VIDEO_BITS_MASK;
const int ver_back_porch_addr = actual_ver_back_porch_addr & VIDEO_BITS_MASK;
const int ver_whole_line_addr = actual_ver_whole_line_addr & VIDEO_BITS_MASK;

// clang-format off
// const uint8_t BIT_NVRESET = 0b00100000;
// const uint8_t BIT_NVSYNC  = 0b01000000;
const uint8_t BIT_VSYNC   = 0b01000000;
const uint8_t BIT_NHSYNC  = 0b10000000;
const uint8_t BIT_NHRESET = 0b00000001;
// clang-format on

const uint8_t DRAWING_BIT = 0b00000100;
const uint8_t NO_DRAWING_MASK = ~DRAWING_BIT;

const uint8_t BIT_DRAWING = 0;
// const uint8_t BIT_NOT_DRAWING =
//     BIT_NVRESET | BIT_NHRESET | BIT_NVSYNC | BIT_NHSYNC | DRAWING_BIT;
const uint8_t BIT_NOT_DRAWING =
    BIT_NHRESET | BIT_NHSYNC | DRAWING_BIT;

FILE *file_ptr;
char **image_data;

void readImageBin() {
  file_ptr = fopen("finch.bin", "rb");
  if (file_ptr == NULL) {
    printf("Error! opening file");
    fclose(file_ptr);
    return;
  }

  int r = actual_ver_visible_area_dur / 4, c = actual_hor_visible_area_dur / 4;

  image_data = (char **)malloc(r * sizeof(char *));
  for (int i = 0; i < r; i++) {
    image_data[i] = (char *)malloc(c * sizeof(char));
    fread(image_data[i], c, 1, file_ptr);
  }
  fclose(file_ptr);
}

// 2 bit blue, 3 green, 2 red
uint8_t makeColor(uint32_t blue, uint32_t green, uint32_t red) {
  // makes sure drawing bit not accidentally set
  red = red & 0b11;
  green = green & 0b111;
  blue = blue & 0b11;
  return NO_DRAWING_MASK & (red | (blue << 3) | (green << 5));
}

// NOTE: assumes getColor is only called when there is actually color data
uint8_t getColor(int addr) {
  uint32_t hor_count = addr & 0b000000011111111;
  uint32_t ver_count = (addr & 0b111111100000000) >> 8;
  // printf("%d %d\n", hor_count, ver_count);

  uint8_t curr_pixel = image_data[ver_count][hor_count];

  // make sure drawing bit is not set
  curr_pixel &= NO_DRAWING_MASK;
  return curr_pixel;
}

uint8_t getInstruction(int addr) {
  uint32_t raw_hor_count = (addr & 0b000000011111111);
  uint32_t raw_ver_count = (addr & 0b111111100000000) >> 8;

  // multiplied to be able to compare with actual counts
  uint32_t hor_count = raw_hor_count << 2;
  uint32_t ver_count = raw_ver_count << 2;

  // done with >= in case the circuit ends up in an invalid address it gets auto
  // reset. could happen at powerup if registers have random values
  // NOTE: reset gets done one clock cycle after, so we must output reset one
  // cycle before actual reset
  int hor_end = hor_count >= hor_whole_line_addr - 4;
  // int ver_end = ver_count >= ver_whole_line_addr;

  // inside drawing area
  if (hor_count < hor_front_porch_addr && ver_count < ver_front_porch_addr) {
    return BIT_DRAWING | getColor(addr);
  }

  // might need to send both
  uint8_t result = BIT_NOT_DRAWING;

  // if (ver_end) {
  //   // reset both
  //   result &= (~BIT_NVRESET);
  //   // result &= (~BIT_NHRESET);
  // }

  if (hor_end) {
    result &= (~BIT_NHRESET);
  }

  // ver count is equal in higher bits, guaranteed
  if (ver_count == ver_sync_pulse_addr) {
    result |= BIT_VSYNC;
  }
  // this case is also guaranteed (case where whole address is within pulse)
  if (ver_count >= ver_sync_pulse_addr && ver_count < ver_back_porch_addr) {
    result |= BIT_VSYNC;
  }
  // ver count is equal in higher bits, possibility of overlap
  if ((ver_count == ver_back_porch_addr) &&
      // check if lower bits of back porch are 0, if not then some overlap
      ((actual_ver_back_porch_addr & 0b11) != 0)) {
    result |= BIT_VSYNC;
  }

  // horizontal pulse has evenly divisible timings, simple check
  if (hor_count >= hor_sync_pulse_addr && hor_count < hor_back_porch_addr) {
    result &= ~BIT_NHSYNC;
  }

  return result;
}

void write_ucode_logism() {
  printf("v3.0 hex words plain\n");
  for (int addr = 0; addr <= MAX_ADDR; addr++) {
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
  readImageBin();
  write_ucode_logism();
  write_binary_image();
}
