project(moond)
cmake_minimum_required(VERSION 3.31)

# Core library (includes shared instruction types, decoder, encoder, and parity)
add_library(moond-core STATIC 
    src/core.c 
    src/instr.c 
    src/decode.c 
    src/encode.c
    src/parity.c)
set_property(TARGET moond-core PROPERTY C_STANDARD 23)
set_property(TARGET moond-core PROPERTY C_STANDARD_REQUIRED ON)
target_include_directories(moond-core PUBLIC ${CMAKE_CURRENT_SOURCE_DIR}/include ${CMAKE_CURRENT_SOURCE_DIR}/src)

# Decoder test executable
add_executable(decode_test src/decode_test.c)
target_link_libraries(decode_test moond-core)
set_property(TARGET decode_test PROPERTY C_STANDARD 23)

# Encoder test executable
add_executable(encode_test src/encode_test.c)
target_link_libraries(encode_test moond-core)
set_property(TARGET encode_test PROPERTY C_STANDARD 23)

# Bit manipulation helper test
add_executable(bits_test src/bits_test.c)
target_link_libraries(bits_test moond-core)
set_property(TARGET bits_test PROPERTY C_STANDARD 23)