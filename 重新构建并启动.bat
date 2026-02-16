@echo off
chcp 65001 >nul
cd /d "%~dp0"

echo ========================================
echo    Rebuild and Start
echo ========================================
echo.

echo Stopping any running instances...
taskkill /F /IM ai-news-aggregator.exe 2>nul
taskkill /F /IM ai_news_aggregator.exe 2>nul

timeout /t 2 >nul

echo.
echo Building application...
echo.

cd src-tauri
cargo build --release 2>nul
cd ..

if errorlevel 1 (
    echo.
    echo [ERROR] Build failed!
    echo.
    pause
    exit /b 1
)

echo.
echo [SUCCESS] Build completed!
echo.
echo Starting application...
echo.

start "" "src-tauri\target\release\ai-news-aggregator.exe"

echo.
echo Application started!
echo.
timeout /t 2 >nul
