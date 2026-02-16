@echo off
chcp 65001 >nul
cd /d "%~dp0"
echo ========================================
echo    AI NEWS TERMINAL
echo ========================================
echo.

if exist "src-tauri\target\release\ai-news-aggregator.exe" (
    echo 正在启动已构建的应用...
    start "" "src-tauri\target\release\ai-news-aggregator.exe"
    echo 应用已启动！
    timeout /t 2 >nul
) else (
    echo 未找到已构建的可执行文件。
    echo 请先运行以下命令构建应用：
    echo   npm run tauri:build
    echo.
    pause
)
