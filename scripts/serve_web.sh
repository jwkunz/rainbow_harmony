#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${1:-8000}"

"$ROOT_DIR/scripts/build_wasm.sh"

cd "$ROOT_DIR/web"
echo "Serving Rainbow Harmony web UI at http://localhost:${PORT}"
python3 -m http.server "$PORT"
