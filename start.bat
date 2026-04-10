@echo off
title NEXUS GOV — Full Distributed GPU System
cd /d "%~dp0"
:: Use miniconda3 base env explicitly (nexus runs in base)
set "PATH=C:\Users\FlowUP\miniconda3;C:\Users\FlowUP\miniconda3\Scripts;C:\Users\FlowUP\miniconda3\Library\bin;%PATH%"

:: Enable ALL distributed GPU features
set COMPUTE_ENABLED=true
set EXO_ENABLED=true
set PETALS_ENABLED=true
set SYNC_ENABLED=true

:: Ollama optimisation
set OLLAMA_FLASH_ATTENTION=1

python start_nexus.py %*
