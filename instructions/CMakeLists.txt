# Set the minimum version of CMake that can be used
# To find the cmake version run
# $ cmake --version
cmake_minimum_required(VERSION 4.1)

# Set the project name
project (instructions)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Create a sources variable with a link to all cpp files to compile
set(SOURCES
    src/main.cpp
    src/Action.cpp
)

# Add an executable with the above sources
add_executable(instructions ${SOURCES})

# Set the directories that should be included in the build command for this target
# when running g++ these will be included as -I/directory/path/
target_include_directories(instructions
    PRIVATE 
        ${PROJECT_SOURCE_DIR}/include
)
