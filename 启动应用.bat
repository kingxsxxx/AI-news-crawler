@echo off
chcp 65001 >nul
cd /d "%~dp0"
echo ========================================
echo    AI NEWS TERMINAL
echo ========================================
echo.
echo 正在启动应用...
echo.

npm run tauri:dev

if errorlevel 1 (
    echo.
    echo 启动失败，请检查：
    echo 1. 是否已安装 Node.js
    echo 2. 是否已安装 Rust 和 Cargo
    echo 3. 是否已运行 npm install
    echo.
    pause
)
