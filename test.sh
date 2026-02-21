#!/usr/bin/env bash
set -euo pipefail

panic() {
  echo "FATAL: $*" >&2
  exit 1
}

log() {
  echo "[run_tests] $*"
}

run_success() {
  local name="$1"
  local bin="$2"
  local expect_exit="$3"
  local out="${BUILD_DIR}/${name}.log"
  local rc=0

  log "case ${name}: expect success exit=${expect_exit}"
  if "${LOADER_CONTAINER}" "${bin}" >"${out}" 2>&1; then
    rc=0
  else
    rc=$?
  fi

  if [[ "${rc}" -ne "${expect_exit}" ]]; then
    echo "--- ${name} output ---" >&2
    cat "${out}" >&2 || true
    panic "case ${name} exit ${rc}, expected ${expect_exit}"
  fi
}

run_fatal() {
  local name="$1"
  local bin="$2"
  local out="${BUILD_DIR}/${name}.log"
  local rc=0

  log "case ${name}: expect fatal"
  if "${LOADER_CONTAINER}" "${bin}" >"${out}" 2>&1; then
    rc=0
  else
    rc=$?
  fi

  if [[ "${rc}" -eq 0 ]]; then
    echo "--- ${name} output ---" >&2
    cat "${out}" >&2 || true
    panic "case ${name} unexpectedly succeeded"
  fi

  if [[ ! -s "${out}" ]]; then
    panic "case ${name} failed without any log output"
  fi
}

assert_musl_dynamic() {
  local name="$1"
  local bin="$2"

  if ! readelf -l "${bin}" | grep -q "Requesting program interpreter: /lib/ld-musl-x86_64.so.1"; then
    panic "case ${name} is not using musl dynamic interpreter"
  fi

  if ! readelf -d "${bin}" | grep -q "Shared library: \\[libc.musl-x86_64.so.1\\]"; then
    panic "case ${name} is not dynamically linked against musl libc"
  fi
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

  cp "${BUILD_DIR}/libvlfoo.so" /lib/
  cp "${BUILD_DIR}/libvlb.so" /lib/
  cp "${BUILD_DIR}/libvla.so" /lib/

  run_success "pos_minimal_pie" "${BUILD_DIR}/pos_minimal_pie" 0
  run_success "pos_one_dso" "${BUILD_DIR}/pos_one_dso" 7
  run_success "pos_multi_dso_bfs" "${BUILD_DIR}/pos_multi_dso_bfs" 12
  run_fatal "neg_missing_needed" "${BUILD_DIR}/neg_missing_needed"
  assert_musl_dynamic "pos_musl_dynamic" "${BUILD_DIR}/pos_musl_dynamic"
  run_success "pos_musl_dynamic" "${BUILD_DIR}/pos_musl_dynamic" 42

  log "all configured Alpine runtime tests passed"
  exit 0
fi

cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
LOADER="${LOADER:-./veriload}"

LOADER_CONTAINER="/work/${LOADER#./}"

log "running Alpine runtime tests with loader ${LOADER} via run.sh"
./run.sh \
  env \
  VERILOAD_TEST_IN_CONTAINER=1 \
  LOADER_CONTAINER="${LOADER_CONTAINER}" \
  bash -eu /work/test.sh
