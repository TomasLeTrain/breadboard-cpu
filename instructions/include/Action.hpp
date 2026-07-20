#pragma once

// effectively any output wire is represented as one action type
// TODO: add special actions that do not actually exist, but signal special
// flags that get used to fill info in?
// or use variants instead to avoid polluting the enum
#include <string>

enum Action {
  halt = 0,
  nop, // equal to zero outputs
  reset,

  // addr regs cnt
  pc_cnt,
  mar_cnt,
  sp_dec,
  sp_inc,

  // addr regs addr
  pc_addr,
  mar_addr,
  sp_addr,

  // registers bout
  a_bout,
  b_bout,
  x_bout,
  y_bout,
  z_bout,
  pc_lo_bout,
  pc_hi_bout,
  mar_lo_bout,
  mar_hi_bout,
  sp_lo_bout,
  sp_hi_bout,
  keyb_bout,
  flags_bout,
  f_alu_bout,

  // registers write
  a_write,
  b_write,
  x_write,
  y_write,
  z_write,
  pc_lo_write,
  pc_hi_write,
  mar_lo_write,
  mar_hi_write,
  sp_lo_write,
  sp_hi_write,

  // ir regs
  ir_write,
  ir2_write,

  // mem read/write
  mem_read,
  mem_write,

  // vram read/write
  vram_read,
  vram_write,

  // register shifts
  x_shift_left,
  y_shift_left,
  x_shift_right,
  y_shift_right,

  // flags
  flag_direct,
  flag_carry,
  flag_eq,
  flag_zero,
  flag_6,
  flag_5,
  flag_7,
  flag_8,

  flag_write_alu,

  // placeholder registers
  reg0_bout,
  reg1_bout,
  reg0_write,
  reg1_write,
  output_flags_selector
};

// split into structs to make referencing easier
struct a {
  static const Action bout = a_bout;
  static const Action write = a_write;
};

struct b {
  static const Action bout = b_bout;
  static const Action write = b_write;
};

struct x {
  static const Action bout = x_bout;
  static const Action write = x_write;
  static const Action shift_left = x_shift_left;
  static const Action shift_right = x_shift_right;
};

struct y {
  static const Action bout = y_bout;
  static const Action write = y_write;
  static const Action shift_left = y_shift_left;
  static const Action shift_right = y_shift_right;
};

struct z {
  static const Action bout = z_bout;
  static const Action write = z_write;
};

struct pc {
  static const Action cnt = pc_cnt;
  static const Action addr = pc_addr;

  struct lo {
    static const Action bout = pc_lo_bout;
    static const Action write = pc_lo_write;
  };
  struct hi {
    static const Action bout = pc_hi_bout;
    static const Action write = pc_hi_write;
  };
};

struct mar {
  static const Action cnt = mar_cnt;
  static const Action addr = mar_addr;

  struct lo {
    static const Action bout = mar_lo_bout;
    static const Action write = mar_lo_write;
  };
  struct hi {
    static const Action bout = mar_hi_bout;
    static const Action write = mar_hi_write;
  };
};

struct flags {
  static const Action bout = flags_bout;
  static const Action alu_write = flag_write_alu;

  struct select {
    static const Action direct = flag_direct;
    static const Action carry = flag_carry;
    static const Action eq = flag_eq;
    static const Action zero = flag_zero;
    static const Action f6 = flag_6;
    static const Action f5 = flag_5;
    static const Action f7 = flag_7;
    static const Action f8 = flag_8;
  };
};

struct sp {
  static const Action inc = sp_inc;
  static const Action dec = sp_dec;
  static const Action addr = sp_addr;

  struct lo {
    static const Action bout = sp_lo_bout;
    static const Action write = sp_lo_write;
  };
  struct hi {
    static const Action bout = sp_hi_bout;
    static const Action write = sp_hi_write;
  };
};

struct ir {
  static const Action write = ir_write;
};

struct ir2 {
  static const Action write = ir2_write;
};

struct mem {
  static const Action read = mem_read;
  static const Action write = mem_write;
};

struct vram {
  static const Action read = vram_read;
  static const Action write = vram_write;
};

#define action_to_string_map_macro(s) {s, #s}

const std::string &actionToString(const Action &action);
