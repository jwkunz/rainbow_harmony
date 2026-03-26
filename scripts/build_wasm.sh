#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT_DIR"
wasm-pack build crates/web_wasm \
  --target web \
  --out-dir ../../web/pkg \
  --out-name rainbow_harmony_web_wasm

cp "$ROOT_DIR/rainbow_harmony_logo.png" "$ROOT_DIR/web/rainbow_harmony_logo.png"
