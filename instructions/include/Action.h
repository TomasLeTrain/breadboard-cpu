#pragma once

#include <cstdint>
#include <string>
#include <vector>

#include "CategoryData.h"

class Action {
private:
  std::string name;
  std::vector<CategoryData> data;

public:
  Action(std::string name, std::vector<CategoryData> data);

  std::string toString();
  std::vector<CategoryData> getData();
};
