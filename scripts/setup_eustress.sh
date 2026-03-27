#!/usr/bin/env bash
# setup_eustress.sh — sparse checkout of EustressEngine into eustress/
#
# Usage:
#   EUSTRESS_REPO=https://github.com/WeaveITMeta/EustressEngine.git ./scripts/setup_eustress.sh
#
# The script:
#   1. Initialises a sparse-checkout clone of EustressEngine under eustress/
#   2. Pulls only the crates needed by the ARC-AGI-3 agent:
#        crates/common, crates/cli, crates/server, crates/mcp
#   3. Patches the workspace Cargo.toml to include the new arc-* crates
#   4. Adds eustress/target/ to .gitignore (build artefacts are not committed)

set -euo pipefail

REPO="${EUSTRESS_REPO:?Set EUSTRESS_REPO to the EustressEngine git URL}"
TARGET_DIR="eustress"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$ROOT_DIR"

# ── 1. Clone with sparse checkout ─────────────────────────────────────────

if [ -d "$TARGET_DIR/.git" ]; then
    echo "[setup_eustress] $TARGET_DIR already initialised — pulling latest"
    git -C "$TARGET_DIR" pull --ff-only
else
    echo "[setup_eustress] Cloning $REPO → $TARGET_DIR (sparse)"
    git clone \
        --filter=blob:none \
        --sparse \
        --depth=1 \
        "$REPO" "$TARGET_DIR"
fi

# ── 2. Enable sparse-checkout patterns ────────────────────────────────────

git -C "$TARGET_DIR" sparse-checkout init --cone
git -C "$TARGET_DIR" sparse-checkout set \
    crates/common \
    crates/cli \
    crates/server \
    crates/mcp

echo "[setup_eustress] Sparse paths applied:"
git -C "$TARGET_DIR" sparse-checkout list

# ── 3. Patch workspace Cargo.toml ─────────────────────────────────────────
#
# Add the three new arc-* crates as workspace members.
# Idempotent — checks before inserting.

WORKSPACE_TOML="$TARGET_DIR/Cargo.toml"

if [ ! -f "$WORKSPACE_TOML" ]; then
    echo "[setup_eustress] WARNING: $WORKSPACE_TOML not found — skipping patch"
else
    patch_member() {
        local member="$1"
        if grep -q "\"$member\"" "$WORKSPACE_TOML"; then
            echo "[setup_eustress] member $member already present — skipping"
        else
            # Insert before the closing bracket of the members array.
            # Works for both single-line and multi-line arrays.
            sed -i "s|^\(\s*\)\]|\1    \"$member\",\n\1]|" "$WORKSPACE_TOML"
            echo "[setup_eustress] Added member: $member"
        fi
    }

    patch_member "crates/arc-types"
    patch_member "crates/arc-agent"
    patch_member "crates/arc-policy"
fi

# ── 4. .gitignore — exclude EustressEngine build artefacts ────────────────

GITIGNORE="$ROOT_DIR/.gitignore"
if ! grep -q "eustress/target/" "$GITIGNORE"; then
    echo "" >> "$GITIGNORE"
    echo "# EustressEngine build artefacts (sparse checkout)" >> "$GITIGNORE"
    echo "eustress/target/" >> "$GITIGNORE"
    echo "[setup_eustress] Added eustress/target/ to .gitignore"
fi

echo ""
echo "[setup_eustress] Done.  Start order:"
echo "  1. iggy-server                                       (port 8090/3000)"
echo "  2. cargo run --manifest-path eustress/Cargo.toml -p eustress-server"
echo "  3. cargo run --manifest-path eustress/Cargo.toml -p eustress-arc-agent"
echo "  4. python arc_bridge/arc_bridge.py --task <TASK_ID>"
