#include <assert.h>
#include <inttypes.h>
#include <stdarg.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define MAX_ADDR_LEN 15
#define MAX_ADDR ((1 << MAX_ADDR_LEN) - 1)

const int hor_scale_down_factor = 4;
const int ver_scale_down_factor = 4;

const int hor_visible_area_dur = 640 / hor_scale_down_factor;
const int hor_front_porch_dur = 16 / hor_scale_down_factor;
const int hor_sync_pulse_dur = 96 / hor_scale_down_factor;
const int hor_back_porch_dur = 48 / hor_scale_down_factor;
// 800 with factor = 1
const int hor_whole_line_dur = hor_visible_area_dur + hor_front_porch_dur +
                               hor_sync_pulse_dur + hor_back_porch_dur;

const int hor_front_porch_addr = hor_visible_area_dur;
const int hor_sync_pulse_addr = hor_front_porch_addr + hor_front_porch_dur;
const int hor_back_porch_addr = hor_sync_pulse_addr + hor_sync_pulse_dur;

const int ver_visible_area_dur = 400 / hor_scale_down_factor;
const int ver_front_porch_dur = 12 / hor_scale_down_factor;
// rounded up from 2
const int ver_sync_pulse_dur = 4 / hor_scale_down_factor;
// taken down to 33 because of the rounding
const int ver_back_porch_dur = 33 / hor_scale_down_factor;
// 449 with factor = 1
const int ver_whole_line_dur = hor_visible_area_dur + hor_front_porch_dur +
                               hor_sync_pulse_dur + hor_back_porch_dur;

const int ver_front_porch_addr = ver_visible_area_dur;
const int ver_sync_pulse_addr = ver_front_porch_addr + ver_front_porch_dur;
const int ver_back_porch_addr = ver_sync_pulse_addr + ver_sync_pulse_dur;

const uint8_t BIT_NVRESET = 0b00000001;
const uint8_t BIT_VSYNC = 0b01000000;
const uint8_t BIT_HSYNC = 0b00100000;
const uint8_t BIT_NHRESET = 0b10000000;

const uint8_t BIT_DRAWING = 0;
const uint8_t BIT_NOT_DRAWING = BIT_NVRESET | BIT_NHRESET | 0b00000100;

FILE *file_ptr;
char **image_data;

void readImageBin() {
  file_ptr = fopen("finch.bin", "rb");
  if (file_ptr == NULL) {
    printf("Error! opening file");
    fclose(file_ptr);
    return;
  }
  int r = 100, c = 160;
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
  return (0b01111111) & (red | (green << 3) | (blue << 6));
}

// NOTE: assumes getColor is only called when there is actually color data
uint8_t getColor(int addr) {
  uint32_t hor_count = addr & 0b000000011111111;
  uint32_t ver_count = (addr & 0b111111100000000) >> 8;
  // printf("%d %d\n", hor_count, ver_count);

  uint8_t curr_pixel = image_data[ver_count][hor_count];

  // make sure drawing bit is not set
  curr_pixel &= (0b01111111);
  return curr_pixel;
  // return makeColor(hor_count, ver_count, 0);
}

uint8_t getInstruction(int addr) {
  uint32_t hor_count = addr & 0b000000011111111;
  uint32_t ver_count = (addr & 0b111111100000000) >> 8;

  // done with >= in case the circuit ends up in an invalid address it gets auto
  // reset could happen at powerup if registers have random values
  int hor_end = hor_count >= hor_whole_line_dur;
  int ver_end = ver_count >= ver_whole_line_dur;

  // inside drawing area
  if (hor_count < hor_visible_area_dur && ver_count < ver_visible_area_dur) {
    return BIT_DRAWING | getColor(addr);
  }

  if (hor_end && ver_end) {
    return BIT_NOT_DRAWING & (~BIT_NVRESET);
  }

  if (hor_end) {
    return BIT_NOT_DRAWING & (~BIT_NHRESET);
  }

  // might need to send both
  uint8_t result = BIT_NOT_DRAWING;

  if (ver_count >= ver_sync_pulse_addr && ver_count < ver_back_porch_addr) {
    // do vsync pulse
    result |= BIT_VSYNC;
  }
  if (hor_count >= hor_sync_pulse_addr && hor_count < hor_back_porch_addr) {
    // do hsync pulse
    result |= BIT_HSYNC;
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

int main() {
  readImageBin();
  write_ucode_logism();
}
