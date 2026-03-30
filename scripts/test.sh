#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Running Backend Tests ==="
cd "$ROOT_DIR/backend"
cargo test

echo "=== Running Frontend Tests ==="
cd "$ROOT_DIR/frontend"
npm test

echo "=== All tests passed ==="
