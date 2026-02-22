#!/usr/bin/env bash
set -euo pipefail

VERUS_VERSION="0.2026.02.15.61aa1bf"
RUST_TOOLCHAIN="1.93.0-x86_64-unknown-linux-gnu"
ARCHIVE="verus-${VERUS_VERSION}-x86-linux.zip"
URL="https://github.com/verus-lang/verus/releases/download/release/${VERUS_VERSION}/${ARCHIVE}"
VERUS_ROOT=".verus"

mkdir -p "$VERUS_ROOT"
curl -fsSL -o "${VERUS_ROOT}/${ARCHIVE}" "$URL"
unzip -qo "${VERUS_ROOT}/${ARCHIVE}" -d "$VERUS_ROOT"

if ! command -v rustup &>/dev/null; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
fi
"$HOME/.cargo/bin/rustup" toolchain install "$RUST_TOOLCHAIN"
