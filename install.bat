@echo off
cd /d "%~dp0"
pwsh -ExecutionPolicy Bypass -File ".\scripts\install.ps1"
pause
