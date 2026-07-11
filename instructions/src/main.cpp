#include <iostream>
#include <vector>

#include "Action.h"
#include "CategoryData.h"
#include "OutputCategory.h"

int main() {
  // ActionList list;
  // list.pc_cnt = Action{};
  // list.mar_cnt = Action{};
  //
  // std::cout << "Hello world" << std::endl;

  // std::vector<OutputCategory> categories;
}

// TODO: define some state class to hold all the categories and actions
void init() {
  std::vector<OutputCategory> categories;

  // TODO: be able to look up category by name through state class
  // const OutputCategory& bout_cat =

  OutputCategory bout_cat(4, 0, "bout");
  OutputCategory write_cat(4, 3, "write");
  categories.push_back(bout_cat);
  categories.push_back(write_cat);

  Action pc_cnt("pc_cnt", {{&bout_cat, 0}, {&write_cat, 0}});
}
