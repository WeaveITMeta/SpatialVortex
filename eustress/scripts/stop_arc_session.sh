#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# stop_arc_session.sh — Shut down ARC Agent
#
# Called automatically by EustressEngine on exit, or manually.
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

log() {
    echo "[arc-session] $(date +%H:%M:%S) $*"
}

log "Shutting down ARC infrastructure..."

if pgrep -f "eustress-arc-agent" >/dev/null 2>&1; then
    log "Stopping ARC agent..."
    pkill -f "eustress-arc-agent" || true
fi

log "ARC infrastructure stopped."
