#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Linting Backend ==="
cd "$ROOT_DIR/backend"
cargo clippy -- -D warnings

echo "=== Checking Backend Formatting ==="
cargo fmt -- --check

echo "=== Linting Frontend ==="
cd "$ROOT_DIR/frontend"
npm run lint

echo "=== Checking Frontend Formatting ==="
npm run format:check

echo "=== Running Svelte Check ==="
npm run check

echo "=== All lint checks passed ==="
