#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

echo "=== Setting up git hooks ==="
cd "$ROOT_DIR"
git config core.hooksPath .githooks

echo "=== Installing frontend dependencies ==="
cd "$ROOT_DIR/frontend"
npm install

echo "=== Creating .env file (if not exists) ==="
cd "$ROOT_DIR"
if [ ! -f .env ]; then
    cp .env.example .env
    echo "Created .env from .env.example"
else
    echo ".env already exists, skipping"
fi

echo "=== Setup complete ==="
