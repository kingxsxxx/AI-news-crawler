@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion

:: 获取脚本所在目录
set "SCRIPT_DIR=%~dp0"

echo ========================================
echo    AI NEWS TERMINAL
echo ========================================
echo.
echo Creating desktop shortcut...
echo.

:: 桌面路径
set "DESKTOP_PATH=C:\Users\浅笑如初\Desktop"

:: 快捷方式路径
set "SHORTCUT_PATH=%DESKTOP_PATH%\AI News Terminal.lnk"

:: 可执行文件路径
set "EXE_PATH=%SCRIPT_DIR%src-tauri\target\release\ai-news-aggregator.exe"

:: 检查可执行文件是否存在
if not exist "%EXE_PATH%" (
    echo ERROR: Executable not found
    echo %EXE_PATH%
    echo.
    echo Please run: npm run tauri:build
    pause
    exit /b 1
)

:: 创建快捷方式（使用 VBScript 方法）
set "VBSCRIPT=%TEMP%\CreateShortcut.vbs"

echo Set WshShell = WScript.CreateObject("WScript.Shell") > "%VBSCRIPT%"
echo Set Shortcut = WshShell.CreateShortcut("%SHORTCUT_PATH%", True) >> "%VBSCRIPT%"
echo Shortcut.TargetPath = "%EXE_PATH%" >> "%VBSCRIPT%"
echo Shortcut.WorkingDirectory = "%SCRIPT_DIR%src-tauri\target\release" >> "%VBSCRIPT%"
echo Shortcut.Description = "AI News Terminal" >> "%VBSCRIPT%"
echo Shortcut.Save >> "%VBSCRIPT%"

:: 执行 VBScript 创建快捷方式
cscript //nologo "%VBSCRIPT%"

:: 清理临时文件
del "%VBSCRIPT%" 2>nul

if exist "%SHORTCUT_PATH%" (
    echo.
    echo [SUCCESS] Desktop shortcut created!
    echo Location: %SHORTCUT_PATH%
    echo.
    echo You can now double-click the shortcut on your desktop.
) else (
    echo.
    echo [ERROR] Failed to create shortcut
    echo.
    pause
)
