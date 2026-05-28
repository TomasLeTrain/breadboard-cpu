/**
 * Copyright (c) 2020 Raspberry Pi (Trading) Ltd.
 *
 * SPDX-License-Identifier: BSD-3-Clause
 */

// Output a 12.5 MHz square wave (if system clock frequency is 125 MHz).
//
// Note this program is accessing the PIO registers directly, for illustrative
// purposes. We pull this program into the datasheet so we can talk a little
// about PIO's hardware register interface. The `hardware_pio` SDK library
// provides simpler or better interfaces for all of these operations.
//
// _*This is not best practice! I don't want to see you copy/pasting this*_
//
// For a minimal example of loading and running a program using the SDK
// functions (which is what you generally want to do) have a look at
// `hello_pio` instead. That example is also the subject of a tutorial in the
// SDK book, which walks you through building your first PIO program.

#include "hardware/pio.h"
#include "pico/stdlib.h"

// Our assembled program:
#include "squarewave_fast.pio.h"

#include "hardware/clocks.h"

int main() {
  // approx 100.71428571428571 MHz - close enough to 100.7
  set_sys_clock_pll(1410 * MHZ, 7, 2);

  // 100.7MHz / 4 = 25.175 MHz -> clock required for vga
  // 25.175 / 4 = aproxx 6.3 MHz -> slower clock for vga circuit
  // total division needed is 16
  static const double div = 16.0;

  static const uint8_t output_pin = 21;
  PIO pio = pio0;
  uint8_t sm = pio_claim_unused_sm(pio, true);
  uint8_t offset = pio_add_program(pio, &squarewave_fast_program);
  clk_program_init(pio, sm, offset, output_pin, div);
  pio_sm_set_enabled(pio, sm, true);
}
