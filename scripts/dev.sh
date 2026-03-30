#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

cleanup() {
    echo ""
    echo "=== Shutting down dev servers ==="
    kill $BACKEND_PID $FRONTEND_PID 2>/dev/null
}
trap cleanup EXIT

echo "=== Starting Backend (port 3000) ==="
cd "$ROOT_DIR/backend"
cargo run &
BACKEND_PID=$!

echo "=== Starting Frontend Dev Server (port 5173) ==="
cd "$ROOT_DIR/frontend"
npm run dev &
FRONTEND_PID=$!

echo "=== Dev servers running. Frontend: http://localhost:5173 ==="
wait
