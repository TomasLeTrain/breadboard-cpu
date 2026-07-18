#include "ActionDefinitions.hpp"

const std::map<Action, Output> action_to_output_map{
    {pc_cnt, Output(0, 0, 0, 0, 0, 0)},
    {mar_cnt, Output(0, 0, 0, 0, 0, 0)},
    {sp_dec, Output(0, 0, 0, 0, 0, 0)},
    {sp_inc, Output(0, 0, 0, 0, 0, 0)},
};

const Output &actionToOutput(const Action &action) {
  return action_to_output_map.at(action);
}
