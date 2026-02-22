#!/usr/bin/env bash
set -euo pipefail

make

cd build

exec ./veriload \
  main \
  libfoo.so \
  libbar.so \
  libbaz.so \
  libunused.so
