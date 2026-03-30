#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Building Frontend ==="
cd "$ROOT_DIR/frontend"
npm ci
npm run build

echo "=== Building Backend ==="
cd "$ROOT_DIR/backend"
cargo build --release

echo "=== Copying frontend build to backend static dir ==="
rm -rf "$ROOT_DIR/backend/static"
cp -r "$ROOT_DIR/frontend/build" "$ROOT_DIR/backend/static"

echo "=== Build complete ==="
