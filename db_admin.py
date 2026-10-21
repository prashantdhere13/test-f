#!/usr/bin/env python3
"""ffplayout database admin CLI (Ubuntu).

ffplayout's database is a single JSON document at ``demo_data/db.json``.
There is no separate DB server to provision - this script creates, inspects,
backs up and restores that file, and it hashes passwords with the exact same
bcrypt scheme server.py verifies against.

Commands
--------
  init          create db.json if it does not exist (default command)
  reset         archive the current db.json, then create a fresh one
  info          show channels / users / counts
  add-user      create a user (global_admin | channel_admin | user)
  passwd        change a user's password
  del-user      delete a user
  add-channel   create a channel and its folders
  del-channel   delete a channel and its playout config
  backup        timestamped copy under demo_data/backups/
  restore       load a backup file back into db.json
  export        copy db.json somewhere else
  import        overwrite db.json from a file (with --force)

Examples
--------
  sudo -u ffpu /opt/ffplayout/.venv/bin/python /opt/ffplayout/db_admin.py init
  sudo -u ffpu /opt/ffplayout/.venv/bin/python /opt/ffplayout/db_admin.py \
      add-user --username prashant --password 'S3cret!' --role global_admin
  sudo -u ffpu /opt/ffplayout/.venv/bin/python /opt/ffplayout/db_admin.py \
      add-channel --id 1 --name 'Channel 1' --timezone Asia/Kolkata
  sudo -u ffpu /opt/ffplayout/.venv/bin/python /opt/ffplayout/db_admin.py backup
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
from datetime import datetime
from pathlib import Path

APP_DIR = Path(__file__).resolve().parent
DATA_DIR = Path(os.environ.get("FFPU_DATA_DIR", APP_DIR / "demo_data"))
DB_FILE = Path(os.environ.get("FFPU_DB_FILE", DATA_DIR / "db.json"))
BACKUP_DIR = DATA_DIR / "backups"

ROLES = {1: "global_admin", 2: "channel_admin", 3: "user", 4: "guest"}

CHANNEL_FOLDERS = ("channel", "channels", "playlists", "public", "storage", "logs")
ASSET_FOLDERS = ("storage", "logo", "text", "subtitles")


# --------------------------------------------------------------------------- #
# helpers
# --------------------------------------------------------------------------- #
def hash_password(password: str) -> str:
    import bcrypt

    return bcrypt.hashpw(password.encode("utf-8"), bcrypt.gensalt()).decode("utf-8")


def default_db() -> dict:
    """Mirror of server.py's DEFAULT_DB (bcrypt hash generated per install)."""
    return {
        "users": [
            {
                "id": 1,
                "username": "admin",
                "mail": "admin@example.com",
                "password": hash_password("admin"),
                "role_id": 1,
                "channel_ids": [1],
            }
        ],
        "channels": [
            {
                "id": 1,
                "name": "Channel 1",
                "preview_url": "/1/live/master.m3u8",
                "public": str(DATA_DIR / "public" / "Channel 1"),
                "playlists": str(DATA_DIR / "playlists"),
                "storage": str(DATA_DIR / "storage" / "Channel 1"),
                "extra_extensions": "",
                "timezone": "UTC",
                "process": "",
                "auto_preprocess": True,
            }
        ],
        "playout_configs": {
            "1": {
                "general": {"stop_threshold": 2.0},
                "mail": {
                    "show": False,
                    "subject": "",
                    "recipient": "",
                    "mail_level": "warning",
                    "interval": 0,
                    "smtp_server": "",
                    "smtp_port": 465,
                    "smtp_user": "",
                    "smtp_password": "",
                    "smtp_starttls": False,
                },
                "logging": {
                    "ffmpeg_level": "warning",
                    "ingest_level": "warning",
                    "detect_silence": False,
                    "ignore_lines": [],
                },
                "processing": {
                    "mode": "playlist",
                    "audio_only": False,
                    "copy_audio": True,
                    "copy_video": False,
                    "width": 1920,
                    "height": 1080,
                    "aspect": 1.778,
                    "fps": 25.0,
                    "add_logo": False,
                    "logo": "logo/logo.png",
                    "logo_scale": "",
                    "logo_opacity": 0.0,
                    "logo_position": "overlay=W-w-20:20",
                    "audio_tracks": 1,
                    "audio_track_index": 0,
                    "audio_channels": 2,
                    "volume": 1.0,
                    "custom_filter": "",
                    "override_filter": False,
                    "vtt_enable": False,
                    "vtt_dummy": "subtitles/dummy.vtt",
                    "burn_subtitles": False,
                },
                "output": {
                    "output": None,
                    "output_param": "",
                    "hls": {
                        "hls_enable": False,
                        "hls_path": str(DATA_DIR / "public" / "Channel 1"),
                        "hls_list_size": 6,
                        "hls_time": 2,
                        "segment_duration": 4.0,
                        "hls_delete_threshold": 3,
                        "hls_abr": [],
                    },
                },
                "ingest": {
                    "ingest_enable": False,
                    "input_param": "",
                    "input_args": "",
                    "fps": 25.0,
                },
                # required by the web ui: config.ts does
                # `data.playlist.startInSec = ...` unguarded, so omitting this
                # makes the Playback page report "No playout config found!"
                "playlist": {
                    "day_start": "00:00:00",
                    "length": "24:00:00",
                    "infinit": False,
                },
                "quality": {
                    "x264_preset": "",
                    "crf": None,
                    "video_bitrate": "",
                    "maxrate": "",
                    "bufsize": "",
                    "audio_bitrate": "128k",
                    "sample_rate": 48000,
                },
                "secondary_events": {
                    "enable": True,
                    "graphic_enable": False,
                    "graphic": "",
                    "graphic_position": "bottom right",
                    "graphic_scale": 20,
                    "styles": {"nnl": {"position": "top right", "size": 100}},
                },
                "hls_variants": [
                    {"name": "1080p", "width": 1920, "height": 1080, "bitrate": "6000k", "audio": "192k"},
                    {"name": "720p", "width": 1280, "height": 720, "bitrate": "2800k", "audio": "128k"},
                    {"name": "480p", "width": 854, "height": 480, "bitrate": "1400k", "audio": "128k"},
                    {"name": "240p", "width": 426, "height": 240, "bitrate": "400k", "audio": "64k"},
                ],
                "storage": {
                    "filler": "",
                    "extensions": ["mp4", "mxf", "mpg", "mov"],
                    "shuffle": False,
                    "shared_storage": False,
                },
            }
        },
        "outputs": [],
        "advanced_configs": [],
        "presets": [],
    }


def load_db() -> dict:
    if not DB_FILE.exists():
        raise SystemExit(
            f"no database at {DB_FILE}\n"
            f"create it first:  {sys.executable} {__file__} init"
        )
    with open(DB_FILE, "r", encoding="utf-8-sig") as f:
        db = json.load(f)
    for key in ("users", "channels", "playout_configs",
                "outputs", "advanced_configs", "presets"):
        db.setdefault(key, [] if key != "playout_configs" else {})
    return db


def save_db(db: dict) -> None:
    DB_FILE.parent.mkdir(parents=True, exist_ok=True)
    tmp = DB_FILE.with_suffix(".json.tmp")
    with open(tmp, "w", encoding="utf-8") as f:
        json.dump(db, f, indent=2, ensure_ascii=False)
    os.replace(tmp, DB_FILE)
    print(f"wrote {DB_FILE}")


def make_dirs() -> None:
    for folder in CHANNEL_FOLDERS:
        (DATA_DIR / folder).mkdir(parents=True, exist_ok=True)
    for folder in ASSET_FOLDERS:
        (DATA_DIR / "storage" / folder).mkdir(parents=True, exist_ok=True)
    assets_src = APP_DIR / "assets"
    for src_name, target in {
        "logo.png": DATA_DIR / "storage" / "logo" / "logo.png",
        "DejaVuSans.ttf": DATA_DIR / "storage" / "text" / "DejaVuSans.ttf",
        "dummy.vtt": DATA_DIR / "storage" / "subtitles" / "dummy.vtt",
    }.items():
        src = assets_src / src_name
        if src.is_file() and not target.exists():
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, target)
            print(f"  asset  {src_name} -> {target}")


def stop_hint() -> None:
    print("\n!! stop the service before writing:  "
          "systemctl stop ffplayout-playout")


# --------------------------------------------------------------------------- #
# commands
# --------------------------------------------------------------------------- #
def cmd_init(a: argparse.Namespace) -> int:
    if DB_FILE.exists() and not a.force:
        print(f"database already exists: {DB_FILE}  (use --force to recreate)")
        return 0
    stop_hint()
    make_dirs()
    save_db(default_db())
    print("created default database: 1 user (admin/admin), 1 channel (Channel 1)")
    return 0


def cmd_reset(a: argparse.Namespace) -> int:
    stop_hint()
    BACKUP_DIR.mkdir(parents=True, exist_ok=True)
    if DB_FILE.exists():
        stamp = datetime.now().strftime("%Y%m%d-%H%M%S")
        arch = BACKUP_DIR / f"db-{stamp}.json"
        shutil.copy2(DB_FILE, arch)
        DB_FILE.unlink()
        print(f"archived old database -> {arch}")
    make_dirs()
    save_db(default_db())
    print("fresh database created")
    return 0


def cmd_info(a: argparse.Namespace) -> int:
    db = load_db()
    print(f"database : {DB_FILE}")
    print(f"size     : {DB_FILE.stat().st_size} bytes")
    print(f"channels : {len(db['channels'])}")
    for c in db["channels"]:
        cfg = db["playout_configs"].get(str(c.get("id")), {})
        proc = (cfg.get("processing") or {}).get("mode", "?")
        out = (cfg.get("output") or {}).get("output") or "null"
        print(f"   [{c.get('id')}] {c.get('name'):<20} "
              f"mode={proc:<9} out={out:<8} tz={c.get('timezone')}")
    print(f"users    : {len(db['users'])}")
    for u in db["users"]:
        print(f"   [{u.get('id')}] {u.get('username'):<16} "
              f"role={ROLES.get(u.get('role_id'), u.get('role_id')):<14} "
              f"channels={u.get('channel_ids')}")
    print(f"outputs  : {len(db['outputs'])}   "
          f"presets: {len(db['presets'])}   "
          f"advanced: {len(db['advanced_configs'])}")
    return 0


def cmd_add_user(a: argparse.Namespace) -> int:
    stop_hint()
    db = load_db()
    if any(u["username"] == a.username for u in db["users"]):
        print(f"user '{a.username}' already exists")
        return 1
    role_id = next((k for k, v in ROLES.items() if v == a.role), None)
    if role_id is None:
        print(f"unknown role '{a.role}' (use: {', '.join(ROLES.values())})")
        return 1
    chans = [int(c) for c in (a.channels or [])] or \
        [c["id"] for c in db["channels"]]
    db["users"].append({
        "id": max((u.get("id", 0) for u in db["users"]), default=0) + 1,
        "username": a.username,
        "mail": a.mail or f"{a.username}@example.com",
        "password": hash_password(a.password),
        "role_id": role_id,
        "channel_ids": chans,
    })
    save_db(db)
    print(f"created {a.role} '{a.username}' for channels {chans}")
    return 0


def cmd_passwd(a: argparse.Namespace) -> int:
    stop_hint()
    db = load_db()
    for u in db["users"]:
        if u["username"] == a.username:
            u["password"] = hash_password(a.password)
            save_db(db)
            print(f"password updated for '{a.username}'")
            return 0
    print(f"no such user: {a.username}")
    return 1


def cmd_del_user(a: argparse.Namespace) -> int:
    stop_hint()
    if a.username == "admin":
        print("refusing to delete the last-resort 'admin' user")
        return 1
    db = load_db()
    before = len(db["users"])
    db["users"] = [u for u in db["users"] if u["username"] != a.username]
    if len(db["users"]) == before:
        print(f"no such user: {a.username}")
        return 1
    save_db(db)
    print(f"deleted user '{a.username}'")
    return 0


def cmd_add_channel(a: argparse.Namespace) -> int:
    stop_hint()
    db = load_db()
    if any(int(c.get("id")) == a.id for c in db["channels"]):
        print(f"channel id {a.id} already exists")
        return 1
    db["channels"].append({
        "id": a.id,
        "name": a.name,
        "preview_url": f"/{a.id}/live/master.m3u8",
        "public": str(DATA_DIR / "public" / a.name),
        "playlists": str(DATA_DIR / "playlists"),
        "storage": str(DATA_DIR / "storage" / a.name),
        "extra_extensions": "",
        "timezone": a.timezone,
        "process": "",
        "auto_preprocess": a.auto_preprocess,
    })
    db["playout_configs"][str(a.id)] = default_db()["playout_configs"]["1"]
    db["playout_configs"][str(a.id)]["output"]["hls"]["hls_path"] = \
        str(DATA_DIR / "public" / a.name)
    for sub in (a.name,):
        (DATA_DIR / "public" / sub).mkdir(parents=True, exist_ok=True)
        (DATA_DIR / "storage" / sub).mkdir(parents=True, exist_ok=True)
    save_db(db)
    print(f"created channel {a.id} '{a.name}' (tz={a.timezone})")
    print(f"  media   -> {DATA_DIR / 'storage' / a.name}")
    print(f"  hls out -> {DATA_DIR / 'public' / a.name}")
    return 0


def cmd_del_channel(a: argparse.Namespace) -> int:
    stop_hint()
    db = load_db()
    before = len(db["channels"])
    db["channels"] = [c for c in db["channels"] if int(c.get("id")) != a.id]
    if len(db["channels"]) == before:
        print(f"no channel with id {a.id}")
        return 1
    db["playout_configs"].pop(str(a.id), None)
    save_db(db)
    print(f"deleted channel {a.id}")
    return 0


def cmd_backup(a: argparse.Namespace) -> int:
    db = load_db()
    BACKUP_DIR.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    dest = Path(a.out) if a.out else BACKUP_DIR / f"db-{stamp}.json"
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(DB_FILE, dest)
    print(f"backup -> {dest}  ({len(db['channels'])} channels, "
          f"{len(db['users'])} users)")
    return 0


def cmd_restore(a: argparse.Namespace) -> int:
    stop_hint()
    src = Path(a.file)
    if not src.is_file():
        print(f"no such backup: {src}")
        return 1
    with open(src, "r", encoding="utf-8-sig") as f:
        json.load(f)  # validate before destroying anything
    BACKUP_DIR.mkdir(parents=True, exist_ok=True)
    stamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    shutil.copy2(DB_FILE, BACKUP_DIR / f"db-{stamp}-prerestore.json")
    shutil.copy2(src, DB_FILE)
    print(f"restored {DB_FILE} from {src}")
    return 0


def cmd_export(a: argparse.Namespace) -> int:
    db = load_db()
    dest = Path(a.out)
    dest.parent.mkdir(parents=True, exist_ok=True)
    with open(dest, "w", encoding="utf-8") as f:
        json.dump(db, f, indent=2, ensure_ascii=False)
    print(f"exported -> {dest}")
    return 0


def cmd_import(a: argparse.Namespace) -> int:
    stop_hint()
    src = Path(a.file)
    if not src.is_file():
        print(f"no such file: {src}")
        return 1
    with open(src, "r", encoding="utf-8-sig") as f:
        imported = json.load(f)
    if DB_FILE.exists():
        if not a.force:
            print(f"{DB_FILE} exists - re-run with --force to overwrite")
            return 1
        stamp = datetime.now().strftime("%Y%m%d-%H%M%S")
        BACKUP_DIR.mkdir(parents=True, exist_ok=True)
        shutil.copy2(DB_FILE, BACKUP_DIR / f"db-{stamp}-preimport.json")
    save_db(imported)
    print(f"imported {src} -> {DB_FILE}")
    return 0


# --------------------------------------------------------------------------- #
def main() -> int:
    p = argparse.ArgumentParser(
        description="ffplayout JSON database admin",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    p.add_argument("--db", help=f"override database path (default {DB_FILE})")
    sub = p.add_subparsers(dest="cmd")

    sub.add_parser("init", help="create db.json if missing").add_argument(
        "--force", action="store_true")

    sub.add_parser("reset", help="archive + recreate db.json")

    sub.add_parser("info", help="show channels, users and counts")

    u = sub.add_parser("add-user", help="create a user")
    u.add_argument("--username", required=True)
    u.add_argument("--password", required=True)
    u.add_argument("--mail")
    u.add_argument("--role", default="global_admin",
                   choices=sorted(ROLES.values()))
    u.add_argument("--channels", nargs="*", type=int)

    u = sub.add_parser("passwd", help="change a password")
    u.add_argument("--username", required=True)
    u.add_argument("--password", required=True)

    u = sub.add_parser("del-user", help="delete a user")
    u.add_argument("--username", required=True)

    c = sub.add_parser("add-channel", help="create a channel")
    c.add_argument("--id", type=int, required=True)
    c.add_argument("--name", required=True)
    c.add_argument("--timezone", default="UTC")
    c.add_argument("--auto-preprocess", action="store_true")

    c = sub.add_parser("del-channel", help="delete a channel")
    c.add_argument("--id", type=int, required=True)

    b = sub.add_parser("backup", help="timestamped copy under demo_data/backups")
    b.add_argument("--out")

    r = sub.add_parser("restore", help="restore from a backup")
    r.add_argument("--file", required=True)

    e = sub.add_parser("export", help="copy db.json elsewhere")
    e.add_argument("--out", required=True)

    i = sub.add_parser("import", help="overwrite db.json from a file")
    i.add_argument("--file", required=True)
    i.add_argument("--force", action="store_true")

    a = p.parse_args()

    if a.db:
        globals()["DB_FILE"] = Path(a.db)

    cmd = a.cmd or "init"
    return {
        "init": cmd_init, "reset": cmd_reset, "info": cmd_info,
        "add-user": cmd_add_user, "passwd": cmd_passwd, "del-user": cmd_del_user,
        "add-channel": cmd_add_channel, "del-channel": cmd_del_channel,
        "backup": cmd_backup, "restore": cmd_restore,
        "export": cmd_export, "import": cmd_import,
    }[cmd](a)


if __name__ == "__main__":
    sys.exit(main() or 0)