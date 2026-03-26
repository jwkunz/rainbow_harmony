#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT_DIR"
wasm-pack build crates/web_wasm \
  --target web \
  --out-dir ../../web/pkg \
  --out-name rainbow_harmony_web_wasm
