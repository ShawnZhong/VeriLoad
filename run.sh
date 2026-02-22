#!/usr/bin/env bash
set -euo pipefail

make

exec ./build/veriload \
  build/main \
  build/libfoo.so \
  build/libbar.so \
  build/libbaz.so \
  build/libunused.so
