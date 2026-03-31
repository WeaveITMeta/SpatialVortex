@echo off
REM ─────────────────────────────────────────────────────────────────────────
REM stop_arc_session.bat — Shut down ARC Agent + EustressEngine (Windows)
REM
REM Called automatically by EustressEngine on exit, or manually.
REM ─────────────────────────────────────────────────────────────────────────

echo [arc-session] Shutting down ARC infrastructure...

tasklist /FI "IMAGENAME eq eustress-arc-agent.exe" 2>NUL | find /I "eustress-arc-agent.exe" >NUL
if not errorlevel 1 (
    echo [arc-session] Stopping ARC agent...
    taskkill /IM eustress-arc-agent.exe /F >NUL 2>&1
)

echo [arc-session] ARC infrastructure stopped.
