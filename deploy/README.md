# ffplayout on Ubuntu — Deployment Pack

Everything needed to run this project on Ubuntu Server 22.04 / 24.04.

| File | Purpose |
|---|---|
| `install.sh` | one-shot installer (apt deps, venv, frontend build, systemd, nginx) |
| `start.sh` | foreground start for development |
| `requirements.txt` | pinned Python deps (same versions as the Windows box) |
| `db_admin.py` | **database CLI** — create/init/backup/restore/users/channels |
| `deploy/ffplayout-playout.service` | systemd unit template |
| `deploy/nginx-ffplayout.conf` | nginx reverse proxy template |
| `db/README.md` | database commands cheat-sheet |

`db_admin.py` also lives inside the app directory as `db_admin.py` — the
installer copies it to `/opt/ffplayout/db_admin.py`.

---

## 0. What actually runs on Ubuntu

This checkout is **not** the stock ffplayout release. Two runtimes exist:

| Component | Role | Runs on Ubuntu? |
|---|---|---|
| `server.py` — Python 3.12 + FastAPI | **The real application.** Owns the playout loop, all ffmpeg spawning, the JSON database and the REST API. | ✅ Yes |
| `frontend/` — Vue 3 + Vite + Tailwind 4 | Web UI, built to `frontend/dist` | ✅ Yes |
| `engine/` — Rust + axum + SQLite | Upstream ffplayout engine. **Not used** — `server.py` replaced it. | Not needed |

**The database is `demo_data/db.json`, a single JSON file — not SQLite and not
PostgreSQL.** The Rust engine uses SQLite via `sqlx` + `migrations/*.sql`, but
since `server.py` is what runs, those migrations and `ffplayout -i` are *not*
part of this deployment. Do not run them; they would create a `*.db` file that
nothing reads.

---

## 1. Code changes already made for Linux

Three places in `server.py` were Windows-only and are now cross-platform:

1. **`_sweep_stray_ffmpeg()`** — killed orphans via
   `powershell Get-CimInstance Win32_Process` + `taskkill`, which does not exist
   on Ubuntu. Rewritten on top of `psutil` (already a dependency), so it works
   identically on both. It now also terminates the child processes ffmpeg
   spawned.
2. **`uvicorn.run()`** — was hardcoded to `host="127.0.0.1", port=8787`. Now
   reads `FFPU_HOST` / `FFPU_PORT` / `FFPU_LOG_LEVEL`, so systemd and nginx can
   decide the bind address. Default is still `127.0.0.1` so a bare
   `python3 server.py` is never accidentally exposed.
3. **Static frontend mount** — `server.py` only ever returned JSON; the UI was
   served by the vite dev server. It now mounts `frontend/dist` at `/` (added
   *after* every API route, so `/api`, `/auth`, `/hls` keep priority). One port,
   one origin, no CORS in production.

Plus two optional environment overrides: `FFPU_DATA_DIR` and `FFPU_DB_FILE`, so
the database and media can live on a separate volume.

Everything else already worked on Linux: `_popen_flags()` guards on
`os.name == "nt"`, `_find_ffmpeg()` falls back to `/usr/bin/ffmpeg`,
`_find_blender_executable()` honours `$BLENDER_BIN` and `shutil.which`,
`os.link()` hardlinks work on ext4, and
`_escape_sub_filter_path()` is documented as macOS/Linux-agnostic.

---

## 2. Install

### 2.1 Copy the code to the server

From your Windows box, in PowerShell:

```powershell
# code only (~48 MB, no media)
.\D:\ffplayout-ubuntu\copy-to-ubuntu.ps1 -Server root@<server-ip>

# code + db.json + playlists/  (~50 MB) - keeps your existing channels/users
.\D:\ffplayout-ubuntu\copy-to-ubuntu.ps1 -Server root@<server-ip> -IncludeData

# code + db + playlists + the 14 GB media folder (slow)
.\D:\ffplayout-ubuntu\copy-to-ubuntu.ps1 -Server root@<server-ip> -IncludeData -IncludeMedia
```

The script transfers via `tar | ssh` (not `scp`) because `scp` cannot exclude
anything, and `node_modules/` + `target/` + `.venv/` are ~2 GB of Windows build
artefacts that would arrive useless on Linux. It excludes `demo_data/` by
default because that folder is **14.4 GB** — 14 GB of it is `storage/` media.

Equivalent manual commands if you'd rather not use the script:

```powershell
# 1. code
cd D:\ffplayout-master\ffplayout-master
tar -cf - --exclude node_modules --exclude target --exclude .venv `
        --exclude __pycache__ --exclude .vite --exclude .git `
        --exclude dist --exclude demo_data --exclude *.log . |
  ssh root@<server-ip> "mkdir -p /root/ffplayout && tar -xf - -C /root/ffplayout"

# 2. installer
scp D:\ffplayout-ubuntu\deploy-ubuntu.sh root@<server-ip>:/root/ffplayout/deploy/
```

> Run the manual `tar | ssh` from **cmd.exe**, not PowerShell. PowerShell 5.1
> coerces the binary stream to strings and silently corrupts the archive.
> `copy-to-ubuntu.ps1` handles this for you.

### 2.2 Run the installer

```bash
ssh root@<server-ip>
cd /root/ffplayout/deploy
chmod +x deploy-ubuntu.sh
sudo ./deploy-ubuntu.sh
```

`deploy-ubuntu.sh` is fully self-contained — it writes the systemd unit and the
nginx site itself, so no other file is needed on the server.

Environment overrides (all optional):

```bash
sudo APP_DIR=/srv/ffplayout APP_USER=ffpu BIND_PORT=9000 ./deploy-ubuntu.sh
sudo SKIP_NGINX=1 ./deploy-ubuntu.sh      # no nginx, expose :8787 directly
sudo ENABLE_SERVICE=0 ./deploy-ubuntu.sh   # install files only, no systemd unit
```

### 2.3 What the installer does

1. `apt-get install` ffmpeg, python3-venv, build-essential, nginx, tzdata,
   DejaVu fonts, rsync
2. verifies ffmpeg is ≥ 5 and warns if it was built **without libzmq**
   (dynamic text overlay will not work)
3. installs Node.js 22 via NodeSource if missing
4. creates the `ffpu` system user (no login shell)
5. rsyncs the source to `/opt/ffplayout`, excluding `demo_data/`, so your
   database and media survive upgrades
6. creates `/opt/ffplayout/demo_data/{channel,channels,playlists,public,storage,logs}`
7. builds a venv at `/opt/ffplayout/.venv` and installs `requirements.txt`
8. `npm ci && npm run build` → `frontend/dist`
   (`package.json` is at the **app root**, not in `frontend/` — `vite.config.ts`
   sets `root: './frontend'`, so npm runs from `/opt/ffplayout`)
9. writes `/etc/ffplayout/ffplayout.env`
10. installs + starts the systemd unit and the nginx site

---

## 3. Create the database

**There is no DB server to install.** The database is one JSON file and is
created by the CLI below. Full command reference: [`db/README.md`](db/README.md).

### Create it

```bash
sudo systemctl stop ffplayout-playout          # never write while running

sudo -u ffpu /opt/ffplayout/.venv/bin/python /opt/ffplayout/db_admin.py init
```

That creates `/opt/ffplayout/demo_data/db.json` with one user
(`admin` / `admin`) and one channel (`Channel 1`), and copies the default logo,
DejaVu font and dummy subtitle into the asset folders.

`init` is safe to re-run: it refuses to overwrite an existing database unless
you pass `--force`. If you never call `init`, the server creates the same
default database by itself on first start — the CLI just makes it explicit and
gives you a timestamped backup path.

### Add users and channels

```bash
cd /opt/ffplayout
PY=".venv/bin/python"

sudo -u ffpu $PY db_admin.py add-user \
    --username prashant --password 'S3cret!' --role global_admin

sudo -u ffpu $PY db_admin.py add-channel \
    --id 1 --name 'Channel 1' --timezone Asia/Kolkata

sudo -u ffpu $PY db_admin.py passwd --username admin --password 'NewPass123!'
```

Roles: `global_admin` (everything), `channel_admin` (assigned channels only),
`user` (read-only).

### Migrate the existing Windows database

To carry over the channels, users and playlists you already have instead of
starting clean:

```bash
systemctl stop ffplayout-playout

cp /opt/ffplayout/demo_data/db.json /opt/ffplayout/demo_data/db.json.bak
sudo -u ffpu .venv/bin/python db_admin.py import \
    --file /root/ffplayout/demo_data/db.json --force

systemctl start ffplayout-playout
```

Playlists and media live under `demo_data/`, so copy those across too, or point
the channel `playlists` / `storage` paths at your existing media share (see
§5).

### Backup on a schedule

```bash
sudo -u ffpu /opt/ffplayout/.venv/bin/python /opt/ffplayout/db_admin.py backup
# -> /opt/ffplayout/demo_data/backups/db-20261002-120000.json
```

---

## 4. Verify

```bash
systemctl status ffplayout-playout
journalctl -u ffplayout-playout -f          # live logs

curl -s http://127.0.0.1:8787/api/channels   # returns the channel list
```

Open `http://<server-ip>:8787` (port 8787) and log in as `admin` / `admin`,
then change the password immediately.

```bash
ffmpeg -hide_banner -version | head -1       # must be >= 5.0
ffmpeg -hide_banner -filters | grep zmq      # needed for text overlay
```

---

## 5. Point it at your media

Per channel, under `/opt/ffplayout/demo_data/`:

```
storage/<Channel Name>/     media clips (mp4, mkv, mp3, images)   <- drop files here
playlists/                  daily JSON playlists
public/<Channel Name>/      HLS output (master.m3u8 + segments)
channel/<id>/live/          runtime HLS/stream state
backups/                    db_admin.py backups
```

If your media is on a different disk or an NFS share, either symlink it:

```bash
sudo -u ffpu ln -s /mnt/media/channel1 \
    /opt/ffplayout/demo_data/storage/'Channel 1'
```

or set the `storage` / `playlists` / `public` paths per channel in the UI.
The `ffpu` user needs write access to all of them — HLS segments are written
continuously.

---

## 6. TLS and hardening

```bash
sudo apt install certbot python3-certbot-nginx
sudo certbot --nginx -d playout.example.com
```

Then tighten `/etc/ffplayout/ffplayout.env`:

- change `SECRET_KEY` in `server.py` (line ~181) — it is currently a
  hardcoded default and will forge valid sessions for anyone who knows it
- keep `FFPU_HOST=127.0.0.1` so only nginx can reach the app
- open only 80/443 in the firewall; 8787 stays internal

```bash
sudo ufw allow 22,80,443/tcp && sudo ufw enable
```

---

## 7. Operations

```bash
systemctl restart ffplayout-playout     # restart after a code/config change
journalctl -u ffplayout-playout -f      # follow logs
journalctl -u ffplayout-playout --since "1 hour ago"

# update to a new build
cd /root/ffplayout && git pull
sudo -u ffpu /opt/ffplayout/.venv/bin/python /opt/ffplayout/db_admin.py backup
cd /root/ffplayout-ubuntu && sudo ./install.sh
```

---

## 8. Troubleshooting

| Symptom | Cause / fix |
|---|---|
| `frontend build produced no dist/index.html` | `npm run build` failed — run it manually in `/opt/ffplayout` |
| UI loads but every API call 404s | `frontend/dist` missing; server prints `[web] no frontend build at ...` |
| `Permission denied` on `demo_data/` | `chown -R ffpu:ffpu /opt/ffplayout` |
| Playout does nothing | channel `storage` path is empty or not writable by `ffpu` |
| Text overlay ignored | ffmpeg lacks libzmq — install a build with `--enable-libzmq` |
| HLS stalls at a clip boundary | check `journalctl` for the ffmpeg spawn error; verify each clip with `ffprobe` |
| `Address already in use :8787` | `ss -lptn 'sport = :8787'` |
| Killed on heavy 1080p load | raise `LimitNOFILE`, and check `dmesg \| grep oom` for OOM kills |

---

## 9. Notes / known gaps

- **`SECRET_KEY` is hardcoded** in `server.py`. Change it before exposing the
  service, otherwise JWTs are forgeable.
- **SCTE-35** needs the `threefive` package (installed) — it is a pure-Python
  encoder, so no extra native library.
- **NNL dynamic graphics** need Blender. Optional:
  `sudo snap install blender`, then set `BLENDER_BIN=/snap/bin/blender`.
  Without it the app falls back to baked `.mov` / `drawtext` templates.
- **The Rust engine is unused.** Do not run `cargo build`, `ffplayout -i`, or
  the `migrations/*.sql` files — nothing reads them, and `ffplayout -i` would
  create a competing SQLite `*.db`.