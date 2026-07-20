#include "ActionDefinitions.hpp"
#include <map>

static Output write(uint8_t val) { return Output(0, val, 0, 0, 0, 0); }
static Output bout(uint8_t val) { return Output(val, 0, 0, 0, 0, 0); }
static Output addr(uint8_t val) { return Output(0, 0, val, 0, 0, 0); }
static Output other(uint8_t val) { return Output(0, 0, 0, val, 0, 0); }
static Output flag_select(uint8_t val) { return Output(0, 0, 0, 0, val, 0); }
static Output pc_cnt_output(uint8_t val) { return Output(0, 0, 0, 0, 0, val); }

const std::map<Action, Output> action_to_output_map{
    {halt, bout(5)},
    {nop, Output::createEmpty()},
    {reset, other(2)},

    {pc_cnt, pc_cnt_output(1)},
    {mar_cnt, bout(4)},
    {sp_dec, bout(7)},
    {sp_inc, bout(6)},

    // addr regs cnt
    {pc_cnt, Output(0, 0, 0, 0, 0, 0)},
    {mar_cnt, Output(0, 0, 0, 0, 0, 0)},
    {sp_dec, Output(0, 0, 0, 0, 0, 0)},
    {sp_inc, Output(0, 0, 0, 0, 0, 0)},

    // addr regs addr
    {pc_addr, addr(1)},
    {mar_addr, addr(2)},
    {sp_addr, addr(3)},

    // registers bout
    {a_bout, bout(0b1000 | 0)},
    {b_bout, bout(0b1000 | 1)},
    {x_bout, bout(0b1000 | 5)},
    {y_bout, bout(0b1000 | 6)},
    {z_bout, bout((0b1000 | 7))},
    {pc_lo_bout, Output::merge(addr(1), bout(2))},
    {pc_hi_bout, Output::merge(addr(1), bout(3))},
    {mar_lo_bout, Output::merge(addr(2), bout(2))},
    {mar_hi_bout, Output::merge(addr(2), bout(3))},
    {sp_lo_bout, Output::merge(addr(3), bout(2))},
    {sp_hi_bout, Output::merge(addr(3), bout(3))},
    {keyb_bout, bout(0b1000 | 2)},
    {flags_bout, bout(0b1000 | 4)},
    {f_alu_bout, bout(0b1000 | 3)},

    // registers write
    {a_write, write(0b1000 | 0)},
    {b_write, write(0b1000 | 1)},
    {x_write, write(0b1000 | 5)},
    {y_write, write(0b1000 | 6)},
    {z_write, write(7)},
    {pc_lo_write, write(0b1000 | 3)},
    {pc_hi_write, write(0b1000 | 2)},
    {mar_lo_write, write(5)},
    {mar_hi_write, write(4)},
    {sp_lo_write, write(3)},
    {sp_hi_write, write(2)},

    // ir regs
    {ir_write, write(0b1000 | 7)},
    {ir2_write, write(1)},

    // mem read/write
    {mem_read, bout(1)},
    {mem_write, write(6)},

    // vram read/write
    {vram_read, Output::merge(other(3), flag_select(5))},
    {vram_write, Output::merge(other(3), flag_select(4))},

    // shift left
    {x_shift_left, Output::merge(other(3), flag_select(0))},
    {y_shift_left, Output::merge(other(3), flag_select(2))},

    // shift right
    {x_shift_right, Output::merge(other(3), flag_select(1))},
    {y_shift_right, Output::merge(other(3), flag_select(3))},

    // flags
    {flag_direct, flag_select(0)},
    {flag_carry, flag_select(1)},
    {flag_eq, flag_select(2)},
    {flag_zero, flag_select(3)},
    {flag_6, flag_select(4)},
    {flag_5, flag_select(5)},
    {flag_7, flag_select(6)},
    {flag_8, flag_select(7)},

    {flag_write_alu, other(1)},
};

const Output &actionToOutput(const Action &action) {
  return action_to_output_map.at(action);
}
