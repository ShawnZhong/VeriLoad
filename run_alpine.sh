#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="veriload"

if ! command -v podman &>/dev/null; then
  sudo apt-get update
  sudo apt-get install -y podman
fi

podman build -q -t ${IMAGE_NAME} -f Containerfile .

if [[ $# -eq 0 ]]; then
  exec podman run --rm -it \
    -v "${PWD}:/work" \
    -w /work \
    "${IMAGE_NAME}" \
    sh -lc '
      cd build
      ./veriload /usr/bin/gcc libc.musl-x86_64.so.1
      ./veriload /bin/busybox libc.musl-x86_64.so.1
    '
fi

exec podman run --rm -it \
  -v "${PWD}:/work" \
  -w /work \
  "${IMAGE_NAME}" \
  "$@"
