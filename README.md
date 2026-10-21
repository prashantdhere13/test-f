# ffplayout - 24/7 Broadcast Playout Server

Production deployment of **ffplayout**: a Linux playout engine that runs a 24/7
broadcast channel - schedules clips, burns in graphics and subtitles, and
relays the result out over HLS, MPEG-TS/UDP and SRT.

This repository is the live snapshot of the running instance at
**144.31.146.66** (Ubuntu 24.04 LTS), plus the deployment tooling needed to
rebuild it from scratch on any other server.

```
Web UI (Vue 3)  --HTTP-->  FastAPI / server.py  -->  ffmpeg  --+--> HLS   (nginx /1/live/*.m3u8)
   nginx :443                                                  +--> UDP  (127.0.0.1:9001)
                                                                 +--> SRT  (srt://<ip>:5554)
```

---

## Table of contents

1. [What is in this repository](#1-what-is-in-this-repository)
2. [Requirements](#2-requirements)
3. [Quick install](#3-quick-install)
4. [Step-by-step install](#4-step-by-step-install)
5. [Database](#5-database)
6. [Media and playlist layout](#6-media-and-playlist-layout)
7. [Outputs - HLS, UDP, SRT](#7-outputs---hls-udp-srt)
8. [Service management](#8-service-management)
9. [Verification](#9-verification)
10. [Upgrades](#10-upgrades)
11. [Troubleshooting](#11-troubleshooting)
12. [Security checklist](#12-security-checklist)
13. [Notes and known gaps](#13-notes-and-known-gaps)

---

## 1. What is in this repository

| Path | Purpose |
|---|---|
| `server.py` | **The application.** ~11k lines of Python 3.12 + FastAPI. Owns the playout loop, spawns every ffmpeg process, serves the REST API and hosts the built UI. |
| `frontend/` | Vue 3 + Vite + Tailwind 4 web UI. Built to `frontend/dist`, mounted by `server.py` at `/`. |
| `db_admin.py` | Database CLI: init / add-user / add-channel / import / export / backup / restore / passwd. |
| `deploy/` | Installers and templates for Ubuntu (see below). |
| `docs/` | Upstream ffplayout documentation (playlists, filters, output modes, SCTE-35, ...). |
| `engine/` | Upstream **Rust** engine. **Unused in this deployment** - `server.py` replaced it. See [section 13](#13-notes-and-known-gaps). |
| `migrations/` | SQL migrations for the Rust engine's SQLite DB. **Unused** - nothing reads them. |
| `assets/` | Default logo, DejaVu font and dummy subtitle. |
| `demo_data/` | Live runtime data: `db.json` plus the generated playlists. Media and HLS output are git-ignored. |
| `tests/` | Upstream test suite and fixtures. |

### `deploy/` contents

| File | Purpose |
|---|---|
| `install.sh` | **Primary installer.** apt deps -> service user -> rsync to `/opt/ffplayout` -> venv -> frontend build -> env file -> systemd -> nginx. Idempotent. |
| `deploy-ubuntu.sh` | Self-contained installer variant that writes the systemd unit and nginx site inline (no template files needed). |
| `install-srt.sh` | Installs `srt-tools` and the SRT relay unit (`udp://127.0..1:9001` -> `srt://:5554`). |
| `ffplayout-playout.service` | systemd unit template (`@APP_DIR@`, `@APP_USER@`, `@BIND_ADDR@`, `@BIND_PORT@`). |
| `nginx-ffplayout.conf` | nginx reverse-proxy template with dedicated HLS / upload / WebSocket locations. |
| `srt-transmit.service` | systemd unit for the SRT bridge. |
| `copy-to-ubuntu.ps1` | PowerShell helper that pushes the tree to a remote box over `tar | ssh` (excludes `node_modules`, `.venv`, `demo_data/storage`). |
| `start.sh` | Foreground start for development. |
| `requirements.txt` | Pinned Python dependencies. |
| `README.md` | Longer internal deployment notes. |

---

## 2. Requirements

| Component | Version used on 144.31.146.66 | Minimum |
|---|---|---|
| Ubuntu | 24.04.3 LTS | 22.04 |
| Python | 3.12.3 | 3.10 |
| Node.js | 22.23.3 (npm 10.9.9) | 20 |
| ffmpeg | 6.1.1 (Ubuntu build, **with libzmq**) | 5.0 |
| nginx | 1.24.0 | 1.18 |
| srt-tools | 1.5.3 (`srt-live-transmit`) | optional |
| RAM | 4 GB+ | 2 GB |
| Disk | 20 GB+ plus media | - |

> **libzmq matters.** The `zmq` filter is what makes live text overlay work.
> Ubuntu's stock ffmpeg has it:
> ```bash
> ffmpeg -hide_banner -filters | grep -w zmq
> ```
> If that prints nothing, install a build with `--enable-libzmq`.

---

## 3. Quick install

On a fresh Ubuntu 22.04 / 24.04 server:

```bash
sudo apt-get update && sudo apt-get install -y git
git clone https://github.com/prashantdhere13/test-f.git
cd test-f
sudo ./deploy/install.sh
```

That single command installs everything: system packages, the `ffpu` service
user, the Python venv, the compiled frontend, `/etc/ffplayout/ffplayout.env`,
the systemd unit and the nginx site.

**Useful overrides** (all optional):

```bash
sudo APP_DIR=/srv/ffplayout APP_USER=ffpu BIND_PORT=9000 ./deploy/install.sh
sudo SKIP_NGINX=1 ./deploy/install.sh      # no nginx; the app answers on :8787
sudo ENABLE_SERVICE=0 ./deploy/install.sh   # copy files only, no systemd unit
sudo SRC_DIR=/root/ffplayout ./deploy/install.sh   # source checkout to deploy from
```

Finish with the database (see [section 5](#5-database)) and change the default
`admin` / `admin` password.

---

## 4. Step-by-step install

If you want to run each stage yourself - or you are reproducing the
144.31.146.66 box exactly - follow along.

### 4.1 System packages

```bash
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
    ffmpeg python3 python3-venv python3-pip \
    build-essential pkg-config libssl-dev \
    curl ca-certificates git nginx \
    tzdata fonts-dejavu-core fonts-liberation rsync
```

Node.js 22 (needed only to build the frontend):

```bash
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo bash -
sudo apt-get install -y nodejs
```

Verify:

```bash
ffmpeg -hide_banner -version | head -1      # >= 5.0
ffmpeg -hide_banner -filters | grep -w zmq   # libzmq present
python3 --version && node -v && nginx -v
```

### 4.2 Service user and directories

```bash
sudo useradd --system --home-dir /opt/ffplayout --shell /usr/sbin/nologin ffpu
sudo install -d -m 0755 /opt/ffplayout
sudo install -d -m 0775 -o ffpu -g ffpu /opt/ffplayout/demo_data
```

The app creates its own subfolders on first start (`storage`, `playlists`,
`public`, `channel`, `logs`, `logo`, `text`, `subtitles`).

### 4.3 Copy the code

```bash
sudo rsync -a --delete \
    --exclude '.git/' --exclude '.venv/' --exclude 'node_modules/' \
    --exclude 'target/' --exclude '__pycache__/' --exclude '.vite/' \
    --exclude '*.log' --exclude 'demo_data/storage' --exclude 'demo_data/channel' \
    ./ /opt/ffplayout/
sudo chown -R ffpu:ffpu /opt/ffplayout
```

### 4.4 Python virtualenv

```bash
sudo -u ffpu python3 -m venv /opt/ffplayout/.venv
sudo -u ffpu /opt/ffplayout/.venv/bin/pip install -q --upgrade pip setuptools wheel
sudo -u ffpu /opt/ffplayout/.venv/bin/pip install -q -r /opt/ffplayout/requirements.txt
```

Sanity check the imports:

```bash
sudo -u ffpu /opt/ffplayout/.venv/bin/python -c \
  "import fastapi, uvicorn, psutil, jose, bcrypt, openpyxl, threefive; print('deps OK')"
```

### 4.5 Build the frontend

> `package.json` lives at the **app root**, not in `frontend/`. `vite.config.ts`
> sets `root: './frontend'`, so npm must run from `/opt/ffplayout` and the
> output lands in `/opt/ffplayout/frontend/dist`.

```bash
cd /opt/ffplayout
sudo -u ffpu -H npm ci --no-audit --no-fund --cache /opt/ffplayout/.npm-cache
sudo -u ffpu -H npm run build --cache /opt/ffplayout/.npm-cache
ls -l /opt/ffplayout/frontend/dist/index.html   # must exist
```

### 4.6 Environment file

```bash
sudo install -d -m 0755 /etc/ffplayout
sudo tee /etc/ffplayout/ffplayout.env >/dev/null <<'EOF'
FFPU_HOST=127.0.0.1
FFPU_PORT=8787
FFPU_LOG_LEVEL=info
FFPU_DATA_DIR=/opt/ffplayout/demo_data
FFPU_DB_FILE=/opt/ffplayout/demo_data/db.json
FFPU_SECRET_KEY=replace-with-a-long-random-string
EOF
sudo chmod 0640 /etc/ffplayout/ffplayout.env
sudo chown root:ffpu /etc/ffplayout/ffplayout.env
```

| Variable | Meaning |
|---|---|
| `FFPU_HOST` / `FFPU_PORT` | Bind address. Keep `127.0.0.1` so only nginx can reach the app. |
| `FFPU_LOG_LEVEL` | `info` / `debug` / `warning` / `error` |
| `FFPU_DATA_DIR` | Root of all runtime data. Must match systemd `ReadWritePaths`. |
| `FFPU_DB_FILE` | Path to the JSON database. |
| `FFPU_SECRET_KEY` | Session JWT signing key. **Generate a random one.** Rotating it logs everyone out. |
| `FFPU_AUDIO_FADE` | Audio fade in/out seconds (default `0.04`). |
| `BLENDER_BIN` | Optional. Absolute path to Blender for NNL dynamic graphics. |

### 4.7 systemd unit

```bash
sudo cp /opt/ffplayout/deploy/ffplayout-playout.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now ffplayout-playout
```

Or let the installer substitute the placeholders for you:

```bash
sudo sed -e "s|@APP_DIR@|/opt/ffplayout|g" \
         -e "s|@APP_USER@|ffpu|g" \
         -e "s|@BIND_ADDR@|127.0.0.1|g" \
         -e "s|@BIND_PORT@|8787|g" \
    /opt/ffplayout/deploy/ffplayout-playout.service \
    >/etc/systemd/system/ffplayout-playout.service
```

### 4.8 nginx reverse proxy and TLS

```bash
sudo cp /opt/ffplayout/deploy/nginx-ffplayout.conf /etc/nginx/sites-available/ffplayout
sudo ln -sf /etc/nginx/sites-available/ffplayout /etc/nginx/sites-enabled/ffplayout
sudo rm -f /etc/nginx/sites-enabled/default
sudo nginx -t && sudo systemctl reload nginx
```

The template deliberately splits the locations:

- `/hls` and `/1/live` - `proxy_buffering off` plus `Cache-Control: no-store`,
  because ffmpeg rewrites the playlist continuously.
- `/file` - `proxy_request_buffering off`, because uploads and exports are huge.
- `/` - everything else, with WebSocket upgrade support.

**Self-signed certificate** (what 144.31.146.66 runs today):

```bash
sudo install -d -m 0755 /etc/ssl/ffplayout
sudo openssl req -x509 -newkey rsa:2048 -nodes -days 1095 \
    -keyout /etc/ssl/ffplayout/ffplayout.key \
    -out    /etc/ssl/ffplayout/ffplayout.crt \
    -subj "/C=IN/ST=MH/L=Mumbai/O=Zee/OU=Playout/CN=144.31.146.66"
sudo chmod 0600 /etc/ssl/ffplayout/ffplayout.key
```

Then point the server block at it and redirect 80 to 443:

```nginx
listen 443 ssl http2;
listen [::]:443 ssl http2;
ssl_certificate     /etc/ssl/ffplayout/ffplayout.crt;
ssl_certificate_key /etc/ssl/ffplayout/ffplayout.key;
ssl_protocols       TLSv1.2 TLSv1.3;
add_header Strict-Transport-Security "max-age=31536000" always;
```

**Real certificate** - point a domain at the host and use certbot:

```bash
sudo apt-get install -y certbot python3-certbot-nginx
sudo certbot --nginx -d playout.example.com
```

### 4.9 SRT relay (optional)

Installs `srt-tools` and bridges the UDP output to an SRT listener:

```bash
sudo ./deploy/install-srt.sh
```

This gives you:

```
ffplayout --> udp://127.0.0.1:9001 --> srt-live-transmit --> srt://0.0.0.0:5554
```

A receiver then pulls from `srt://<server-ip>:5554`.

---

## 5. Database

**There is no database server to install.** The database is a single JSON file
at `/opt/ffplayout/demo_data/db.json`, managed through `db_admin.py`.

> Never write to it while the service is running - stop it first.

### Create it

```bash
sudo systemctl stop ffplayout-playout
sudo -u ffpu /opt/ffplayout/.venv/bin/python /opt/ffplayout/db_admin.py init
```

This creates `db.json` with one user (`admin` / `admin`) and one channel, and
copies the default logo, DejaVu font and dummy subtitle into the asset
folders. `init` refuses to overwrite an existing database unless `--force` is
passed. If you never run it, the server creates the same default database
itself on first start.

### Users and channels

```bash
cd /opt/ffplayout
PY=".venv/bin/python"

sudo -u ffpu $PY db_admin.py add-user \
    --username operator --password 'S3cret!' --role global_admin

sudo -u ffpu $PY db_admin.py add-channel \
    --id 1 --name 'Channel 1' --timezone Asia/Kolkata

sudo -u ffpu $PY db_admin.py passwd --username admin --password 'NewPass123!'
```

Roles: `global_admin` (everything) / `channel_admin` (assigned channels only)
/ `user` (read-only).

### Backup and restore

```bash
sudo -u ffpu $PY db_admin.py backup
# -> /opt/ffplayout/demo_data/backups/db-20261002-120000.json

sudo -u ffpu $PY db_admin.py import --file /path/to/old-db.json --force
```

### Migrating from another box

Stop the service, copy `db.json` plus the `playlists/` and `storage/` folders
across, then repoint the Windows-style paths inside `db.json` to the Linux
paths:

```
"public"    -> /opt/ffplayout/demo_data/public/Channel 1
"playlists" -> /opt/ffplayout/demo_data/playlists
"storage"   -> /opt/ffplayout/demo_data/storage/Channel 1
```

---

## 6. Media and playlist layout

Everything lives under `FFPU_DATA_DIR` (`/opt/ffplayout/demo_data`):

```
demo_data/
|-- db.json                        # the database
|-- storage/
|   `-- <Channel Name>/            # source media: mp4, mkv, mp3, images
|-- playlists/
|   `-- <channel_id>_<date>.json   # generated daily playlists
|-- public/
|   `-- <Channel Name>/            # HLS output (master.m3u8 + segments)
|-- channel/<id>/live/             # runtime HLS/stream state
|-- logo/  text/  subtitles/       # overlays
|-- graphics/                      # generated NNL / template graphics
|-- logs/
`-- backups/                       # db_admin.py backups
```

Per-channel paths are configurable in the UI (`storage`, `playlists`, `public`).
To put the media on another disk or an NFS share, symlink it:

```bash
sudo -u ffpu ln -s /mnt/media/channel1 \
    /opt/ffplayout/demo_data/storage/'Channel 1'
```

The `ffpu` user needs **write** access - HLS segments are written continuously.

---

## 7. Outputs - HLS, UDP, SRT

### HLS

Served by nginx straight from `demo_data/public/`:

```
https://<server-ip>/<channel_id>/live/master.m3u8
```

### MPEG-TS over UDP

The channel's output config writes to a local UDP port, which the SRT relay
picks up:

```
udp://127.0.0.1:9001?pkt_size=1316
```

### SRT

```
srt://<server-ip>:5554
```

### Multi-variant HLS

The live instance renders three renditions in parallel:

| Name | Resolution | Bitrate |
|---|---|---|
| 1080p | 1920x1080 | 6000k |
| 720p | 1280x720 | 2800k |
| 480p | 854x480 | 1400k |

---

## 8. Service management

Two units run the deployment:

| Unit | What it does |
|---|---|
| `ffplayout-playout.service` | The FastAPI app plus the ffmpeg playout loop. Runs as `ffpu`. |
| `srt-transmit.service` | `srt-live-transmit` bridge: UDP 9001 -> SRT 5554. |

```bash
sudo systemctl status ffplayout-playout
sudo systemctl restart ffplayout-playout
sudo journalctl -u ffplayout-playout -f            # live logs
sudo journalctl -u ffplayout-playout --since "1 hour ago"
sudo journalctl -u srt-transmit -f
```

The unit is hardened: `NoNewPrivileges`, `PrivateTmp`, `ProtectSystem=full`,
`ProtectHome=read-only`, and `ReadWritePaths` limited to the data directory and
the npm cache.

---

## 9. Verification

```bash
# service
sudo systemctl is-active ffplayout-playout        # -> active

# API
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:8787/                  # 200 (UI)
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:8787/api/channels      # 401 without a token
curl -s -o /dev/null -w '%{http_code}\n' http://127.0.0.1:8787/1/live/master.m3u8 # 200

# ports
ss -lptn 'sport = :8787'
ss -lunp | grep 9001
ss -ltnp | grep 5554

# ffmpeg is actually encoding
pgrep -af ffmpeg
```

Then open `https://<server-ip>` in a browser and log in.

### REST API

70 routes are exposed. The ones you will use most:

| Method | Path | Purpose |
|---|---|---|
| `POST` | `/auth/login/` | Log in, returns a JWT |
| `POST` | `/auth/refresh/` | Refresh an access token |
| `GET` | `/api/channels` | List channels |
| `GET`/`PUT` | `/api/playout/config/{channel_id}` | Channel playout settings |
| `GET`/`PUT` | `/api/playout/outputs/{channel_id}` | Output / stream targets |
| `GET` | `/api/playlist/{channel_id}` | Current playlist |
| `POST` | `/api/playlist/{channel_id}/generate/{date}` | Generate a playlist for a date |
| `POST` | `/api/control/{channel_id}/playout/` | Start / stop playout |
| `POST` | `/api/control/{channel_id}/process/` | Preprocess the playlist |
| `GET` | `/api/control/{channel_id}/stream-info/` | Live stream statistics |
| `GET`/`POST` | `/api/user/`, `/api/users` | User management |

Full list: `grep -nE '^@app\.(get|post|put|delete)' server.py`.

---

## 10. Upgrades

```bash
cd /opt/ffplayout && git pull

# back up the database first
sudo -u ffpu /opt/ffplayout/.venv/bin/python /opt/ffplayout/db_admin.py backup

# re-run the installer (idempotent - safe to re-run after edits)
cd /opt/ffplayout && sudo ./deploy/install.sh

sudo systemctl restart ffplayout-playout
sudo journalctl -u ffplayout-playout -f
```

The installer rsyncs the source into `/opt/ffplayout` **excluding** `demo_data/`,
so your database and media survive upgrades untouched.

---

## 11. Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `frontend build produced no dist/index.html` | `npm run build` failed - run it manually in `/opt/ffplayout` and read the npm error |
| UI loads but every API call 404s | `frontend/dist` missing - the server logs `[web] no frontend build at ...` |
| `Permission denied` on `demo_data/` | `sudo chown -R ffpu:ffpu /opt/ffplayout` |
| Playout does nothing | the channel's `storage` path is empty or not writable by `ffpu` |
| Text overlay ignored | ffmpeg lacks libzmq - install a build with `--enable-libzmq` |
| HLS stalls at a clip boundary | check the ffmpeg spawn error in the journal; verify the clip with `ffprobe` |
| `Address already in use :8787` | `ss -lptn 'sport = :8787'` - another instance is running |
| Killed on heavy 1080p load | raise `LimitNOFILE`; check `dmesg | grep oom` for OOM kills |
| `502 Bad Gateway` through nginx | the app is down - `systemctl status ffplayout-playout` |
| SRT receiver sees nothing | `ss -lunp | grep 9001` and `journalctl -u srt-transmit -f`; confirm the channel's output target is `udp://127.0.0.1:9001` |
| Database looks stale after an edit | the service was running while the file changed - stop it before editing `db.json` |

---

## 12. Security checklist

- [ ] Change the `admin` / `admin` password immediately.
- [ ] Set a random `FFPU_SECRET_KEY` in `/etc/ffplayout/ffplayout.env`
      (`openssl rand -hex 32`). The fallback in `server.py` is public - with it,
      anyone can forge a session token.
- [ ] Keep `FFPU_HOST=127.0.0.1`; let nginx be the only public entry point.
- [ ] Firewall: allow only 22, 80 and 443.
      `sudo ufw allow 22,80,443/tcp && sudo ufw enable`
- [ ] Use a real TLS certificate (certbot) instead of the self-signed one.
- [ ] `/etc/ffplayout/ffplayout.env` is `0640 root:ffpu` - it holds the signing key.
- [ ] **This repository contains the live `demo_data/db.json`**, which holds
      bcrypt password hashes and user e-mail addresses for the running instance.
      Rotate those passwords (or make the repository private) if that matters.

---

## 13. Notes and known gaps

- **This is not the stock upstream ffplayout.** `server.py` is a Python/FastAPI
  reimplementation that replaced the Rust engine. The `engine/` directory, the
  `migrations/*.sql` files and the `ffplayout -i` CLI are **not** part of this
  deployment - nothing reads them. Do not run `cargo build` or the migrations;
  they create a competing SQLite `*.db` file.
- **The database is JSON, not SQLite/PostgreSQL.** It is written by a single
  process, so it is safe, but it is not concurrent-write safe - stop the
  service before hand-editing it.
- **SCTE-35** is handled by the pure-Python `threefive` package; no extra
  native library is needed. Configure it under the channel's output settings.
- **NNL dynamic graphics** need Blender. Optional:
  `sudo snap install blender`, then set `BLENDER_BIN=/snap/bin/blender`.
  Without it the app falls back to baked `.mov` / `drawtext` templates.
- **Media is not in this repository.** `demo_data/storage` (11 GB of source
  media) and the live HLS output are git-ignored - they stay on the server.
- The `engine/` Rust code is kept for reference and future re-integration.
- Stale `server.py.bak*` snapshots on the server are git-ignored; they are
  local edit history, not part of the application.

---

## About upstream ffplayout

ffplayout is a 24/7 broadcasting solution. It can play out a folder containing
audio or video clips, or play a *JSON* playlist for each day, keeping the
current playlist editable. It is designed to run as a system service on Linux.

Upstream project: <https://github.com/ffplayout/ffplayout>
Upstream README (stock release): [docs/upstream-README.md](docs/upstream-README.md)

## License

GPL-3.0 - see [LICENSE](LICENSE).