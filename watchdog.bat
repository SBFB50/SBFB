@echo off
title NEXUS Watchdog
color 0C

:loop
echo [%date% %time%] Starting NEXUS backend...
cd /d %~dp0
call conda activate nexus 2>nul
uvicorn nexus.main:app --host 0.0.0.0 --port 8000

echo [%date% %time%] Backend crashed! Restarting in 5 seconds...
timeout /t 5 /nobreak >nul
goto loop
