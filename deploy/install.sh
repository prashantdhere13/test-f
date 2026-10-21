#!/usr/bin/env bash
# =============================================================================
#  ffplayout - Ubuntu installer
#  Installs system packages, builds the Vue frontend, creates the Python venv
#  and (optionally) registers a systemd service behind an nginx reverse proxy.
#
#  Usage:   sudo ./install.sh
#  Re-run: sudo ./install.sh          (idempotent - safe to re-run after edits)
# =============================================================================
set -euo pipefail

# --- overridable settings ---------------------------------------------------
APP_DIR="${APP_DIR:-/opt/ffplayout}"
APP_USER="${APP_USER:-ffpu}"
FRONTEND_DIR="${FRONTEND_DIR:-/var/www/ffplayout}"
BIND_ADDR="${BIND_ADDR:-127.0.0.1}"
BIND_PORT="${BIND_PORT:-8787}"
ENABLE_SERVICE="${ENABLE_SERVICE:-1}"

# SRC_DIR = the ffplayout checkout that contains server.py.
# Override explicitly if auto-detection picks the wrong one:
#   sudo SRC_DIR=/root/ffplayout ./install.sh
SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -z "${SRC_DIR:-}" ]]; then
    for cand in "$SELF_DIR" "$SELF_DIR/.." "$SELF_DIR/../ffplayout" \
                "$SELF_DIR/ffplayout" "$HOME/ffplayout" "/opt/ffplayout-src"; do
        if [[ -f "$cand/server.py" ]]; then SRC_DIR="$(cd "$cand" && pwd)"; break; fi
    done
fi
SRC_DIR="${SRC_DIR:-}"

log()  { printf '\033[1;32m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m warn:\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

[[ $EUID -eq 0 ]] || die "run with sudo:  sudo ./install.sh"
[[ -n "$SRC_DIR" && -f "$SRC_DIR/server.py" ]] \
    || die "could not find server.py - set SRC_DIR=/path/to/ffplayout"
log "Source directory : $SRC_DIR"
log "Install directory: $APP_DIR"

# =============================================================================
# 1. System packages
# =============================================================================
log "Installing system packages (apt)"
export DEBIAN_FRONTEND=noninteractive
apt-get update -y
apt-get install -y --no-install-recommends \
    ffmpeg \
    python3 \
    python3-venv \
    python3-pip \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    ca-certificates \
    git \
    nginx \
    tzdata \
    fonts-dejavu-core \
    fonts-liberation \
    rsync

# FFmpeg >= 5 is required (7.2+ for WebVTT subtitles / SRT output).
ffmpeg -hide_banner -version | head -n1
ffmpeg -hide_banner -filters 2>/dev/null | grep -qw zmq \
    && log "ffmpeg has libzmq (dynamic text overlay enabled)" \
    || warn "ffmpeg built WITHOUT libzmq - text overlay via JSON-RPC will not work"

# Node.js 20+ for the frontend build.
if ! command -v node >/dev/null 2>&1; then
    log "Installing Node.js 22 LTS via NodeSource"
    curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
    apt-get install -y nodejs
fi
node -v; npm -v

# =============================================================================
# 2. Service account + directories
# =============================================================================
log "Creating service user '$APP_USER' and directories"
id -u "$APP_USER" >/dev/null 2>&1 || useradd --system \
    --home-dir "$APP_DIR" --shell /usr/sbin/nologin "$APP_USER"

install -d -m 0755 "$APP_DIR"
install -d -m 0755 -o "$APP_USER" -g "$APP_USER" "$APP_DIR/demo_data"

# Runtime dirs the app writes to (DATA_DIR = $APP_DIR/demo_data)
for d in channel channels playlists public storage; do
    install -d -m 0775 -o "$APP_USER" -g "$APP_USER" "$APP_DIR/demo_data/$d"
done
install -d -m 0775 -o "$APP_USER" -g "$APP_USER" "$APP_DIR/demo_data/logs"
install -d -m 0755 "$FRONTEND_DIR"

# =============================================================================
# 3. Copy the application (excluding build/dev cruft)
# =============================================================================
log "Syncing application files to $APP_DIR"
rsync -a --delete \
    --exclude '.git/' \
    --exclude '.venv/' \
    --exclude 'node_modules/' \
    --exclude 'target/' \
    --exclude '__pycache__/' \
    --exclude '.vite/' \
    --exclude '*.log' \
    --exclude 'server_stdout*.log' \
    --exclude 'server_stderr*.log' \
    --exclude 'demo_data/' \
    "$SRC_DIR/" "$APP_DIR/"

# Fresh DB on first install; preserved across re-runs.
if [[ ! -f "$APP_DIR/demo_data/db.json" ]]; then
    log "No existing db.json - it will be auto-created on first start"
fi
chown -R "$APP_USER:$APP_USER" "$APP_DIR"

# =============================================================================
# 4. Python virtualenv
# =============================================================================
log "Creating Python venv at $APP_DIR/.venv"
sudo -u "$APP_USER" python3 -m venv "$APP_DIR/.venv"
sudo -u "$APP_USER" "$APP_DIR/.venv/bin/pip" install -q --upgrade pip setuptools wheel
sudo -u "$APP_USER" "$APP_DIR/.venv/bin/pip" install -q -r "$APP_DIR/requirements.txt"
sudo -u "$APP_USER" "$APP_DIR/.venv/bin/python" -c \
    "import fastapi, uvicorn, psutil, jose, bcrypt, openpyxl, threefive; print('python deps OK')"

# =============================================================================
# 5. Build the Vue frontend
# =============================================================================
if [[ -f "$APP_DIR/package.json" && -d "$APP_DIR/frontend" ]]; then
    # package.json is at the APP ROOT, not in frontend/ - vite.config.ts sets
    # `root: './frontend'`, so npm must run from $APP_DIR.
    log "Building frontend (npm ci && npm run build)"
    cd "$APP_DIR"
    sudo -u "$APP_USER" -H npm ci --no-audit --no-fund --cache "$APP_DIR/.npm-cache"
    sudo -u "$APP_USER" -H npm run build --cache "$APP_DIR/.npm-cache"
    [[ -f "$APP_DIR/frontend/dist/index.html" ]] \
        || die "frontend build produced no frontend/dist/index.html"
    log "frontend built -> $APP_DIR/frontend/dist (server.py serves it)"
else
    die "no $APP_DIR/package.json - the Vue sources did not get copied"
fi

# =============================================================================
# 6. Environment file
# =============================================================================
ENV_FILE=/etc/ffplayout/ffplayout.env
install -d -m 0755 /etc/ffplayout
if [[ ! -f "$ENV_FILE" ]]; then
    log "Writing $ENV_FILE"
    SECRET=$(head -c 32 /dev/urandom | base64 | tr -d '/+=')
    cat >"$ENV_FILE" <<EOF
# ffplayout runtime environment
FFPU_HOST=$BIND_ADDR
FFPU_PORT=$BIND_PORT
FFPU_LOG_LEVEL=info

# Where the JSON database, media, playlists, HLS output and logs live.
# Keep this path writable by $APP_USER (systemd ReadWritePaths must match).
FFPU_DATA_DIR=$APP_DIR/demo_data
FFPU_DB_FILE=$APP_DIR/demo_data/db.json

# Signing key for the session JWTs. Rotating it invalidates all sessions.
FFPU_SECRET_KEY=$SECRET

# Optional: absolute path to Blender for NNL dynamic graphics
#BLENDER_BIN=/usr/local/blender/blender
EOF
    chmod 0640 "$ENV_FILE"
    chown root:"$APP_USER" "$ENV_FILE"
    log "generated FFPU_SECRET_KEY in $ENV_FILE - keep a copy, do not commit it"
else
    log "$ENV_FILE already exists - keeping it"
fi

# =============================================================================
# 7. systemd unit
# =============================================================================
if [[ "$ENABLE_SERVICE" == "1" ]]; then
    log "Installing systemd unit"
    sed -e "s|@APP_DIR@|$APP_DIR|g" \
        -e "s|@APP_USER@|$APP_USER|g" \
        -e "s|@BIND_ADDR@|$BIND_ADDR|g" \
        -e "s|@BIND_PORT@|$BIND_PORT|g" \
        "$SELF_DIR/ffplayout-playout.service" \
        >/etc/systemd/system/ffplayout-playout.service

    systemctl daemon-reload
    systemctl enable --now ffplayout-playout.service
    sleep 3
    systemctl --no-pager --full status ffplayout-playout.service || true
fi

# =============================================================================
# 8. nginx reverse proxy (optional - set SKIP_NGINX=1 to skip)
# =============================================================================
if [[ "${SKIP_NGINX:-0}" != "1" ]]; then
    log "Installing nginx site"
    sed -e "s|@BIND_ADDR@|$BIND_ADDR|g" \
        -e "s|@BIND_PORT@|$BIND_PORT|g" \
        "$SELF_DIR/nginx-ffplayout.conf" \
        >/etc/nginx/sites-available/ffplayout
    ln -sf /etc/nginx/sites-available/ffplayout /etc/nginx/sites-enabled/ffplayout
    rm -f /etc/nginx/sites-enabled/default
    nginx -t && systemctl reload nginx
fi

cat <<EOF

===============================================================================
 ffplayout installed.

   APP_DIR      : $APP_DIR
   DATABASE     : $APP_DIR/demo_data/db.json
   Frontend     : $APP_DIR/frontend/dist
   Listening on : http://$BIND_ADDR:$BIND_PORT
   Web UI       : http://<server-ip>:8787   (via nginx, if enabled)

   Login        : admin / admin      -> CHANGE IMMEDIATELY

 Next steps:
   1. Create the database:   sudo -u $APP_USER $APP_DIR/.venv/bin/python $APP_DIR/db_admin.py --init
   2. Point your media at   $APP_DIR/demo_data/storage/<Channel Name>/
   3. Logs:  journalctl -u ffplayout-playout -f
   4. TLS:   certbot --nginx -d your.domain
===============================================================================
EOF