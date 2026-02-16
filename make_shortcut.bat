@echo off
set "APP_PATH=C:\Users\浅笑如初\Desktop\Newsagregator\src-tauri\target\release\ai-news-aggregator.exe"
set "SHORTCUT_PATH=C:\Users\浅笑如初\Desktop\AI News Terminal.lnk"

echo Set WshShell = WScript.CreateObject("WScript.Shell") > "%TEMP%\CreateShortcut.vbs"
echo Set Shortcut = WshShell.CreateShortcut("%SHORTCUT_PATH%", True) >> "%TEMP%\CreateShortcut.vbs"
echo Shortcut.TargetPath = "%APP_PATH%" >> "%TEMP%\CreateShortcut.vbs"
echo Shortcut.WorkingDirectory = "C:\Users\浅笑如初\Desktop\Newsagregator\src-tauri\target\release" >> "%TEMP%\CreateShortcut.vbs"
echo Shortcut.Description = "AI News Terminal" >> "%TEMP%\CreateShortcut.vbs"
echo Shortcut.Save >> "%TEMP%\CreateShortcut.vbs"

cscript //nologo "%TEMP%\CreateShortcut.vbs"
del "%TEMP%\CreateShortcut.vbs" 2>nul

if exist "%SHORTCUT_PATH%" (
    echo Shortcut created: %SHORTCUT_PATH%
) else (
    echo Failed to create shortcut
)
pause
