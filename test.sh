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

assert_et_dyn() {
  local bin="$1"
  if ! readelf -h "${bin}" | grep -q "Type:[[:space:]]*DYN"; then
    panic "fixture is not ET_DYN: ${bin}"
  fi
}

require_src() {
  local src="$1"
  if [[ ! -f "${TESTS_DIR}/${src}" ]]; then
    panic "missing test source: ${TESTS_DIR}/${src}"
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
  BUILD_DIR="/tmp/veriload-tests"
  rm -rf "${BUILD_DIR}"
  mkdir -p "${BUILD_DIR}"

  require_src "pos_minimal_pie.S"
  require_src "libvlfoo.S"
  require_src "pos_one_dso.S"
  require_src "libvlb.S"
  require_src "libvla.S"
  require_src "pos_multi_dso_bfs.S"
  require_src "neg_missing_needed.S"

  cc -nostdlib -fPIC -shared -Wl,-soname,libvlfoo.so -o "${BUILD_DIR}/libvlfoo.so" "${TESTS_DIR}/libvlfoo.S"
  cc -nostdlib -fPIC -shared -Wl,-soname,libvlb.so -o "${BUILD_DIR}/libvlb.so" "${TESTS_DIR}/libvlb.S"
  cc -nostdlib -fPIC -shared -Wl,-soname,libvla.so -o "${BUILD_DIR}/libvla.so" "${TESTS_DIR}/libvla.S" -L"${BUILD_DIR}" -lvlb
  cc -nostdlib -fPIC -shared -Wl,-soname,libvlmissing.so -o "${BUILD_DIR}/libvlmissing.so" "${TESTS_DIR}/libvlfoo.S"

  cc -nostdlib -fPIE -pie -Wl,-e,_start -o "${BUILD_DIR}/pos_minimal_pie" "${TESTS_DIR}/pos_minimal_pie.S"
  cc -nostdlib -fPIE -pie -Wl,-e,_start -o "${BUILD_DIR}/pos_one_dso" "${TESTS_DIR}/pos_one_dso.S" -L"${BUILD_DIR}" -lvlfoo
  cc -nostdlib -fPIE -pie -Wl,-e,_start -Wl,-rpath-link,"${BUILD_DIR}" -o "${BUILD_DIR}/pos_multi_dso_bfs" "${TESTS_DIR}/pos_multi_dso_bfs.S" -L"${BUILD_DIR}" -lvla
  cc -nostdlib -fPIE -pie -Wl,-e,_start -o "${BUILD_DIR}/neg_missing_needed" "${TESTS_DIR}/neg_missing_needed.S" -L"${BUILD_DIR}" -lvlmissing

  assert_et_dyn "${BUILD_DIR}/pos_minimal_pie"
  assert_et_dyn "${BUILD_DIR}/pos_one_dso"
  assert_et_dyn "${BUILD_DIR}/pos_multi_dso_bfs"
  assert_et_dyn "${BUILD_DIR}/neg_missing_needed"

  cp "${BUILD_DIR}/libvlfoo.so" /lib/
  cp "${BUILD_DIR}/libvlb.so" /lib/
  cp "${BUILD_DIR}/libvla.so" /lib/

  run_success "pos_minimal_pie" "${BUILD_DIR}/pos_minimal_pie" 0
  run_success "pos_one_dso" "${BUILD_DIR}/pos_one_dso" 7
  run_success "pos_multi_dso_bfs" "${BUILD_DIR}/pos_multi_dso_bfs" 12
  run_fatal "neg_missing_needed" "${BUILD_DIR}/neg_missing_needed"

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
