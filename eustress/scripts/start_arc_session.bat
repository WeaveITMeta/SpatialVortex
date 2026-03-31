@echo off
REM ─────────────────────────────────────────────────────────────────────────
REM start_arc_session.bat — Launch EustressEngine + ARC Agent (Windows)
REM
REM Usage:
REM   scripts\start_arc_session.bat [--standalone]
REM
REM Default: EustressStream mode (in-process pub/sub, no external server)
REM ─────────────────────────────────────────────────────────────────────────

setlocal enabledelayedexpansion

set "SCRIPT_DIR=%~dp0"
set "REPO_ROOT=%SCRIPT_DIR%.."
set "ENGINE_DIR=%REPO_ROOT%\EustressEngine\eustress"
set "ENGINE_BIN=%ENGINE_DIR%\target\release\eustress-engine.exe"
set "AGENT_BIN=%REPO_ROOT%\target\release\eustress-arc-agent.exe"

if defined ARC_UNIVERSE_ROOT (
    set "UNIVERSE_DIR=%ARC_UNIVERSE_ROOT%"
) else (
    set "UNIVERSE_DIR=%USERPROFILE%\Documents\Eustress\ARC-AGI-3"
)

set "USE_STREAM=1"
if "%~1"=="--standalone" set "USE_STREAM=0"

echo [arc-session] Universe: %UNIVERSE_DIR%

REM ─── Ensure Universe directory exists ─────────────────────────────────
if not exist "%UNIVERSE_DIR%\spaces" mkdir "%UNIVERSE_DIR%\spaces"
if not exist "%UNIVERSE_DIR%\knowledge" mkdir "%UNIVERSE_DIR%\knowledge"

REM ─── Build if needed ──────────────────────────────────────────────────
if not exist "%AGENT_BIN%" (
    echo [arc-session] Building eustress-arc-agent...
    pushd "%REPO_ROOT%"
    cargo build --release -p eustress-arc-agent --features eustress-streaming
    popd
)

REM ─── EustressEngine ───────────────────────────────────────────────────
if exist "%ENGINE_BIN%" (
    tasklist /FI "IMAGENAME eq eustress-engine.exe" 2>NUL | find /I "eustress-engine.exe" >NUL
    if errorlevel 1 (
        echo [arc-session] Starting EustressEngine...
        start "" "%ENGINE_BIN%" --universe "%UNIVERSE_DIR%"
    ) else (
        echo [arc-session] EustressEngine already running
    )
) else (
    echo [arc-session] EustressEngine not built — skipping
)

REM ─── ARC Agent ────────────────────────────────────────────────────────
if %USE_STREAM%==1 (
    tasklist /FI "IMAGENAME eq eustress-arc-agent.exe" 2>NUL | find /I "eustress-arc-agent.exe" >NUL
    if errorlevel 1 (
        echo [arc-session] Starting ARC agent (EustressStream mode^)...
        set "ARC_UNIVERSE_ROOT=%UNIVERSE_DIR%"
        set "RUST_LOG=eustress_arc_agent=info"
        start "" "%AGENT_BIN%"
    ) else (
        echo [arc-session] ARC agent already running
    )
) else (
    echo [arc-session] Standalone mode — run: python main.py -a vortex
)

echo.
echo [arc-session] ═══════════════════════════════════════════
echo [arc-session]   ARC-AGI-3 Session Ready
echo [arc-session] ═══════════════════════════════════════════
if %USE_STREAM%==1 (
    echo [arc-session]   Mode: EustressStream (in-process, no server^)
    echo [arc-session]   Latency: ^< 1 us in-process, ~50 ns SHM
) else (
    echo [arc-session]   Mode: Standalone (stdin/stdout^)
    echo [arc-session]   To start: cd ARC-AGI-3-Agents ^&^& python main.py -a vortex
)
echo [arc-session] ═══════════════════════════════════════════

endlocal
