#!/usr/bin/env bash
# =============================================================================
#  ffplayout - SRT relay installer
#
#  Bridges the MPEG-TS stream ffplayout writes to UDP with the SRT listener
#  used by the CDN / receiver:
#
#      ffplayout  --->  udp://127.0.0.1:9001  --->  srt://0.0.0.0:5554
#
#  Usage:  sudo ./install-srt.sh
# =============================================================================
set -euo pipefail

SELF_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UDP_PORT="${UDP_PORT:-9001}"
SRT_PORT="${SRT_PORT:-5554}"

log()  { printf '\033[1;32m==>\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m warn:\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31mERROR:\033[0m %s\n' "$*" >&2; exit 1; }

[[ $EUID -eq 0 ]] || die "run with sudo:  sudo ./install-srt.sh"

# --- srt-tools provides srt-live-transmit -----------------------------------
if ! command -v srt-live-transmit >/dev/null 2>&1; then
    log "Installing srt-tools"
    export DEBIAN_FRONTEND=noninteractive
    apt-get update -y
    apt-get install -y --no-install-recommends srt-tools
fi
command -v srt-live-transmit >/dev/null 2>&1 || die "srt-live-transmit not found"
log "srt-live-transmit: $(command -v srt-live-transmit)"

# --- systemd unit ------------------------------------------------------------
UNIT_SRC="$SELF_DIR/srt-transmit.service"
UNIT_DST=/etc/systemd/system/srt-transmit.service
[[ -f "$UNIT_SRC" ]] || die "missing template $UNIT_SRC"

log "Installing $UNIT_DST"
sed -e "s|127.0.0.1:9001|127.0.0.1:${UDP_PORT}|g" \
    -e "s|:5554|:${SRT_PORT}|g" \
    "$UNIT_SRC" >"$UNIT_DST"

systemctl daemon-reload
systemctl enable --now srt-transmit.service
sleep 2
systemctl --no-pager --full status srt-transmit.service || true

cat <<EOF

===============================================================================
 SRT relay installed.

   ffplayout (UDP) : udp://127.0.0.1:${UDP_PORT}
   SRT listener     : srt://0.0.0.0:${SRT_PORT}

 A receiver connects to:  srt://<server-ip>:${SRT_PORT}

 Check it:
   ss -lunp | grep ${UDP_PORT}
   journalctl -u srt-transmit -f
===============================================================================
EOF