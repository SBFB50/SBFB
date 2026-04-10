@echo off
title NEXUS - Stop
echo Stopping NEXUS services...

for /f "tokens=5" %%a in ('netstat -ano 2^>nul ^| findstr ":8000.*LISTEN"') do (
    taskkill /PID %%a /F >nul 2>&1
    echo   Backend PID %%a stopped.
)

for /f "tokens=5" %%a in ('netstat -ano 2^>nul ^| findstr ":3002.*LISTEN"') do (
    taskkill /PID %%a /F >nul 2>&1
    echo   Frontend PID %%a stopped.
)

docker compose down >nul 2>&1
echo   Docker services stopped.
echo.
echo All NEXUS services stopped.
