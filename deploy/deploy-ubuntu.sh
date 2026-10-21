#!/usr/bin/env bash
# =============================================================================
#  ffplayout - self-contained Ubuntu installer
#
#  One file. Assumes server.py + frontend/ are already in APP_DIR
#  (copied there by copy-to-ubuntu.ps1).
#
#      ssh root@server
#      cd /root/ffplayout/deploy
#      chmod +x deploy-ubuntu.sh
#      sudo ./deploy-ubuntu.sh
#
#  Overrides:  APP_DIR APP_USER BIND_ADDR BIND_PORT SKIP_NGINX ENABLE_SERVICE
#  Idempotent - safe to re-run after pulling a code update.
# =============================================================================
set -euo pipefail

APP_DIR="${APP_DIR:-/root/ffplayout}"
APP_USER="${APP_USER:-ffpu}"
BIND_ADDR="${BIND_ADDR:-127.0.0.1}"
BIND_PORT="${BIND_PORT:-8787}"
DATA_DIR="${DATA_DIR:-$APP_DIR/demo_data}"
ENABLE_SERVICE="${ENABLE_SERVICE:-1}"
SKIP_NGINX="${SKIP_NGINX:-0}"

log()  { printf '\033[1;32m==>\033[0m %s\n' "$*"; }
step() { printf '\n\033[1;36m--- %s ---\033[0m\n' "$*"; }
warn() { printf '\033[1;33m warn:\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

[[ $EUID -eq 0 ]] || die "run with sudo:  sudo ./deploy-ubuntu.sh"
[[ -f "$APP_DIR/server.py" ]] \
    || die "server.py not found in $APP_DIR - copy the project there first"

log "APP_DIR=$APP_DIR  USER=$APP_USER  BIND=$BIND_ADDR:$BIND_PORT  DATA=$DATA_DIR"

# =============================================================================
step "1/9  System packages"
# =============================================================================
export DEBIAN_FRONTEND=noninteractive
apt-get update -y
apt-get install -y --no-install-recommends \
    ffmpeg python3 python3-venv python3-pip \
    build-essential pkg-config libssl-dev \
    curl ca-certificates git nginx \
    tzdata fonts-dejavu-core fonts-liberation

ffmpeg -hide_banner -version | head -n1
if ffmpeg -hide_banner -filters 2>/dev/null | grep -qw zmq; then
    log "ffmpeg has libzmq - dynamic text overlay enabled"
else
    warn "ffmpeg WITHOUT libzmq - text overlay via JSON-RPC will not work"
    warn "  install a build with --enable-libzmq if you need it"
fi

# =============================================================================
step "2/9  Node.js 20+ (frontend build)"
# =============================================================================
if ! command -v node >/dev/null 2>&1 || [[ "$(node -v | tr -d 'v' | cut -d. -f1)" -lt 20 ]]; then
    log "Installing Node.js 22 LTS via NodeSource"
    curl -fsSL https://deb.nodesource.com/setup_22.x | bash -
    apt-get install -y nodejs
fi
log "node $(node -v)   npm $(npm -v)"

# =============================================================================
step "3/9  Service user and directories"
# =============================================================================
id -u "$APP_USER" >/dev/null 2>&1 || \
    useradd --system --home-dir "$APP_DIR" --shell /usr/sbin/nologin "$APP_USER"
log "user $APP_USER ready"

mkdir -p "$DATA_DIR"
for d in channel channels playlists public storage backups; do
    mkdir -p "$DATA_DIR/$d"
done
mkdir -p "$DATA_DIR/logs" "$DATA_DIR/logo" "$DATA_DIR/text" "$DATA_DIR/subtitles"
chown -R "$APP_USER:$APP_USER" "$APP_DIR"

# =============================================================================
step "4/9  Python virtualenv"
# =============================================================================
REQ="$APP_DIR/requirements.txt"
[[ -f "$REQ" ]] || cat >"$REQ" <<'EOF'
fastapi==0.141.1
starlette==1.6.0
uvicorn==0.52.4
python-multipart==0.0.32
pydantic==2.13.4
python-jose==3.5.0
bcrypt==4.2.1
passlib==1.7.4
psutil==7.2.2
openpyxl==3.1.5
threefive==3.0.95
pytz==2026.3.post1
tzdata==2026.3
aiofiles==25.1.0
EOF

[[ -d "$APP_DIR/.venv" ]] || sudo -u "$APP_USER" python3 -m venv "$APP_DIR/.venv"
sudo -u "$APP_USER" "$APP_DIR/.venv/bin/pip" install -q --upgrade pip setuptools wheel
sudo -u "$APP_USER" "$APP_DIR/.venv/bin/pip" install -q -r "$REQ"
sudo -u "$APP_USER" "$APP_DIR/.venv/bin/python" - <<'PY'
import fastapi, uvicorn, psutil, jose, bcrypt, openpyxl, threefive
print("    python deps OK")
PY

# =============================================================================
step "5/9  Build the frontend"
# =============================================================================
# NOTE: package.json lives at the APP ROOT, not in frontend/. vite.config.ts
# sets `root: './frontend'`, so npm must run from $APP_DIR and the output
# lands in $APP_DIR/frontend/dist.
if [[ -f "$APP_DIR/package.json" && -d "$APP_DIR/frontend" ]]; then
    cd "$APP_DIR"
    # Debian's nodejs has no enforced prefix; npm drops to root when it cannot
    # write ~/.npm - point the cache somewhere the app user owns.
    sudo -u "$APP_USER" -H npm ci --no-audit --no-fund --cache "$APP_DIR/.npm-cache"
    sudo -u "$APP_USER" -H npm run build --cache "$APP_DIR/.npm-cache"
    [[ -f "$APP_DIR/frontend/dist/index.html" ]] \
        || die "npm run build produced no frontend/dist/index.html"
    chown -R "$APP_USER:$APP_USER" "$APP_DIR/frontend/dist"
    log "frontend built -> $APP_DIR/frontend/dist (server.py serves it on /)"
else
    die "no $APP_DIR/package.json - the Vue sources did not get copied. Re-run copy-to-ubuntu.ps1"
fi

# =============================================================================
step "6/9  Environment file"
# =============================================================================
mkdir -p /etc/ffplayout
ENV_FILE=/etc/ffplayout/ffplayout.env
if [[ ! -f "$ENV_FILE" ]]; then
    cat >"$ENV_FILE" <<EOF
FFPU_HOST=$BIND_ADDR
FFPU_PORT=$BIND_PORT
FFPU_LOG_LEVEL=info
FFPU_DATA_DIR=$DATA_DIR
FFPU_DB_FILE=$DATA_DIR/db.json
#BLENDER_BIN=/snap/bin/blender
EOF
    chmod 0640 "$ENV_FILE"; chown root:"$APP_USER" "$ENV_FILE"
    log "wrote $ENV_FILE"
else
    log "$ENV_FILE exists - keeping"
fi

# =============================================================================
step "7/9  systemd unit"
# =============================================================================
if [[ "$ENABLE_SERVICE" == "1" ]]; then
    cat >/etc/systemd/system/ffplayout-playout.service <<EOF
[Unit]
Description=ffplayout playout server (FastAPI + ffmpeg)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=$APP_USER
Group=$APP_USER
WorkingDirectory=$APP_DIR
EnvironmentFile=-$ENV_FILE
Environment=FFPU_HOST=$BIND_ADDR
Environment=FFPU_PORT=$BIND_PORT
Environment=PYTHONUNBUFFERED=1
ExecStart=$APP_DIR/.venv/bin/python $APP_DIR/server.py
Restart=always
RestartSec=5
TimeoutStopSec=20
KillMode=mixed
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=full
ProtectHome=read-only
ReadWritePaths=$DATA_DIR $APP_DIR/.npm-cache
LimitNOFILE=65536

[Install]
WantedBy=multi-user.target
EOF
    systemctl daemon-reload
    systemctl enable ffplayout-playout.service
    log "unit installed (not started yet - database first)"
else
    log "ENABLE_SERVICE=0 - skipping systemd"
fi

# =============================================================================
step "8/9  nginx reverse proxy"
# =============================================================================
if [[ "$SKIP_NGINX" != "1" ]]; then
    cat >/etc/nginx/sites-available/ffplayout <<EOF
upstream ffplayout_app { server $BIND_ADDR:$BIND_PORT; keepalive 32; }
server {
    listen 80 default_server;
    listen [::]:80 default_server;
    server_name _;
    client_max_body_size 0;

    # HLS: ffmpeg rewrites the playlist constantly - never buffer, never cache.
    location /hls {
        proxy_pass http://ffplayout_app;
        proxy_http_version 1.1; proxy_set_header Connection "";
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_buffering off; proxy_request_buffering off;
        proxy_read_timeout 3600s; proxy_send_timeout 3600s;
        add_header Cache-Control "no-store" always;
    }
    location /1/live { proxy_pass http://ffplayout_app; proxy_buffering off;
        proxy_http_version 1.1; proxy_set_header Connection "";
        proxy_read_timeout 3600s; add_header Cache-Control "no-store" always; }

    # uploads / exports can be huge
    location /file {
        proxy_pass http://ffplayout_app;
        proxy_http_version 1.1; proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_request_buffering off; proxy_buffering off;
        proxy_read_timeout 3600s; proxy_send_timeout 3600s;
    }

    location / {
        proxy_pass http://ffplayout_app;
        proxy_http_version 1.1; proxy_set_header Connection "";
        proxy_set_header Host \$host;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_read_timeout 300s;
    }
}
EOF
    rm -f /etc/nginx/sites-enabled/default
    ln -sf /etc/nginx/sites-available/ffplayout /etc/nginx/sites-enabled/ffplayout
    nginx -t && systemctl reload nginx && log "nginx configured"
else
    log "SKIP_NGINX=1 - app reachable directly on :$BIND_PORT"
fi

# =============================================================================
step "9/9  Database + start"
# =============================================================================
PY="$APP_DIR/.venv/bin/python"
if [[ ! -f "$DATA_DIR/db.json" ]]; then
    log "creating database at $DATA_DIR/db.json"
    sudo -u "$APP_USER" "$PY" "$APP_DIR/db_admin.py" --db "$DATA_DIR/db.json" init
else
    log "database already present - left untouched"
    log "  review it with:  sudo -u $APP_USER $PY $APP_DIR/db_admin.py --db $DATA_DIR/db.json info"
    log "  Windows paths inside db.json will NOT resolve here - repoint"
    log "  storage/playlists/public to Linux paths, then:"
    log "    sudo -u $APP_USER $PY $APP_DIR/db_admin.py --db $DATA_DIR/db.json export --out /tmp/db.json"
fi

if [[ "$ENABLE_SERVICE" == "1" ]]; then
    systemctl restart ffplayout-playout
    sleep 4
    if systemctl is-active --quiet ffplayout-playout; then
        log "service is RUNNING"
    else
        warn "service failed to start:"
        systemctl --no-pager --full status ffplayout-playout || true
        journalctl -u ffplayout-playout -n 40 --no-pager || true
    fi
fi

cat <<EOF

===============================================================================
 ffplayout deployed to $APP_DIR

   web UI        http://<server-ip>:$BIND_PORT        (nginx on :80 if enabled)
   database      $DATA_DIR/db.json
   frontend      $APP_DIR/frontend/dist
   logs          journalctl -u ffplayout-playout -f
   media         $DATA_DIR/storage/<Channel Name>/

   login         admin / admin   -> CHANGE IT NOW

 Commands:
   systemctl status ffplayout-playout
   systemctl restart ffplayout-playout
   journalctl -u ffplayout-playout -f

 Database:
   sudo -u $APP_USER $PY $APP_DIR/db_admin.py --db $DATA_DIR/db.json info
   sudo -u $APP_USER $PY $APP_DIR/db_admin.py --db $DATA_DIR/db.json backup

 Before going live:
   1. change the admin password in the UI
   2. change SECRET_KEY in $APP_DIR/server.py (hardcoded default)
   3. repoint channel storage/playlists/public at real media, chown -R $APP_USER
   4. TLS:  apt install certbot python3-certbot-nginx && certbot --nginx -d your.domain
===============================================================================
EOF