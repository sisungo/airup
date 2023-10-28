cmake_minimum_required(VERSION 3.18)
cmake_policy(SET CMP0048 NEW)
project(airup_sdk VERSION 0.4.0 LANGUAGES C CXX)

set(CMAKE_C_STANDARD 23)
set(CMAKE_CXX_STANDARD 20)

add_compile_options(-Wall -Wextra -pedantic -Werror)

find_package(RapidJSON CONFIG REQUIRED)

include_directories(include/)
file(GLOB_RECURSE AIRUP_SDK_SRC
        src/*.cpp
)
add_library(airup_sdk STATIC ${AIRUP_SDK_SRC})
target_link_libraries(airup_sdk PRIVATE rapidjson)