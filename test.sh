#!/usr/bin/env bash
set -euo pipefail

panic() {
  echo "FATAL: $*" >&2
  exit 1
}

log() {
  echo "[run_tests] $*"
}

if [[ "${VERILOAD_TEST_IN_CONTAINER:-0}" == "1" ]]; then
  if [[ -z "${LOADER_CONTAINER:-}" ]]; then
    panic "LOADER_CONTAINER is required in container mode"
  fi
  if [[ ! -x "${LOADER_CONTAINER}" ]]; then
    panic "loader is missing or not executable in container: ${LOADER_CONTAINER}"
  fi

  TESTS_DIR="/work/tests"
  BUILD_DIR="${TESTS_DIR}/build"
  rm -rf "${BUILD_DIR}"
  make -C "${TESTS_DIR}" BUILD_DIR="${BUILD_DIR}"

  for elf in "${BUILD_DIR}/main" "${BUILD_DIR}/libfoo.so" "${BUILD_DIR}/libbar.so" "${BUILD_DIR}/libbaz.so"; do
    if readelf -d "${elf}" | grep -q "Shared library: \\[libc"; then
      panic "${elf} links libc; tests must use nolibc only"
    fi
  done

  cp "${BUILD_DIR}/libfoo.so" /lib/libfoo.so
  cp "${BUILD_DIR}/libbar.so" /lib/libbar.so
  cp "${BUILD_DIR}/libbaz.so" /lib/libbaz.so

  OUT="${BUILD_DIR}/single_case.log"
  RC=0
  if "${LOADER_CONTAINER}" "${BUILD_DIR}/main" >"${OUT}" 2>&1; then
    RC=0
  else
    RC=$?
  fi

  cat "${OUT}"

  if [[ "${RC}" -ne 0 ]]; then
    panic "single case exited with ${RC}"
  fi
  if ! grep -Fxq "PASS" "${OUT}"; then
    panic "single case did not print PASS"
  fi

  log "single case passed"
  exit 0
fi

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
LOADER="${LOADER:-./veriload}"
LOADER_CONTAINER="/work/${LOADER#./}"

log "running tests with loader ${LOADER} via run.sh"
./run.sh \
  env \
  VERILOAD_TEST_IN_CONTAINER=1 \
  LOADER_CONTAINER="${LOADER_CONTAINER}" \
  bash -eu /work/test.sh
