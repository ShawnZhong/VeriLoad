#!/usr/bin/env bash
set -euo pipefail

VERUS_VERSION="0.2026.02.15.61aa1bf"
RUST_TOOLCHAIN="1.93.0-x86_64-unknown-linux-gnu"
if command -v rustup >/dev/null 2>&1; then
  RUSTUP_BIN="$(command -v rustup)"
else
  RUSTUP_BIN="$HOME/.cargo/bin/rustup"
fi
ARCHIVE="verus-${VERUS_VERSION}-x86-linux.zip"
ARCHIVE_PATH=".verus/$ARCHIVE"
URL="https://github.com/verus-lang/verus/releases/download/release/${VERUS_VERSION}/${ARCHIVE}"
VERUS_ROOT=".verus"
VERUS_DIR="${VERUS_ROOT}/verus-x86-linux"

mkdir -p "$VERUS_ROOT"

if [ ! -x "${VERUS_DIR}/verus" ]; then
  if [ ! -f "$ARCHIVE_PATH" ]; then
    curl -fsSL -o "$ARCHIVE_PATH" "$URL"
  fi
  unzip -qo "$ARCHIVE_PATH" -d "$VERUS_ROOT"
fi

if [ ! -x "$RUSTUP_BIN" ]; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  RUSTUP_BIN="$HOME/.cargo/bin/rustup"
fi

if ! "$RUSTUP_BIN" toolchain list | grep -q "^${RUST_TOOLCHAIN}\\b"; then
  "$RUSTUP_BIN" toolchain install "$RUST_TOOLCHAIN"
fi
