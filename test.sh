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

assert_no_libc_linkage() {
  local name="$1"
  local elf="$2"

  if readelf -d "${elf}" | grep -q "Shared library: \\[libc"; then
    panic "case ${name} links libc, but first-stage tests must avoid libc"
  fi
}

assert_log_contains() {
  local name="$1"
  local pattern="$2"
  local out="${BUILD_DIR}/${name}.log"

  if ! grep -Fq "${pattern}" "${out}"; then
    echo "--- ${name} output ---" >&2
    cat "${out}" >&2 || true
    panic "case ${name} log does not contain: ${pattern}"
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

  assert_no_libc_linkage "libveriload_basic.so" "${BUILD_DIR}/libveriload_basic.so"
  assert_no_libc_linkage "libveriload_dep_a.so" "${BUILD_DIR}/libveriload_dep_a.so"
  assert_no_libc_linkage "libveriload_dep_b.so" "${BUILD_DIR}/libveriload_dep_b.so"
  assert_no_libc_linkage "libveriload_missing_top.so" "${BUILD_DIR}/libveriload_missing_top.so"
  assert_no_libc_linkage "libveriload_missing_leaf.so" "${BUILD_DIR}/libveriload_missing_leaf.so"
  assert_no_libc_linkage "pos_no_deps" "${BUILD_DIR}/pos_no_deps"
  assert_no_libc_linkage "pos_one_dep" "${BUILD_DIR}/pos_one_dep"
  assert_no_libc_linkage "pos_transitive_dep" "${BUILD_DIR}/pos_transitive_dep"
  assert_no_libc_linkage "neg_missing_dep" "${BUILD_DIR}/neg_missing_dep"

  cp "${BUILD_DIR}/libveriload_basic.so" /lib/
  cp "${BUILD_DIR}/libveriload_dep_a.so" /lib/
  cp "${BUILD_DIR}/libveriload_dep_b.so" /lib/
  cp "${BUILD_DIR}/libveriload_missing_top.so" /lib/
  rm -f /lib/libveriload_missing_leaf.so

  run_success "pos_no_deps" "${BUILD_DIR}/pos_no_deps" 0
  run_success "pos_one_dep" "${BUILD_DIR}/pos_one_dep" 0
  assert_log_contains "pos_one_dep" "[veriload] load DSO /lib/libveriload_basic.so"

  run_success "pos_transitive_dep" "${BUILD_DIR}/pos_transitive_dep" 0
  assert_log_contains "pos_transitive_dep" "[veriload] load DSO /lib/libveriload_dep_a.so"
  assert_log_contains "pos_transitive_dep" "[veriload] load DSO /lib/libveriload_dep_b.so"

  run_fatal "neg_missing_dep" "${BUILD_DIR}/neg_missing_dep"
  assert_log_contains "neg_missing_dep" "[veriload] load DSO /lib/libveriload_missing_top.so"
  assert_log_contains "neg_missing_dep" "/lib/libveriload_missing_leaf.so"

  log "all configured stage-1 dependency tests passed"
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
