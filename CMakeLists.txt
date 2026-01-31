project(moond)
cmake_minimum_required(VERSION 3.31)

# Core library
add_library(moond-core STATIC src/core.c src/decode.c)
set_property(TARGET moond-core PROPERTY C_STANDARD 23)
set_property(TARGET moond-core PROPERTY C_STANDARD_REQUIRED ON)
target_include_directories(moond-core PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include ${CMAKE_CURRENT_SOURCE_DIR}/src)

# Decoder test executable
add_executable(decode_test src/decode_test.c)
target_link_libraries(decode_test moond-core)
set_property(TARGET decode_test PROPERTY C_STANDARD 23)