<#
.DESCRIPTION
  DesignGPT 一键安装脚本。将便携 Node.js 和 PowerShell 运行时解压到工程根目录。
  克隆后只需执行一次: ./scripts/install.ps1
#>

$ErrorActionPreference = "Stop"
$ROOT = Split-Path $PSScriptRoot -Parent

Write-Host "`n  DesignGPT Installer" -ForegroundColor Cyan
Write-Host "  ROOT = $ROOT`n"

# --- 定位 zip ---
$NODE_ZIP  = "$ROOT\scripts\node.zip"
$PWSH_ZIP  = "$ROOT\scripts\PowerShell-7.6.1-win-x64.zip"
$FALLBACK  = "D:\"

if (-not (Test-Path $NODE_ZIP))  { $NODE_ZIP = Join-Path $FALLBACK "node.zip" }
if (-not (Test-Path $PWSH_ZIP))  { $PWSH_ZIP = Join-Path $FALLBACK "PowerShell-7.6.1-win-x64.zip" }

# --- node ---
if (Test-Path "$ROOT\node\node.exe") {
    Write-Host "[SKIP] node/ 已存在" -ForegroundColor Yellow
} else {
    if (-not (Test-Path $NODE_ZIP)) {
        Write-Host "[FAIL] 找不到 node.zip (查找: scripts\ 和 D:\)" -ForegroundColor Red
        exit 1
    }
    Write-Host "[EXTRACT] $NODE_ZIP -> $ROOT\" -ForegroundColor Green
    Expand-Archive -Path $NODE_ZIP -DestinationPath $ROOT -Force
}

# --- powershell ---
if (Test-Path "$ROOT\PowerShell-7.6.1-win-x64\pwsh.exe") {
    Write-Host "[SKIP] PowerShell-7.6.1-win-x64/ 已存在" -ForegroundColor Yellow
} else {
    if (-not (Test-Path $PWSH_ZIP)) {
        Write-Host "[FAIL] 找不到 PowerShell-7.6.1-win-x64.zip (查找: scripts\ 和 D:\)" -ForegroundColor Red
        exit 1
    }
    Write-Host "[EXTRACT] $PWSH_ZIP -> $ROOT\" -ForegroundColor Green
    Expand-Archive -Path $PWSH_ZIP -DestinationPath $ROOT -Force
}

# --- verify ---
Write-Host ""
$env:PATH = "$ROOT\node;$env:PATH"
$ok = $true

try { $v = & "$ROOT\node\node.exe" -v 2>&1; Write-Host "[OK] node  $v" -ForegroundColor Green } catch { Write-Host "[XX] node failed" -ForegroundColor Red; $ok = $false }
try { $v = & "$ROOT\PowerShell-7.6.1-win-x64\pwsh.exe" -NoProfile -Command '$PSVersionTable.PSVersion' 2>&1; Write-Host "[OK] pwsh  $v" -ForegroundColor Green } catch { Write-Host "[XX] pwsh failed" -ForegroundColor Red; $ok = $false }

if ($ok) {
    Write-Host "`n  Install complete. Run .\Launcher.bat to start.`n" -ForegroundColor Cyan
} else {
    Write-Host "`n  Install failed. Check errors above.`n" -ForegroundColor Red
    exit 1
}
