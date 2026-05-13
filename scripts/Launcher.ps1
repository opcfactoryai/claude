#Requires -Version 7
# Launcher — PowerShell native launcher for Claude Code
# Uses portable Node.js + PowerShell 7.6.

$ErrorActionPreference = "Stop"

$ROOT = Split-Path $PSScriptRoot -Parent
$NODEBIN = Join-Path $ROOT "node"

$env:PATH = "$NODEBIN;$env:PATH"

Write-Host "[BOOT] Checking environment..." -ForegroundColor Cyan

# --- node ---
try {
    $nodeVer = & node -v 2>&1
    Write-Host "  node version = $nodeVer" -ForegroundColor Green
} catch {
    Write-Host "[ERROR] node not found in $NODEBIN" -ForegroundColor Red
    exit 1
}

# --- claude ---
try {
    $claudeVer = & claude --version 2>&1
    Write-Host "  Claude Code version = $claudeVer" -ForegroundColor Green
} catch {
    Write-Host "[WARN] claude --version check failed, continuing anyway..." -ForegroundColor Yellow
}

# --- launch ---
$env:CLAUDE_CONFIG_DIR = Join-Path $ROOT ".claude"
# No CLAUDE_CODE_GIT_BASH_PATH — Git-free by design

Write-Host "[BOOT] Starting Claude..." -ForegroundColor Cyan

& claude.exe
exit $LASTEXITCODE
