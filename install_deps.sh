#!/usr/bin/env bash
set -euo pipefail

PACKAGES=(
  podman
  curl
  unzip
)

sudo apt-get update
sudo apt-get install -y "${PACKAGES[@]}"
