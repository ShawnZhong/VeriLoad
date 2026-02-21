#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="veriload"

make container

if [[ $# -eq 0 ]]; then
  exec podman run --rm -it \
    -v "${PWD}:/work" \
    -w /work \
    "${IMAGE_NAME}" \
    sh
fi

exec podman run --rm \
  -v "${PWD}:/work" \
  -w /work \
  "${IMAGE_NAME}" \
  "$@"
