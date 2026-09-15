#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
docker run --rm --platform linux/amd64 \
  -v "$ROOT":/src -w /src \
  -e CARGO_TARGET_DIR=/src/target-linux \
  -e CARGO_HOME=/tmp/cargo-home \
  ubuntu:24.04 \
  bash -lc '
    set -euo pipefail
    apt-get update
    apt-get install -y --no-install-recommends \
      ca-certificates curl build-essential pkg-config python3 \
      libgtk-4-dev libssl-dev
    curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    . "$HOME/.cargo/env"
    python3 packaging/icon.py
    cargo build --release
    cargo install cargo-deb --locked
    mkdir -p dist
    cargo deb --no-build --output dist/
    ls -la dist
  '
