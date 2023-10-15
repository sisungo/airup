# `CMakeLists.txt` for the Airup C/C++ SDK

## Set CMake Policy
cmake_minimum_required(VERSION 3.26)
cmake_policy(SET CMP0048 NEW)

## Project Information
project(airup_sdk VERSION 0.4.0.0 LANGUAGES C CXX)

## Essential Compiler Options
add_compile_options(-Wall -Wextra -pedantic -Werror)
set(CMAKE_C_STANDARD 23)
set(CMAKE_CXX_STANDARD 20)
include_directories(include)

## Compiler Options for "Debug" Mode
set(CMAKE_C_FLAGS_DEBUG "${CMAKE_C_FLAGS_DEBUG} -g -O0")
set(CMAKE_CXX_FLAGS_DEBUG "${CMAKE_CXX_FLAGS_DEBUG} -g -O0")

## Compiler Options for "Release" Mode
set(CMAKE_C_FLAGS_RELEASE "${CMAKE_C_FLAGS_RELEASE} -O3")
set(CMAKE_CXX_FLAGS_RELEASE "${CMAKE_CXX_FLAGS_RELEASE} -O3")

## Target: `libairup_sdk.so`
file(
    GLOB_RECURSE AIRUP_SDK_LIB_SRC
    src/*.cxx
)
add_library(airup_sdk SHARED ${AIRUP_SDK_LIB_SRC})
