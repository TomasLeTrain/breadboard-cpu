#include "OutputCategory.h"

#include <cstdint>
#include <string>
#include <vector>

class Output {
  uint16_t data;
  std::vector<OutputCategory> categories;

public:
  Output(uint16_t data);

  uint16_t getData();
};
