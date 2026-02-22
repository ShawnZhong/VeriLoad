#!/usr/bin/env bash
set -euo pipefail

make
make -C tests

exec ./veriload \
  tests/build/main \
  tests/build/libfoo.so \
  tests/build/libbar.so \
  tests/build/libbaz.so \
  tests/build/libunused.so
