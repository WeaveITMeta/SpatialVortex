#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# start_arc_session.sh — Launch Iggy + EustressEngine + ARC Agent
#
# Checks if each process is already running before spawning.
# Usage:
#   ./scripts/start_arc_session.sh [--standalone]
#
# Modes:
#   default        : Iggy mode — full Eustress Iggy streaming pipeline
#   --standalone   : Standalone mode — stdin/stdout JSON, Python bridge spawns agent
# ─────────────────────────────────────────────────────────────────────────────

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Paths (adjust if your layout differs)
IGGY_SERVER="${IGGY_SERVER:-iggy-server}"
ENGINE_DIR="$REPO_ROOT/EustressEngine/eustress"
ENGINE_BIN="$ENGINE_DIR/target/release/eustress-engine"
AGENT_BIN="$REPO_ROOT/target/release/eustress-arc-agent"
UNIVERSE_DIR="${ARC_UNIVERSE_ROOT:-$HOME/Documents/Eustress/ARC-AGI-3}"

USE_IGGY=true
if [[ "${1:-}" == "--standalone" ]]; then
    USE_IGGY=false
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

# ─── 1. Iggy Server (default — always started unless --standalone) ──────────

if $USE_IGGY; then
    if is_running "iggy-server"; then
        log "Iggy server already running"
    else
        log "Starting Iggy server..."
        if command -v "$IGGY_SERVER" >/dev/null 2>&1; then
            "$IGGY_SERVER" &
            IGGY_PID=$!
            log "Iggy server started (PID $IGGY_PID)"
            sleep 2  # Wait for Iggy to be ready
        else
            log "ERROR: iggy-server not found in PATH."
            log "Install with: cargo install --path /path/to/iggy/core/server"
            exit 1
        fi
    fi
fi

# ─── 2. Ensure Universe directory exists ──────────────────────────────────────

mkdir -p "$UNIVERSE_DIR/spaces"
mkdir -p "$UNIVERSE_DIR/knowledge"
log "Universe: $UNIVERSE_DIR"

# ─── 3. Build binaries if needed ─────────────────────────────────────────────

ensure_built "$AGENT_BIN" "$REPO_ROOT"

if [[ -d "$ENGINE_DIR" ]]; then
    ensure_built "$ENGINE_BIN" "$ENGINE_DIR"
fi

# ─── 4. EustressEngine ───────────────────────────────────────────────────────

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

# ─── 5. ARC Agent ────────────────────────────────────────────────────────────

if is_running "eustress-arc-agent"; then
    log "ARC agent already running"
else
    if $USE_IGGY; then
        log "Starting ARC agent (Iggy mode — default)..."
        ARC_UNIVERSE_ROOT="$UNIVERSE_DIR" \
        RUST_LOG=eustress_arc_agent=info \
            "$AGENT_BIN" &
        AGENT_PID=$!
        log "ARC agent started in Iggy mode (PID $AGENT_PID)"
    else
        log "ARC agent will be started by the Python bridge (standalone mode)"
        log "Run: python main.py -a vortex"
    fi
fi

# ─── 6. Summary ──────────────────────────────────────────────────────────────

echo ""
log "═══════════════════════════════════════════"
log "  ARC-AGI-3 Session Ready"
log "═══════════════════════════════════════════"
log "  Mode:     $(if $USE_IGGY; then echo 'Iggy streaming (default)'; else echo 'Standalone (stdin/stdout)'; fi)"
log "  Universe: $UNIVERSE_DIR"
log "  Iggy:     iggy://iggy:iggy@127.0.0.1:8090"
log "  Stream:   eustress"
log ""
if ! $USE_IGGY; then
    log "  To start a game:"
    log "    cd ARC-AGI-3-Agents && python main.py -a vortex"
fi
log "═══════════════════════════════════════════"
