#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Formatting Backend ==="
cd "$ROOT_DIR/backend"
cargo fmt

echo "=== Formatting Frontend ==="
cd "$ROOT_DIR/frontend"
npm run format

echo "=== Formatting complete ==="
