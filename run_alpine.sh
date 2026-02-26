#!/usr/bin/env bash
set -euo pipefail

if ! command -v podman &>/dev/null; then
  sudo apt-get update
  sudo apt-get install -y podman
fi

IMAGE_NAME="alpine-veriload"
PODMAN_RUN_PREFIX="podman run --rm -it -v ${PWD}:/work -w /work ${IMAGE_NAME}"

podman build -q -t ${IMAGE_NAME} -f Containerfile .

if [[ $# -gt 0 ]]; then
  exec ${PODMAN_RUN_PREFIX} "$@"
fi

exec ${PODMAN_RUN_PREFIX} sh -lc '
    cd build
    ./veriload main libfoo.so libbar.so libbaz.so libunused.so libc.so
    ./veriload /bin/busybox libc.musl-x86_64.so.1
    ./veriload /usr/bin/file /usr/lib/libmagic.so.1 libc.musl-x86_64.so.1
    ./veriload /usr/bin/readelf /usr/lib/libctf-nobfd.so.0 /usr/lib/libz.so.1 /usr/lib/libzstd.so.1 /usr/lib/libsframe.so.2 libc.musl-x86_64.so.1
  '
