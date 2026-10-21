#!/usr/bin/env bash
# Start ffplayout in the foreground for development / debugging.
#   ./start.sh            -> backend :8787 + vite dev server :5757
#   ./start.sh backend    -> backend only (frontend served from frontend/dist)
#   ./start.sh frontend   -> vite dev server only
set -euo pipefail

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

PY="${PY:-$PWD/.venv/bin/python}"
MODE="${1:-all}"
HOST="${FFPU_HOST:-127.0.0.1}"
PORT="${FFPU_PORT:-8787}"

[[ -x "$PY" ]] || { echo "no venv - create it first:"; echo "  python3 -m venv .venv && .venv/bin/pip install -r requirements.txt"; exit 1; }

start_backend() {
    echo ">>> backend  http://$HOST:$PORT"
    FFPU_HOST="$HOST" FFPU_PORT="$PORT" "$PY" server.py
}

start_frontend() {
    command -v npm >/dev/null || { echo "npm not found - skipping vite"; return 0; }
    # package.json is at the APP ROOT (vite.config.ts sets root: './frontend'),
    # so npm must run from here, not from frontend/.
    [[ -f "$PWD/package.json" ]] || { echo "no package.json in $PWD"; return 1; }
    [[ -d node_modules ]] || npm install
    echo ">>> frontend http://127.0.0.1:5757  (proxies /api to :$PORT)"
    npm run dev
}

case "$MODE" in
    backend)  start_backend ;;
    frontend) start_frontend ;;
    all)      start_frontend & sleep 3; start_backend ;;
    *)        echo "usage: $0 [all|backend|frontend]"; exit 1 ;;
esac