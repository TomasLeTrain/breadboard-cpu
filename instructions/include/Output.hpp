#pragma once

#include <cassert>
#include <cstddef>
#include <cstdint>

struct Output {
public:
  Output(uint8_t bout, uint8_t write, uint8_t addr, uint8_t other,
         uint8_t flag_select, bool pc_cnt)
      : _bout(bout), _write(write), _addr(addr), _misc(other),
        _flag_select(flag_select), _pc_cnt(pc_cnt) {
    assert(_bout < 1 << bout_size);
    assert(_write < 1 << write_size);
    assert(_addr < 1 << addr_size);
    assert(_misc < 1 << misc_size);
    assert(_flag_select < 1 << flag_select_size);
    assert(_pc_cnt < 1 << pc_cnt_size);
  }

  bool intersect(const Output &other) const {
    if ((_bout > 0) && (other._bout > 0))
      return true;
    if ((_write > 0) && (other._write > 0))
      return true;
    if ((_addr > 0) && (other._addr > 0))
      return true;
    if ((_flag_select > 0) && (other._flag_select > 0))
      return true;
    if ((_pc_cnt > 0) && (other._pc_cnt > 0))
      return true;
    return false;
  }

  void merge(const Output &other) {
    // make sure there is no intersection before continuing
    assert(!intersect(other));

    _bout |= other._bout;
    _write |= other._write;
    _addr |= other._addr;
    _misc |= other._misc;
    _flag_select |= other._flag_select;
    _pc_cnt |= other._pc_cnt;
  }

  static Output createEmpty() { return Output(0, 0, 0, 0, 0, 0); }

private:
  uint8_t _bout;
  uint8_t _write;
  uint8_t _addr;
  uint8_t _misc;
  uint8_t _flag_select;
  uint8_t _pc_cnt;

  // number of bits each number occupies
  static const size_t bout_size = 4;
  static const size_t write_size = 4;
  static const size_t addr_size = 2;
  static const size_t misc_size = 2;
  static const size_t flag_select_size = 3;
  static const size_t pc_cnt_size = 1;
};
