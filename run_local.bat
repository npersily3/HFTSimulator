@echo off

REM Run the HFT Simulator locally with the web GUI.
REM Opens two terminal windows (exchange + bridge) and then the browser.
REM Run this script from the project root.

REM ── Install bridge dependencies if not already present ────────────────────
if not exist "frontend\node_modules" (
    echo Installing bridge dependencies...
    cd frontend
    npm install
    cd ..
)

REM ── Start the exchange ────────────────────────────────────────────────────
REM --features gui compiles in the TCP tick server on port 9000.
REM This window must stay open for the simulation to run.
start "HFT Exchange" cargo run --features gui

REM ── Start the bridge ──────────────────────────────────────────────────────
REM Connects to the exchange on port 9000 and serves the web page on port 8000.
REM The bridge retries the exchange connection automatically, so it is fine
REM if cargo is still compiling when this window opens.
start "HFT Bridge" cmd /k "cd /d "%~dp0frontend" && node server.js"

REM ── Open the browser ──────────────────────────────────────────────────────
REM Wait a moment for the bridge to start before opening the page.
REM The page shows "disconnected" and reconnects on its own if not ready yet.
timeout /t 3 /nobreak >nul
start http://localhost:8000
