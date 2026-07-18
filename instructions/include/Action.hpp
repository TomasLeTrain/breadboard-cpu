#pragma once

// effectively any output wire is represented as one action type
// TODO: add special actions that do not actually exist, but signal special
// flags that get used to fill info in?
// or use variants instead to avoid polluting the enum
#include <map>
#include <string>
enum Action {
  // TODO: halt = 0 so the default action is to halt if nothing else is
  // specified?
  halt = 0,
  pc_cnt,
  mar_cnt,
  sp_dec,
  sp_inc,
  // etc
};

#define action_to_string_map_macro(s) {s, #s}

const std::string &actionToString(const Action &action);
