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
void init(){
  std::vector<OutputCategory> categories;

  OutputCategory bout_cat(4, 0, "bout");
  OutputCategory write_cat(4, 3, "write");

  // CategoryData data(&categories[0], 0);

  Action pc_cnt("pc_cnt", {CategoryData(&bout_cat, 0)});
}
