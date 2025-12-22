' start-minimized.vbs
' Launches universal_copilot.exe minimized
Set objShell = CreateObject("Wscript.Shell")
scriptPath = WScript.ScriptFullName
scriptDir = Left(scriptPath, Len(scriptPath) - Len(WScript.ScriptName))
exePath = scriptDir & "universal_copilot.exe"
If Not CreateObject("Scripting.FileSystemObject").FileExists(exePath) Then
    Wscript.Echo "Cannot find universal_copilot.exe in " & scriptDir
    Wscript.Quit 1
End If
objShell.Run """" & exePath & """", 7, False  ' 7 = minimized, no wait
