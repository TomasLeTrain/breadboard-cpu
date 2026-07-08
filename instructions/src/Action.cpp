#include "Action.h"

Action::Action(std::string name, std::vector<CategoryData> data)
    : name(name), data(data) {}
std::string Action::toString() { return name; }
std::vector<CategoryData> Action::getData() { return data; }
