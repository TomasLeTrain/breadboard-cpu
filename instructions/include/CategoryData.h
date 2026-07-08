#pragma once

#include "OutputCategory.h"

class CategoryData {
  // has parent category and holds some data
  OutputCategory *parent;
  uint16_t data;

public:
  CategoryData(OutputCategory *parent, uint16_t data)
      : parent(parent), data(data) {
    assert(parent->dataValid(data));
  }
};
