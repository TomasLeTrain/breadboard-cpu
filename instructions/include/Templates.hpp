#pragma once

#include "Action.hpp"
#include "Opcode.hpp"
#include <vector>

using StepTemplateType = std::array<Action, 10>;
using IstrTemplateType = std::array<StepTemplateType, 16>;
