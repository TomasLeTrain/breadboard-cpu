#include "Action.hpp"

const std::map<Action, std::string> action_to_string_map{
    action_to_string_map_macro(halt),    action_to_string_map_macro(pc_cnt),
    action_to_string_map_macro(mar_cnt), action_to_string_map_macro(sp_dec),
    action_to_string_map_macro(sp_inc),
};

const std::string &actionToString(const Action &action) {
  return action_to_string_map.at(action);
}
