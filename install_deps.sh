#!/usr/bin/env bash
set -euo pipefail

PACKAGES=(
  podman
)

sudo apt-get update
sudo apt-get install -y "${PACKAGES[@]}"
