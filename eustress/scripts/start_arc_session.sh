#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# start_arc_session.sh — Launch EustressEngine + ARC Agent
#
# Usage:
#   ./scripts/start_arc_session.sh [--standalone]
#
# Modes:
#   default        : EustressStream mode — in-process pub/sub, no external server
#   --standalone   : Standalone mode — stdin/stdout JSON, Python bridge spawns agent
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Paths (adjust if your layout differs)
ENGINE_DIR="$REPO_ROOT/EustressEngine/eustress"
ENGINE_BIN="$ENGINE_DIR/target/release/eustress-engine"
AGENT_BIN="$REPO_ROOT/target/release/eustress-arc-agent"
UNIVERSE_DIR="${ARC_UNIVERSE_ROOT:-$HOME/Documents/Eustress/ARC-AGI-3}"

USE_STREAM=true
if [[ "${1:-}" == "--standalone" ]]; then
    USE_STREAM=false
fi

# ─── Helpers ──────────────────────────────────────────────────────────────────

is_running() {
    pgrep -f "$1" >/dev/null 2>&1
}

log() {
    echo "[arc-session] $(date +%H:%M:%S) $*"
}

ensure_built() {
    local bin="$1"
    local crate_dir="$2"
    if [[ ! -f "$bin" ]]; then
        log "Building $(basename "$bin")..."
        (cd "$crate_dir" && cargo build --release)
    fi
}

# ─── 1. Ensure Universe directory exists ──────────────────────────────────────

mkdir -p "$UNIVERSE_DIR/spaces"
mkdir -p "$UNIVERSE_DIR/knowledge"
log "Universe: $UNIVERSE_DIR"

# ─── 2. Build binaries if needed ─────────────────────────────────────────────

ensure_built "$AGENT_BIN" "$REPO_ROOT"

if [[ -d "$ENGINE_DIR" ]]; then
    ensure_built "$ENGINE_BIN" "$ENGINE_DIR"
fi

# ─── 3. EustressEngine ───────────────────────────────────────────────────────

if [[ -f "$ENGINE_BIN" ]]; then
    if is_running "eustress-engine"; then
        log "EustressEngine already running"
    else
        log "Starting EustressEngine..."
        "$ENGINE_BIN" --universe "$UNIVERSE_DIR" &
        ENGINE_PID=$!
        log "EustressEngine started (PID $ENGINE_PID)"
    fi
else
    log "NOTE: EustressEngine binary not found at $ENGINE_BIN — skipping"
fi

# ─── 4. ARC Agent ────────────────────────────────────────────────────────────

if is_running "eustress-arc-agent"; then
    log "ARC agent already running"
else
    if $USE_STREAM; then
        log "Starting ARC agent (EustressStream mode)..."
        ARC_UNIVERSE_ROOT="$UNIVERSE_DIR" \
        RUST_LOG=eustress_arc_agent=info \
            "$AGENT_BIN" &
        AGENT_PID=$!
        log "ARC agent started (PID $AGENT_PID)"
    else
        log "ARC agent will be started by the Python bridge (standalone mode)"
        log "Run: python main.py -a vortex"
    fi
fi

# ─── 5. Summary ──────────────────────────────────────────────────────────────

echo ""
log "═══════════════════════════════════════════"
log "  ARC-AGI-3 Session Ready"
log "═══════════════════════════════════════════"
log "  Mode:     $(if $USE_STREAM; then echo 'EustressStream (in-process, no server)'; else echo 'Standalone (stdin/stdout)'; fi)"
log "  Universe: $UNIVERSE_DIR"
log "  Latency:  < 1 µs in-process | ~50 ns SHM"
log ""
if ! $USE_STREAM; then
    log "  To start a game:"
    log "    cd ARC-AGI-3-Agents && python main.py -a vortex"
fi
log "═══════════════════════════════════════════"
