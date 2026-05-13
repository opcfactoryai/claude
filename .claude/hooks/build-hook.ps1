#Requires -Version 7
# DesignGPT build hook (PowerShell)
# 每次 src/*.rs 修改后自动编译，通过后复制 exe 到根目录
# 编译失败则强制 Claude 修复，不通过不让过

Write-Host "`n[BUILD] DesignGPT Build Hook — $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -ForegroundColor Cyan
Write-Host "[BUILD] cargo build --release ..." -ForegroundColor Cyan

$buildOutput = cargo build --release 2>&1
$exitCode = $LASTEXITCODE

if ($exitCode -eq 0) {
    Copy-Item -Force target/release/Launcher.exe ./Launcher.exe
    $sizeKB = [math]::Round((Get-Item ./Launcher.exe).Length / 1KB, 1)
    Write-Host "`n[BUILD] PASS — Launcher.exe (${sizeKB} KB) 已复制到根目录`n" -ForegroundColor Green
    exit 0
} else {
    Write-Host "`n[BUILD] FAIL — 退出码: $exitCode" -ForegroundColor Red
    Write-Host "`n=== 编译错误详情 ===" -ForegroundColor Red
    Write-Host $buildOutput -ForegroundColor Red
    Write-Host "========================`n" -ForegroundColor Red

    @"

[HOOK_CRITICAL_FEEDBACK]
编译失败！

错误信息如下：
$buildOutput

指令：
1. 分析上面的编译错误原因
2. 用 Edit 工具修复所有导致编译失败的文件
3. 修复后我会再次运行 build 验证
4. **必须让 cargo build --release 100% 成功才能停止**

请开始修复代码。
[/HOOK_CRITICAL_FEEDBACK]
"@
    exit 2
}
