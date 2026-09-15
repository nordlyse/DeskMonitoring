#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export PATH="${HOME}/.cargo/bin:/usr/bin:${PATH}"
python3 packaging/icon.py
cargo build --release
if [ -n "${CARGO_TARGET_DIR:-}" ] && [ ! -f target/release/desk-monitoring ]; then
  mkdir -p target/release
  cp "${CARGO_TARGET_DIR}/release/desk-monitoring" target/release/
fi
cargo install cargo-deb --locked --quiet
mkdir -p dist
cargo deb --no-build --output dist/
find target -name '*.deb' -exec cp {} dist/ \; 2>/dev/null || true
ls -la dist/*.deb
