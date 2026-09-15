#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

export PATH="${HOME}/.cargo/bin:/usr/local/bin:/opt/homebrew/bin:${PATH}"
export PKG_CONFIG_PATH="$(brew --prefix 2>/dev/null)/lib/pkgconfig:${PKG_CONFIG_PATH:-}"

python3 packaging/icon.py
cargo build --release
python3 packaging/bundle_macos.py
