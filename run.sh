#!/usr/bin/env bash
set -euo pipefail

make

if [[ -f /lib/libc.musl-x86_64.so.1 ]]; then
  libc_path="/lib/libc.musl-x86_64.so.1"
elif [[ -f /usr/lib/x86_64-linux-musl/libc.so ]]; then
  libc_path="/usr/lib/x86_64-linux-musl/libc.so"
else
  echo "could not find musl libc" >&2
  exit 1
fi

cd build && ./veriload "$@" main libc_shim.so libfoo.so libbar.so libbaz.so libunused.so "$libc_path"
