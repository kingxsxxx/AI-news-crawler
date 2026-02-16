# AI News Terminal - 桌面快捷方式创建脚本

$ErrorActionPreference = "Stop"

# 获取脚本所在目录
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# 检查是否已构建
if (Test-Path "$ScriptDir\src-tauri\target\release\ai-news-aggregator.exe") {
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "   AI NEWS TERMINAL" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host ""

    Write-Host "创建桌面快捷方式..." -ForegroundColor Green

    # 桌面路径
    $DesktopPath = [Environment]::GetFolderPath("Desktop")

    # 快捷方式路径
    $ShortcutPath = "$DesktopPath\AI News Terminal.lnk"

    # 使用 PowerShell 创建快捷方式
    $WshShell = New-Object -ComObject WScript.Shell
    $Shortcut = $WshShell.CreateShortcut($ShortcutPath)
    $Shortcut.TargetPath = "$ScriptDir\src-tauri\target\release\ai-news-aggregator.exe"
    $Shortcut.WorkingDirectory = "$ScriptDir\src-tauri\target\release"
    $Shortcut.Description = "AI 资讯聚合终端"
    $Shortcut.Save()

    Write-Host ""
    Write-Host "✓ 桌面快捷方式已创建！" -ForegroundColor Green
    Write-Host "  位置: $ShortcutPath" -ForegroundColor Gray
    Write-Host ""
    Write-Host "现在可以双击桌面上的快捷方式启动应用了。" -ForegroundColor Yellow
    Write-Host ""
}
else {
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "   未找到已构建的可执行文件" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    Write-Host ""
    Write-Host "请先构建应用：" -ForegroundColor Yellow
    Write-Host "  npm run tauri:build" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "或者直接运行开发版本：" -ForegroundColor Yellow
    Write-Host "  双击 启动应用.bat" -ForegroundColor Cyan
    Write-Host ""
}

Write-Host "按任意键退出..." -ForegroundColor Gray
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
