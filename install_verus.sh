#!/usr/bin/env bash
set -euo pipefail

VERUS_VERSION="0.2026.02.15.61aa1bf"
RUST_TOOLCHAIN="1.93.0-x86_64-unknown-linux-gnu"
RUSTUP_BIN="$HOME/.cargo/bin/rustup"
ARCHIVE="verus-${VERUS_VERSION}-x86-linux.zip"
ARCHIVE_PATH=".verus/$ARCHIVE"
URL="https://github.com/verus-lang/verus/releases/download/release/${VERUS_VERSION}/${ARCHIVE}"

mkdir -p ".verus"
curl -fsSL -o "$ARCHIVE_PATH" "$URL"
unzip -qo "$ARCHIVE_PATH" -d ".verus"

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
"$RUSTUP_BIN" toolchain install "$RUST_TOOLCHAIN"
