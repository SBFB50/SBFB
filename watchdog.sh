#!/bin/bash
# NEXUS GOV Watchdog -- auto-restart on crash
cd "$(dirname "$0")"

# Activate conda env if available
if command -v conda &>/dev/null; then
    eval "$(conda shell.bash hook)"
    conda activate nexus 2>/dev/null
fi

while true; do
    echo "[$(date)] Starting NEXUS backend..."
    uvicorn nexus.main:app --host 0.0.0.0 --port 8000
    EXIT_CODE=$?
    echo "[$(date)] Backend exited with code $EXIT_CODE. Restarting in 5 seconds..."
    sleep 5
done
