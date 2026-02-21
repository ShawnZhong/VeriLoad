#!/usr/bin/env bash
set -euo pipefail

make container

if [[ $# -eq 0 ]]; then
  exec podman run --rm -it \
    -v "${PWD}:/work" \
    -w /work \
    veriload \
    sh
fi

exec podman run --rm \
  -v "${PWD}:/work" \
  -w /work \
  veriload \
  "$@"
