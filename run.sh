#!/usr/bin/env bash
set -euo pipefail

make

cd build

./veriload "$@" main libc_shim.so libfoo.so libbar.so libbaz.so libunused.so /usr/lib/x86_64-linux-musl/libc.so
