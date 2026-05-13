@echo off
setlocal

set ROOT=%~dp0
set PWSH=%ROOT%PowerShell-7.6.1-win-x64\pwsh.exe

if not exist "%PWSH%" (
    echo [ERROR] PowerShell 7.6 not found at %PWSH%
    exit /b 1
)

"%PWSH%" -NoProfile -ExecutionPolicy Bypass -File "%ROOT%scripts\Launcher.ps1" %*
