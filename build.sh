#!/bin/bash
set -e

# Build script for moond (without CMake)
# Based on CMakeLists.txt configuration

# Compiler settings
CC="${CC:-cc}"
CFLAGS="-std=c2x -I./include -I./src"
LDFLAGS=""

echo "Building moond-core library..."
$CC $CFLAGS -c src/core.c -o build/core.o
$CC $CFLAGS -c src/instr.c -o build/instr.o
$CC $CFLAGS -c src/decode.c -o build/decode.o
$CC $CFLAGS -c src/encode.c -o build/encode.o
$CC $CFLAGS -c src/parity.c -o build/parity.o

echo "Archiving moond-core library..."
ar rcs build/libmoond-core.a build/core.o build/instr.o build/decode.o build/encode.o build/parity.o

echo "Building decode_test executable..."
$CC $CFLAGS src/decode_test.c build/libmoond-core.a -o decode_test

echo "Build complete!"
