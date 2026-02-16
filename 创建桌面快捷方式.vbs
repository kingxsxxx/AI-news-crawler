' AI News Terminal - Desktop Shortcut Creator

Dim WshShell, Shortcut, DesktopPath, AppPath, ShortcutPath

' 创建 WScript Shell 对象
Set WshShell = WScript.CreateObject("WScript.Shell")

' 桌面路径
DesktopPath = WshShell.SpecialFolders("Desktop")

' 快捷方式路径
ShortcutPath = DesktopPath & "\AI News Terminal.lnk"

' 可执行文件路径（使用当前脚本所在目录）
AppPath = WScript.ScriptFullName
AppPath = Left(AppPath, InStrRev(AppPath, "\") - 1) & "src-tauri\target\release\ai-news-aggregator.exe"

' 创建快捷方式
Set Shortcut = WshShell.CreateShortcut(ShortcutPath, True)
Shortcut.TargetPath = AppPath
Shortcut.WorkingDirectory = Left(AppPath, InStrRev(AppPath, "\") - 1) & "src-tauri\target\release"
Shortcut.Description = "AI News Terminal"
Shortcut.Save

' 显示结果
If WshShell.FileExists(ShortcutPath) Then
    WScript.Echo "Success! Desktop shortcut created:"
    WScript.Echo "  " & ShortcutPath
    WScript.Echo ""
    WScript.Echo "You can now double-click the shortcut on your desktop."
Else
    WScript.Echo "Error! Failed to create shortcut."
    WScript.Echo ""
    WScript.Echo "Please check that the executable file exists:"
    WScript.Echo "  " & AppPath
End If

WScript.Sleep 5000
